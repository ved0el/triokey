//! Signed self-update (docs/ARCHITECTURE.md, 9. Updates). The only network code in TrioKey: plain GETs of two
//! files at fixed GitHub URLs (the signature, then the exe, each through GitHub's redirects),
//! on this module's own thread, never on the hook or tray thread.
//!
//! Trust comes from the minisign signature alone, checked with the public key compiled in from
//! `update-pubkey.txt`. TLS and the host allow-list only keep the requests where they belong.
//! Without a real key in that file nothing here ever connects, downloads or installs.
//! A version is shown to the user, and anything is written to disk, only after the downloaded
//! exe has passed that check: until then the release is treated as if it did not exist.

use std::fs;
use std::io::Write;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::ptr::{null, null_mut};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minisign_verify::{PublicKey, Signature};
use windows_sys::Win32::Networking::WinHttp::*;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK, SW_SHOWNORMAL};

use crate::tray::wide;
use crate::{admin, app, tray};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const PUBKEY: &str = include_str!("../update-pubkey.txt");
const SIG_URL: &str = "https://github.com/ved0el/triokey/releases/latest/download/triokey.exe.minisig";
const EXE_URL: &str = "https://github.com/ved0el/triokey/releases/latest/download/triokey.exe";
const RELEASES_URL: &str = "https://github.com/ved0el/triokey/releases/latest";
/// github.com answers with a redirect to its release-asset CDN (the first name is the one seen
/// in November 2026, the second the one used before it). Exact names, no subdomain matching.
const HOSTS: [&str; 3] = ["github.com", "release-assets.githubusercontent.com", "objects.githubusercontent.com"];
const FILE: &str = "triokey.exe";
const NEW: &str = "triokey.new.exe";
const OLD: &str = "triokey.old.exe";
const MAX_SIG: usize = 4096;
const MAX_EXE: usize = 20 << 20;
const MAX_HOPS: usize = 5;
/// No machine or user identifier, no version: the same for everyone.
const USER_AGENT: &str = "TrioKey-updater";
const START_DELAY: Duration = Duration::from_secs(60);
const WAKE: Duration = Duration::from_secs(3600);
const DAY: u64 = 24 * 3600;
/// One whole request, on top of WinHTTP's per-step timeouts.
const TOTAL: Duration = Duration::from_secs(600);

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Net,
    /// The signature file or its trusted comment is not what a TrioKey release carries.
    Format,
    Signature,
    File,
    /// The new exe cannot be created next to the running one.
    NotWritable,
    /// The swap failed half-way and could not be undone: the folder has no `triokey.exe`.
    Stranded,
}

impl Error {
    fn text(&self) -> &'static str {
        match self {
            Error::Net => "không tải được từ GitHub (mất mạng, hoặc chưa có bản phát hành nào)",
            Error::Format => "không đọc được chữ ký của bản phát hành",
            Error::Signature => "chữ ký không hợp lệ, không cài",
            Error::File | Error::NotWritable => "không ghi được vào thư mục chứa triokey.exe",
            Error::Stranded => "đổi file dở dang: hãy Thoát rồi đổi tên triokey.old.exe thành triokey.exe",
        }
    }
}

/// GET `url` into the sink, at most this many bytes.
type Fetch<'a> = &'a mut dyn FnMut(&str, usize, &mut dyn Write) -> Result<(), Error>;

/// `X.Y.Z`, optionally `-pre` and `+build`, as a value that orders correctly: the last number
/// is 0 for a pre-release, so `1.0.0-rc1 < 1.0.0`. Pre-release tags are not compared with each
/// other (releases are never tagged as pre-releases, see docs/RELEASING.md).
fn parse_version(s: &str) -> Option<[u64; 4]> {
    let s = s.split('+').next()?;
    let (core, release) = match s.split_once('-') {
        Some((core, _)) => (core, 0),
        None => (s, 1),
    };
    let mut v = [0, 0, 0, release];
    let mut parts = core.split('.');
    for n in &mut v[..3] {
        let p = parts.next()?;
        // u64::from_str also takes "+1"
        if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *n = p.parse().ok()?;
    }
    parts.next().is_none().then_some(v)
}

/// The version a trusted comment (`version=X.Y.Z file=triokey.exe`) offers, if it is newer than
/// `current`: no downgrade, no reinstall of the same version. `Err` when the comment is not
/// for this file, or the version is anything but three numbers (`xtask sign` writes nothing
/// else, and the string is shown to the user). The comment is only trustworthy once the
/// signature has been verified; before that the result decides nothing but whether to download.
fn offered(comment: &str, current: &str) -> Result<Option<String>, Error> {
    let field = |key: &str| comment.split_whitespace().find_map(|t| t.strip_prefix(key)?.strip_prefix('='));
    let (Some(version), Some(FILE)) = (field("version"), field("file")) else { return Err(Error::Format) };
    if version.contains(['-', '+']) {
        return Err(Error::Format);
    }
    let (Some(new), Some(cur)) = (parse_version(version), parse_version(current)) else { return Err(Error::Format) };
    Ok((new > cur).then(|| version.to_string()))
}

/// Host and path of a URL the updater may request: https, default port, no user part, a host
/// from the allow-list. Every redirect target goes through this before it is followed.
fn target(url: &str) -> Option<(&str, &str)> {
    let rest = url.strip_prefix("https://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (host, path) = rest.split_at(end);
    // an exact match also rules out "user@host", "host:port" and upper case
    (HOSTS.contains(&host) && path.starts_with('/')).then_some((host, path))
}

/// The key in a minisign public-key file. `None` for the placeholder (comment lines only) and
/// for anything that is not exactly one valid key line.
fn key_from(text: &str) -> Option<PublicKey> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("untrusted comment:"));
    let line = lines.next()?;
    if lines.next().is_some() {
        return None;
    }
    PublicKey::from_base64(line).ok()
}

fn key() -> Option<PublicKey> {
    key_from(PUBKEY)
}

/// This build can update itself (a real key is compiled in).
pub fn available() -> bool {
    key().is_some()
}

/// Step 1: fetch the signature file and read the version it claims. Without a key: no request.
/// Returns that version and the signature text. Nothing in it is verified yet: the version
/// only decides whether to go on to step 2, and must not be shown or stored.
fn find(key: Option<&PublicKey>, current: &str, fetch: Fetch) -> Result<Option<(String, String)>, Error> {
    if key.is_none() {
        return Ok(None);
    }
    let mut sig = Vec::new();
    fetch(SIG_URL, MAX_SIG, &mut sig)?;
    let text = String::from_utf8(sig).map_err(|_| Error::Format)?;
    let comment = Signature::decode(&text).map_err(|_| Error::Format)?;
    Ok(offered(comment.trusted_comment(), current)?.map(|version| (version, text)))
}

/// Step 2: download the exe into memory and verify it against the signature from step 1.
/// Returns the now authentic version and the verified bytes.
fn verified(key: &PublicKey, current: &str, sig_text: &str, fetch: Fetch) -> Result<(String, Vec<u8>), Error> {
    let sig = Signature::decode(sig_text).map_err(|_| Error::Format)?;
    let mut bytes = Vec::new();
    fetch(EXE_URL, MAX_EXE, &mut bytes)?;
    // prehashed signatures only; this also verifies the trusted comment
    key.verify(&bytes, &sig, false).map_err(|_| Error::Signature)?;
    // the comment is now authentic: decide on it again
    let version = offered(sig.trusted_comment(), current)?.ok_or(Error::Signature)?;
    Ok((version, bytes))
}

/// A swap that stopped half-way (see `install`) left no `triokey.exe`: give the old one its
/// name back.
fn repair(exe: &Path, old: &Path) {
    if !exe.exists() && old.exists() {
        let _ = fs::rename(old, exe);
    }
}

/// Step 3: write the verified bytes into `dir` (the running exe's folder), then swap:
/// `triokey.exe` becomes `triokey.old.exe` (Windows lets a running image be renamed) and the
/// new file takes its name.
///
/// The new file is created without sharing and flushed to disk before it is closed, so nothing
/// else can open it while it is written.
/// shortcut: the handle is closed before the two renames, which go by path. Callers only pass
/// a folder of the caller's own privilege level (`mode`), where a swap gains nothing; rename
/// by handle if that ever changes.
/// shortcut: a process killed between the two renames leaves `triokey.old.exe` and
/// `triokey.new.exe` but no `triokey.exe`, and a TrioKey that cannot start cannot repair that.
/// Replacing the name in one step (`ReplaceFileW`, or a hard link) is untested on a running
/// image; switch to it once that has been tried on a real machine.
fn install(dir: &Path, bytes: &[u8]) -> Result<(), Error> {
    let (exe, new, old) = (dir.join(FILE), dir.join(NEW), dir.join(OLD));
    repair(&exe, &old);
    // left by a run that was killed half-way
    let _ = fs::remove_file(&new);
    let mut f = fs::OpenOptions::new().write(true).create_new(true).share_mode(0).open(&new).map_err(|_| Error::NotWritable)?;
    // sync_all: the file must not get its final name while its data is only in the cache
    let written = f.write_all(bytes).and_then(|_| f.sync_all());
    drop(f);
    let swapped = written.map_err(|_| Error::File).and_then(|_| {
        let _ = fs::remove_file(&old);
        fs::rename(&exe, &old).map_err(|_| Error::File)?;
        if fs::rename(&new, &exe).is_ok() {
            return Ok(());
        }
        // Put the running version back under its name. Whatever briefly held the new file
        // (a scanner) may hold this one too: try for a second before giving up.
        for _ in 0..5 {
            if fs::rename(&old, &exe).is_ok() {
                return Err(Error::File);
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        Err(Error::Stranded)
    });
    if swapped.is_err() {
        let _ = fs::remove_file(&new);
    }
    swapped
}

struct Handle(*mut core::ffi::c_void);

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { WinHttpCloseHandle(self.0) };
        }
    }
}

/// WinHTTP: the system's TLS, certificate store and proxy settings. Redirects are followed
/// here, one hop at a time, so that every target passes `target`; cookies and authentication
/// are off, and the request carries no header beyond the fixed User-Agent.
fn http_get(url: &str, cap: usize, out: &mut dyn Write) -> Result<(), Error> {
    let deadline = Instant::now() + TOTAL;
    let mut url = url.to_string();
    unsafe {
        let session = Handle(WinHttpOpen(wide(USER_AGENT).as_ptr(), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, null(), null(), 0));
        if session.0.is_null() || WinHttpSetTimeouts(session.0, 10_000, 10_000, 15_000, 30_000) == 0 {
            return Err(Error::Net);
        }
        for _ in 0..MAX_HOPS {
            let (host, path) = target(&url).ok_or(Error::Net)?;
            let conn = Handle(WinHttpConnect(session.0, wide(host).as_ptr(), 443, 0));
            if conn.0.is_null() {
                return Err(Error::Net);
            }
            let req = Handle(WinHttpOpenRequest(conn.0, wide("GET").as_ptr(), wide(path).as_ptr(), null(), null(), null(), WINHTTP_FLAG_SECURE));
            let off: u32 = WINHTTP_DISABLE_REDIRECTS | WINHTTP_DISABLE_COOKIES | WINHTTP_DISABLE_AUTHENTICATION;
            // fail closed: without this option WinHTTP would follow redirects by itself
            if req.0.is_null()
                || WinHttpSetOption(req.0, WINHTTP_OPTION_DISABLE_FEATURE, (&off as *const u32).cast(), 4) == 0
                || WinHttpSendRequest(req.0, null(), 0, null(), 0, 0, 0) == 0
                || WinHttpReceiveResponse(req.0, null_mut()) == 0
            {
                return Err(Error::Net);
            }
            let (mut status, mut size) = (0u32, 4u32);
            if WinHttpQueryHeaders(
                req.0,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                null(),
                (&mut status as *mut u32).cast(),
                &mut size,
                null_mut(),
            ) == 0
            {
                return Err(Error::Net);
            }
            match status {
                200 => return read_body(&req, cap, deadline, out),
                301 | 302 | 303 | 307 | 308 => {
                    let mut buf = [0u16; 4096];
                    let mut bytes = (buf.len() * 2) as u32;
                    if WinHttpQueryHeaders(req.0, WINHTTP_QUERY_LOCATION, null(), buf.as_mut_ptr().cast(), &mut bytes, null_mut()) == 0 {
                        return Err(Error::Net);
                    }
                    // `bytes` does not count the terminating NUL
                    url = String::from_utf16(&buf[..bytes as usize / 2]).map_err(|_| Error::Net)?;
                }
                _ => return Err(Error::Net),
            }
        }
    }
    Err(Error::Net)
}

/// The size limit is enforced on the bytes that arrive, not on what a header claims.
unsafe fn read_body(req: &Handle, cap: usize, deadline: Instant, out: &mut dyn Write) -> Result<(), Error> {
    let mut buf = vec![0u8; 64 * 1024];
    let mut total = 0;
    loop {
        let mut n = 0u32;
        if Instant::now() > deadline || WinHttpReadData(req.0, buf.as_mut_ptr().cast(), buf.len() as u32, &mut n) == 0 {
            return Err(Error::Net);
        }
        if n == 0 {
            return Ok(());
        }
        total += n as usize;
        if total > cap {
            return Err(Error::Net);
        }
        out.write_all(&buf[..n as usize]).map_err(|_| Error::File)?;
    }
}

/// The folder to install into: the running exe's, but only when the exe is named `triokey.exe`.
/// A copy running as `triokey (1).exe` would otherwise replace a neighbour called
/// `triokey.exe`, stay old itself and do so again every day; it gets the "new version" notice.
fn own_dir(exe: &Path) -> Option<&Path> {
    exe.file_name()?.eq_ignore_ascii_case(FILE).then(|| exe.parent())?
}

/// What this process may do with a newer release.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Not elevated: install into the exe's own folder without asking.
    Auto,
    /// The elevated copy in `%ProgramFiles%\TrioKey`, which only admins can write: install
    /// there, but only when the user clicks.
    Click,
    /// Elevated from any other folder ("Run as administrator"): that folder may be writable by
    /// unelevated programs, so an elevated TrioKey creates, renames and deletes nothing in it.
    /// The exe is still downloaded into memory, to verify the version before announcing it.
    Notify,
}

fn mode() -> Mode {
    match (admin::is_elevated(), admin::is_admin_copy()) {
        (false, _) => Mode::Auto,
        (true, true) => Mode::Click,
        (true, false) => Mode::Notify,
    }
}

#[derive(Default)]
struct State {
    /// A newer version, verified but not installed yet, and whether a click installs it (else
    /// the click leads to the releases page).
    pending: Option<(String, bool)>,
    /// Installed in this run; active from the next start. One install per run: the running
    /// image now is `triokey.old.exe` and stays locked until exit.
    installed: Option<String>,
    /// Shown in the settings window.
    status: String,
    /// (title, text) for the tray to show once.
    balloon: Option<(String, String)>,
}

static STATE: Mutex<State> = Mutex::new(State { pending: None, installed: None, status: String::new(), balloon: None });
static TX: OnceLock<Sender<bool>> = OnceLock::new();

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

fn set(f: impl FnOnce(&mut State)) {
    f(&mut state());
    tray::notify_update();
}

/// The line under the update controls in the settings window.
pub fn status() -> String {
    if !available() {
        return "Bản build này không có khoá cập nhật nên không tự cập nhật.".into();
    }
    state().status.clone()
}

/// The newer version waiting for a click (installs it, or opens the download page), if any: the
/// settings window puts its link after the status while this is Some.
pub fn waiting() -> Option<String> {
    state().pending.as_ref().map(|(v, _)| v.clone())
}

/// The status while a version waits: short, the window adds the link and `click` knows the way.
fn notice(version: &str) -> String {
    format!("Có bản mới v{version}.")
}

/// Tray thread: the notification to show, once.
pub fn take_balloon() -> Option<(String, String)> {
    state().balloon.take()
}

/// Tray menu item, only while a newer version waits.
pub fn menu_label() -> Option<String> {
    state().pending.as_ref().map(|(v, click)| {
        if *click {
            format!("Có bản {v} — bấm để cập nhật")
        } else {
            format!("Có bản {v} — mở trang tải về")
        }
    })
}

/// Settings button: check now, whatever `auto_update` says.
pub fn check_now() {
    if let Some(tx) = TX.get() {
        let _ = tx.send(false);
    }
}

/// Tray thread: the user clicked the notification, the menu item or the settings window's link.
pub fn click() {
    let Some((_, install)) = state().pending.clone() else { return };
    if install {
        if let Some(tx) = TX.get() {
            let _ = tx.send(true);
        }
    } else if admin::is_elevated() {
        // never ShellExecute while elevated (an unelevated program can redirect it): the address as text
        let t = format!("TrioKey đang chạy quyền admin nên không tự mở trình duyệt. Tải bản mới tại:\n\n{RELEASES_URL}");
        unsafe { MessageBoxW(null_mut(), wide(&t).as_ptr(), wide("TrioKey").as_ptr(), MB_OK | MB_ICONINFORMATION) };
    } else {
        unsafe { ShellExecuteW(null_mut(), wide("open").as_ptr(), wide(RELEASES_URL).as_ptr(), null(), null(), SW_SHOWNORMAL) };
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Updater thread. `clicked`: the user asked for the pending version to be installed.
/// `version` exists only after `verified`: what the unverified signature file claims is never
/// shown, stored or acted on beyond starting the download.
fn run(key: &PublicKey, clicked: bool, manual: bool) {
    if let Some(v) = state().installed.clone() {
        set(|s| s.status = format!("Đã cập nhật lên {v}, có hiệu lực lần chạy sau."));
        return;
    }
    if manual {
        set(|s| s.status = if clicked { "Đang tải bản mới…".into() } else { "Đang kiểm tra…".into() });
    }
    let found = find(Some(key), VERSION, &mut http_get);
    if found.is_ok() {
        // a failed check is tried again at the next wake-up
        app::set_last_update_check(now());
    }
    let (claimed, sig) = match found {
        Ok(Some(found)) => found,
        Ok(None) => return set(|s| (s.pending, s.status) = (None, "Đang dùng bản mới nhất.".into())),
        Err(e) => return set(|s| s.status = format!("Chưa kiểm tra được: {}.", e.text())),
    };
    let known = state().pending.as_ref().is_some_and(|(v, _)| *v == claimed);
    if known && !manual {
        // verified and announced earlier in this run: no second download until the user acts
        return;
    }
    let (version, bytes) = match verified(key, VERSION, &sig, &mut http_get) {
        Ok(release) => release,
        Err(e) => return set(|s| s.status = format!("Có bản phát hành mới nhưng chưa dùng được: {}.", e.text())),
    };
    let dir = std::env::current_exe().ok().as_deref().and_then(own_dir).map(Path::to_path_buf);
    let result = match (mode(), dir) {
        (Mode::Auto, Some(dir)) => install(&dir, &bytes),
        (Mode::Click, Some(dir)) if clicked => install(&dir, &bytes),
        (Mode::Click, _) => {
            return set(|s| {
                s.status = notice(&version);
                // announce a version once, not at every daily check
                if !known || manual {
                    s.balloon = Some((format!("Có bản {version} — bấm để cập nhật"), "TrioKey chạy quyền admin chỉ cập nhật khi bạn bấm.".into()));
                }
                s.pending = Some((version, true));
            });
        }
        _ => Err(Error::NotWritable),
    };
    match result {
        Ok(()) => set(|s| {
            let t = format!("Đã cập nhật lên {version}, có hiệu lực lần chạy sau");
            (s.pending, s.status, s.balloon) = (None, format!("{t}."), Some(("TrioKey".into(), t)));
            s.installed = Some(version);
        }),
        Err(Error::NotWritable) => set(|s| {
            s.status = notice(&version);
            if !known || manual {
                s.balloon = Some((format!("Có bản mới {version}"), "Bấm để mở trang tải về.".into()));
            }
            s.pending = Some((version, false));
        }),
        Err(e) => set(|s| s.status = format!("Có bản {version} nhưng chưa cài được: {}.", e.text())),
    }
}

/// Start the updater thread: while `auto_update` is on, one check a day (one that fails is
/// tried again an hour later), the first try a minute after start, plus whatever
/// the user asks for. No key: no thread, no network.
pub fn spawn() {
    let Some(key) = key() else { return };
    if mode() != Mode::Notify {
        // The previous version, and a download cut short. No `repair` here: this process was
        // started from `triokey.exe`, so that name exists.
        if let Some(dir) = std::env::current_exe().ok().as_deref().and_then(own_dir) {
            let _ = fs::remove_file(dir.join(OLD));
            let _ = fs::remove_file(dir.join(NEW));
        }
    }
    let (tx, rx) = channel();
    let _ = TX.set(tx);
    std::thread::spawn(move || {
        let mut wait = START_DELAY;
        loop {
            match rx.recv_timeout(wait) {
                Ok(clicked) => run(&key, clicked, true),
                Err(RecvTimeoutError::Timeout) => {
                    let s = app::settings();
                    // also true when the clock was set back
                    if s.auto_update && now().abs_diff(s.last_update_check) >= DAY {
                        run(&key, false, false);
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
            wait = WAKE;
        }
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::Cursor;

    const EXE: &[u8] = b"MZ pretend this is the new triokey.exe";

    fn keypair() -> (minisign::KeyPair, PublicKey) {
        let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        // through the file format that `xtask keygen` writes
        let pk = key_from(&kp.pk.to_box().unwrap().into_string()).unwrap();
        (kp, pk)
    }

    fn sign(kp: &minisign::KeyPair, data: &[u8], comment: &str) -> String {
        minisign::sign(None, &kp.sk, Cursor::new(data), Some(comment), None).unwrap().into_string()
    }

    /// A release: the two fixed URLs and nothing else.
    fn release<'a>(sig: &'a str, exe: &'a [u8]) -> impl FnMut(&str, usize, &mut dyn Write) -> Result<(), Error> + 'a {
        move |url, cap, out| {
            let body = match url {
                SIG_URL => sig.as_bytes(),
                EXE_URL => exe,
                _ => panic!("unexpected url {url}"),
            };
            assert!(body.len() <= cap);
            out.write_all(body).map_err(|_| Error::File)
        }
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("triokey-update-{name}-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(FILE), "running").unwrap();
        dir
    }

    /// What the settings preview shows, set directly: nothing is checked, fetched or announced.
    pub(crate) fn pretend(pending: Option<(&str, bool)>, status: &str) {
        let mut s = state();
        (s.pending, s.status) = (pending.map(|(v, click)| (v.to_string(), click)), status.to_string());
    }

    #[test]
    fn a_waiting_version_is_short_and_named_once() {
        assert_eq!(notice("0.3.0"), "Có bản mới v0.3.0.");
        pretend(Some(("0.3.0", false)), &notice("0.3.0"));
        assert_eq!((waiting(), menu_label()), (Some("0.3.0".into()), Some("Có bản 0.3.0 — mở trang tải về".into())));
        pretend(None, "Đang dùng bản mới nhất.");
        assert_eq!((waiting(), menu_label()), (None, None));
    }

    #[test]
    fn only_an_exe_named_triokey_exe_is_swapped() {
        assert_eq!(own_dir(Path::new(r"D:\tools\triokey.exe")), Some(Path::new(r"D:\tools")));
        assert_eq!(own_dir(Path::new(r"D:\tools\TrioKey.EXE")), Some(Path::new(r"D:\tools")));
        assert_eq!(own_dir(Path::new(r"D:\tools\triokey (1).exe")), None);
        assert_eq!(own_dir(Path::new(r"D:\tools\triokey.old.exe")), None);
    }

    #[test]
    fn versions_compare_by_number() {
        let v = |s| parse_version(s).unwrap();
        assert!(v("1.10.0") > v("1.9.0"));
        assert!(v("2.0.0") > v("1.99.99"));
        assert!(v("0.1.1") > v("0.1.0"));
        assert_eq!(v("1.2.3"), v("1.2.3+build5"));
        // a pre-release is older than its release and newer than the release before it
        assert!(v("1.0.0-rc1") < v("1.0.0"));
        assert!(v("1.0.0-rc1") > v("0.9.9"));
        for bad in ["", "1", "1.2", "1.2.3.4", "v1.2.3", "1.2.x", "1..3", "+1.2.3", "1.+2.3", "1.2.3 ", "-1.2.3", "99999999999999999999.0.0"] {
            assert_eq!(parse_version(bad), None, "{bad}");
        }
        // what this build calls itself must parse, or no update would ever be offered
        assert!(parse_version(VERSION).is_some());
    }

    #[test]
    fn trusted_comment_offers_only_a_newer_triokey_exe() {
        assert_eq!(offered("version=0.2.0 file=triokey.exe", "0.1.0"), Ok(Some("0.2.0".into())));
        assert_eq!(offered("file=triokey.exe  extra=1 version=1.10.0", "1.9.0"), Ok(Some("1.10.0".into())));
        // same version, downgrade, pre-release of the running version
        assert_eq!(offered("version=0.1.0 file=triokey.exe", "0.1.0"), Ok(None));
        assert_eq!(offered("version=0.0.9 file=triokey.exe", "0.1.0"), Ok(None));
        // a running pre-release (a local build) is offered its release
        assert_eq!(offered("version=0.1.0 file=triokey.exe", "0.1.0-rc1"), Ok(Some("0.1.0".into())));
        for bad in [
            "",
            "timestamp:1700000000",
            "version=0.2.0",
            "file=triokey.exe",
            "version=0.2.0 file=other.exe",
            "version=0.2.0 file=triokey.exe.minisig",
            "version=two file=triokey.exe",
            "version=0.2.0 file=TrioKey.exe",
            "xversion=0.2.0 file=triokey.exe",
            // only three numbers: the string ends up in a notification
            "version=0.1.0-rc1 file=triokey.exe",
            "version=9.9.9-x file=triokey.exe",
            "version=9.9.9+<b>any-text</b> file=triokey.exe",
        ] {
            assert_eq!(offered(bad, "0.1.0"), Err(Error::Format), "{bad}");
        }
    }

    #[test]
    fn only_https_github_hosts_are_requested() {
        assert_eq!(target(SIG_URL), Some(("github.com", "/ved0el/triokey/releases/latest/download/triokey.exe.minisig")));
        assert!(target(EXE_URL).is_some());
        assert_eq!(
            target("https://release-assets.githubusercontent.com/a/b?sig=x%2F&y=1"),
            Some(("release-assets.githubusercontent.com", "/a/b?sig=x%2F&y=1"))
        );
        assert!(target("https://objects.githubusercontent.com/x").is_some());
        for bad in [
            "http://github.com/ved0el/triokey",
            "https://github.com",
            "https://github.com?x",
            "https://GitHub.com/x",
            "https://github.com:8443/x",
            "https://github.com:443/x",
            "https://user@github.com/x",
            "https://github.com@evil.example/x",
            "https://github.com.evil.example/x",
            "https://evil.example/github.com/x",
            "https://raw.githubusercontent.com/x",
            "https://x.release-assets.githubusercontent.com/x",
            "https://github.com\\@evil.example/x",
            "//github.com/x",
            "/ved0el/triokey",
            "ftp://github.com/x",
            "",
        ] {
            assert_eq!(target(bad), None, "{bad}");
        }
    }

    #[test]
    fn placeholder_key_means_no_request_at_all() {
        for text in ["", "\n\n", "# PLACEHOLDER\n# no key yet\n", "untrusted comment: minisign public key\n", "not a key\n"] {
            assert!(key_from(text).is_none(), "{text:?}");
        }
        let (kp, _) = keypair();
        let line = kp.pk.to_base64();
        assert!(key_from(&line).is_some());
        assert!(key_from(&format!("# TrioKey update key\r\n{line}\r\n")).is_some());
        // two keys: which one counts is not for the parser to guess
        assert!(key_from(&format!("{line}\n{line}\n")).is_none());
        let mut never = |_: &str, _: usize, _: &mut dyn Write| -> Result<(), Error> { panic!("the updater connected without a key") };
        assert_eq!(find(None, "0.1.0", &mut never), Ok(None));
    }

    /// The committed file is the placeholder or one valid key, never something in between
    /// that would silently switch updates off.
    #[test]
    fn committed_key_file_is_placeholder_or_valid() {
        let has_key_line = PUBKEY.lines().map(str::trim).any(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("untrusted comment:"));
        assert_eq!(available(), has_key_line);
    }

    /// `find` then `verified`, as `run` chains them.
    fn check(pk: &PublicKey, current: &str, fetch: Fetch) -> Result<Option<(String, Vec<u8>)>, Error> {
        let Some((_, sig)) = find(Some(pk), current, fetch)? else { return Ok(None) };
        verified(pk, current, &sig, fetch).map(Some)
    }

    #[test]
    fn finds_a_newer_signed_release() {
        let (kp, pk) = keypair();
        let sig = sign(&kp, EXE, "version=0.2.0 file=triokey.exe");
        assert_eq!(find(Some(&pk), "0.1.0", &mut release(&sig, EXE)), Ok(Some(("0.2.0".into(), sig.clone()))));
        assert_eq!(check(&pk, "0.1.0", &mut release(&sig, EXE)), Ok(Some(("0.2.0".into(), EXE.to_vec()))));
        assert_eq!(find(Some(&pk), "0.2.0", &mut release(&sig, EXE)), Ok(None));
        assert_eq!(find(Some(&pk), "0.3.0", &mut release(&sig, EXE)), Ok(None));
        assert_eq!(find(Some(&pk), "0.1.0", &mut release("<html>Not Found</html>", EXE)), Err(Error::Format));
        let mut offline = |_: &str, _: usize, _: &mut dyn Write| Err(Error::Net);
        assert_eq!(find(Some(&pk), "0.1.0", &mut offline), Err(Error::Net));
    }

    /// One check fetches each of the two files once, the signature first.
    #[test]
    fn a_check_fetches_each_file_once() {
        let (kp, pk) = keypair();
        let sig = sign(&kp, EXE, "version=0.2.0 file=triokey.exe");
        let mut urls = Vec::new();
        let mut counting = |url: &str, cap: usize, out: &mut dyn Write| {
            urls.push(url.to_string());
            release(&sig, EXE)(url, cap, out)
        };
        assert!(check(&pk, "0.1.0", &mut counting).is_ok());
        assert_eq!(urls, [SIG_URL, EXE_URL]);
    }

    #[test]
    fn installs_verified_bytes_and_keeps_the_old_exe() {
        let dir = temp("ok");
        // a leftover from a run that was killed is replaced
        fs::write(dir.join(NEW), "half a download").unwrap();
        assert_eq!(install(&dir, EXE), Ok(()));
        assert_eq!(fs::read(dir.join(FILE)).unwrap(), EXE);
        assert_eq!(fs::read(dir.join(OLD)).unwrap(), b"running");
        assert!(!dir.join(NEW).exists());
        // a swap that stopped half-way (no triokey.exe): the old exe gets its name back first
        fs::remove_file(dir.join(FILE)).unwrap();
        assert_eq!(install(&dir, b"MZ newer"), Ok(()));
        assert_eq!(fs::read(dir.join(FILE)).unwrap(), b"MZ newer");
        assert_eq!(fs::read(dir.join(OLD)).unwrap(), b"running");
        // a folder the new file cannot be created in is reported as such
        assert_eq!(install(&dir.join("missing"), EXE), Err(Error::NotWritable));
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Nothing unverified comes out of a check: no version to announce, no bytes to install.
    #[test]
    fn refuses_whatever_the_signature_does_not_cover() {
        let (kp, pk) = keypair();
        let (other, _) = keypair();
        let good = sign(&kp, EXE, "version=0.2.0 file=triokey.exe");

        // other bytes than the signed ones
        assert_eq!(check(&pk, "0.1.0", &mut release(&good, b"MZ something else")), Err(Error::Signature));
        // signed by another key
        let foreign = sign(&other, EXE, "version=0.2.0 file=triokey.exe");
        assert_eq!(check(&pk, "0.1.0", &mut release(&foreign, EXE)), Err(Error::Signature));
        // An old release replayed with its comment edited to look new: the comment gets the
        // download started, the signature over the comment stops everything else.
        let old = sign(&kp, EXE, "version=0.0.9 file=triokey.exe");
        let edited = old.replace("version=0.0.9", "version=9.9.9");
        assert!(find(Some(&pk), "0.1.0", &mut release(&edited, EXE)).is_ok_and(|f| f.is_some()));
        assert_eq!(check(&pk, "0.1.0", &mut release(&edited, EXE)), Err(Error::Signature));
        // a signature-shaped file that signs nothing, next to any exe
        let garbage = good.lines().map(|l| if l.starts_with("trusted comment:") { "trusted comment: version=9.9.9 file=triokey.exe" } else { l });
        let garbage = garbage.collect::<Vec<_>>().join("\n");
        assert_eq!(check(&pk, "0.1.0", &mut release(&garbage, b"MZ malware")), Err(Error::Signature));
        // correctly signed, but for another file name, or not newer
        let renamed = sign(&kp, EXE, "version=0.2.0 file=setup.exe");
        assert_eq!(check(&pk, "0.1.0", &mut release(&renamed, EXE)), Err(Error::Format));
        assert_eq!(check(&pk, "0.2.0", &mut release(&good, EXE)), Ok(None));
        // the claim was newer when the download started, the verified comment is not
        assert_eq!(verified(&pk, "0.2.0", &good, &mut release(&good, EXE)), Err(Error::Signature));
        // the download fails half-way
        let mut cut = |url: &str, _: usize, out: &mut dyn Write| {
            out.write_all(if url == SIG_URL { good.as_bytes() } else { b"MZ" }).unwrap();
            if url == SIG_URL {
                Ok(())
            } else {
                Err(Error::Net)
            }
        };
        assert_eq!(check(&pk, "0.1.0", &mut cut), Err(Error::Net));
    }
}
