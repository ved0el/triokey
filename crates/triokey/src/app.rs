//! Startup: settings, engine, threads, tray. User data files are read and written only here, never on
//! the hook thread (docs/ARCHITECTURE.md, 2.3 What may run inside the hook callback).

use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

use triokey_engine::lexicon::{Learned, Lexicon};
use triokey_engine::{Config, ToneStyle};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, GetFinalPathNameByHandleW, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE,
};

use crate::config::{parse, render, Settings};
use crate::{admin, hook, ime_watch, tray, update};

static SETTINGS: RwLock<Settings> = RwLock::new(Settings::DEFAULT);
static ENABLED: AtomicBool = AtomicBool::new(true);
/// Held from changing `SETTINGS` until config.txt is written: the tray thread and the updater
/// thread both save, and the file must end up with the later state. Never taken by the hook.
static SAVE: Mutex<()> = Mutex::new(());
/// config.txt exists but could not be read (locked, not UTF-8): it is never overwritten.
static CONFIG_UNREADABLE: AtomicBool = AtomicBool::new(false);

/// The last write of config.txt failed (an elevated copy with no file to rewrite, a read-only
/// file): the settings live in memory only.
static SAVE_FAILED: AtomicBool = AtomicBool::new(false);

const LEARNED_FILE: &str = "learned.txt";
/// learned.txt above this is not read, and never overwritten.
const LEARNED_CAP: u64 = 256 * 1024;
/// learned.txt is written once the remembered words have not changed for this long.
const LEARNED_STILL: Duration = Duration::from_secs(2);

/// Writes learned.txt for the remembered words.
struct Saver {
    learned: Learned,
    dir: PathBuf,
    /// Change count at the last write, the last one seen, and since when it is still.
    saved: u64,
    seen: u64,
    still_since: Instant,
}
/// None: no folder, or a learned.txt that must be left alone (not UTF-8, too big).
static SAVER: Mutex<Option<Saver>> = Mutex::new(None);
/// The remembered words the engine uses, and the entry counts of force_vi.txt and force_en.txt
/// as read at startup: counts for the debug information, which reads no file.
static USER_DATA: OnceLock<(Learned, usize, usize)> = OnceLock::new();

pub fn config_unreadable() -> bool {
    CONFIG_UNREADABLE.load(Relaxed)
}

/// True once per failed write, for the settings window to report.
pub fn take_save_failed() -> bool {
    SAVE_FAILED.swap(false, Relaxed)
}

/// (remembered words, force_vi.txt entries, force_en.txt entries): numbers only, never the words.
pub fn user_data_counts() -> (usize, usize, usize) {
    USER_DATA.get().map_or((0, 0, 0), |(l, vi, en)| (learned_count(l), *vi, *en))
}

/// The engine crate has no length accessor: the entry lines of the rendered file (the header lines start with `#`).
fn learned_count(l: &Learned) -> usize {
    l.render().lines().filter(|l| !l.starts_with('#')).count()
}

/// Entries of a force file as `Lexicon::with_user` reads them: the text before `#`, trimmed, not empty.
fn entries(text: &str) -> usize {
    text.lines().filter(|l| !l.split('#').next().unwrap_or("").trim().is_empty()).count()
}

pub fn settings() -> Settings {
    *SETTINGS.read().unwrap_or_else(|e| e.into_inner())
}

pub fn engine_config(s: &Settings) -> Config {
    let tone_style = if s.tone_new { ToneStyle::New } else { ToneStyle::Old };
    Config { tone_style, auto_english: s.auto_english }
}

pub fn enabled() -> bool {
    ENABLED.load(Relaxed)
}

/// Hotkey or tray click. Not written back to the config file.
pub fn toggle() {
    set_enabled(!enabled());
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Relaxed);
    ime_watch::break_word();
    tray::notify();
}

/// `%APPDATA%\triokey`. An elevated TrioKey reads data from it, writes only through
/// `write_file`, and never runs or opens anything in it.
pub fn config_dir() -> Option<PathBuf> {
    admin::roaming_appdata().map(|a| a.join("triokey"))
}

fn write_config(dir: &Path, text: &str, elevated: bool) -> io::Result<()> {
    write_file(dir, "config.txt", text, elevated)
}

/// Write `dir\name`; elevated, only through `open_checked`, in place (no truncating open).
fn write_file(dir: &Path, name: &str, text: &str, elevated: bool) -> io::Result<()> {
    if !elevated {
        fs::create_dir_all(dir)?;
        return fs::write(dir.join(name), text);
    }
    // the folder stays held through the write
    let (_folder, mut f) = open_checked(dir, name, true)?;
    f.set_len(0)?;
    f.write_all(text.as_bytes())
}

/// `dir\name` for an elevated TrioKey. Unelevated programs of the same user own this folder
/// and can swap it, the file, or a folder on the way for a link into an admin-only place, so
/// this opens only an existing plain file in a plain existing folder, checked on handles: the
/// folder is held open (no delete sharing, so it cannot be renamed or replaced meanwhile), is
/// no reparse point and really is `dir` (a junction or object link higher up the path ends
/// elsewhere); the file is opened without following a link or truncating, is a regular file
/// with one name, and its final path is `dir\name`. Returns (folder, file); refused:
/// PermissionDenied; missing: NotFound.
// shortcut: `dir` comes from the user's AppData setting (HKCU), which can name another real
// folder called triokey that has these files; only a per-machine data folder would close it.
fn open_checked(dir: &Path, name: &str, write: bool) -> io::Result<(fs::File, fs::File)> {
    let refused = || io::Error::from(ErrorKind::PermissionDenied);
    let folder = hold_dir(dir)?;
    let f = fs::OpenOptions::new().read(!write).write(write).custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(dir.join(name))?;
    let info = file_info(&f).ok_or_else(refused)?;
    if info.dwFileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0 || info.nNumberOfLinks != 1 {
        return Err(refused());
    }
    let (Some(d), Some(p), Some(asked)) = (final_path(&folder), final_path(&f), dir.to_str()) else { return Err(refused()) };
    let is_dir = d.strip_prefix(r"\\?\").is_some_and(|d| d.eq_ignore_ascii_case(asked.trim_end_matches('\\')));
    if !is_dir || !p.strip_prefix(&d).and_then(|r| r.strip_prefix('\\')).is_some_and(|n| n.eq_ignore_ascii_case(name)) {
        return Err(refused());
    }
    Ok((folder, f))
}

/// `dir` itself (not what a link points to), open with read and write sharing but not delete,
/// so nobody can rename or replace it while the handle lives; refused unless a plain folder.
fn hold_dir(dir: &Path) -> io::Result<fs::File> {
    let h = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(dir)?;
    match file_info(&h) {
        Some(i) if i.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 && i.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT == 0 => Ok(h),
        _ => Err(ErrorKind::PermissionDenied.into()),
    }
}

fn file_info(f: &fs::File) -> Option<BY_HANDLE_FILE_INFORMATION> {
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    (unsafe { GetFileInformationByHandle(f.as_raw_handle(), &mut info) } != 0).then_some(info)
}

/// Where the handle really points, as `\\?\C:\...`.
fn final_path(f: &fs::File) -> Option<String> {
    let mut buf = vec![0u16; 1024];
    let n = unsafe { GetFinalPathNameByHandleW(f.as_raw_handle(), buf.as_mut_ptr(), buf.len() as u32, 0) } as usize;
    (n > 0 && n < buf.len()).then(|| String::from_utf16_lossy(&buf[..n]))
}

/// One-time carry-over from the older folder name (`%APPDATA%\evj`): copy the three data files when
/// `new` does not exist yet; `old` is left in place. `run_as_admin` is reset because the old
/// task and Program Files copy carry the old name, so no TrioKey task stands behind it.
fn migrate(old: &Path, new: &Path) {
    // create_dir fails when `new` exists, which is the "only once" check
    if !old.is_dir() || fs::create_dir(new).is_err() {
        return;
    }
    for name in ["config.txt", "force_vi.txt", "force_en.txt"] {
        // regular files only: a link must not pull in a file from outside the old folder
        if !fs::symlink_metadata(old.join(name)).is_ok_and(|m| m.is_file()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(old.join(name)) else { continue };
        let text = if name == "config.txt" { render(&Settings { run_as_admin: false, ..parse(&text) }) } else { text };
        let _ = fs::write(new.join(name), text);
    }
}

fn store(f: impl FnOnce(&mut Settings)) -> Settings {
    let mut g = SETTINGS.write().unwrap_or_else(|e| e.into_inner());
    f(&mut g);
    *g
}

/// Updater thread: remember when GitHub last answered. Nothing is applied live, and a failed
/// write is not reported: the only cost is one more check after the next start.
/// An elevated copy keeps the time in memory only: it rewrites config.txt only when the user
/// changes a setting.
pub fn set_last_update_check(t: u64) {
    let _saving = SAVE.lock().unwrap_or_else(|e| e.into_inner());
    let s = store(|s| s.last_update_check = t);
    if let (Some(dir), false, false) = (config_dir(), config_unreadable(), admin::is_elevated()) {
        let _ = write_config(&dir, &render(&s), false);
    }
}

/// Settings window (tray thread): store, write `config.txt`, apply live.
pub fn update(f: impl FnOnce(&mut Settings)) {
    let saving = SAVE.lock().unwrap_or_else(|e| e.into_inner());
    let s = store(f);
    // unreadable file: the change applies to this run only (the settings window says so)
    if let (Some(dir), false) = (config_dir(), config_unreadable()) {
        SAVE_FAILED.store(write_config(&dir, &render(&s), admin::is_elevated()).is_err(), Relaxed);
    }
    drop(saving);
    hook::reconfigure();
    // yield_cjk is read by the watcher on its next pass
    ime_watch::request_recheck();
    tray::notify();
}

pub fn run() {
    // two copies would both rewrite every word
    if tray::already_running() {
        return;
    }
    let (dir, elevated) = (config_dir(), admin::is_elevated());
    // before load_settings, which creates the folder. Never elevated: the old folder is
    // user-writable, and an admin copy only exists after a normal run has already migrated.
    if let (Some(new), false) = (&dir, elevated) {
        migrate(&new.with_file_name("evj"), new);
    }
    let (s, readable) = load_settings(dir.as_deref(), elevated);
    CONFIG_UNREADABLE.store(!readable, Relaxed);
    // no file after a successful load: it was missing and could not be created
    SAVE_FAILED.store(readable && !dir.as_ref().is_some_and(|d| d.join("config.txt").is_file()), Relaxed);
    let read = |name: &str| dir.as_ref().and_then(|d| fs::read_to_string(d.join(name)).ok()).unwrap_or_default();
    let (learned, saver) = load_learned(dir.as_deref(), elevated);
    let (vi, en) = (read("force_vi.txt"), read("force_en.txt"));
    let _ = USER_DATA.set((learned.clone(), entries(&vi), entries(&en)));
    let lexicon = Lexicon::builtin().with_user(&vi, &en).with_learned(learned);
    *SAVER.lock().unwrap_or_else(|e| e.into_inner()) = saver;
    ENABLED.store(s.enabled, Relaxed);
    *SETTINGS.write().unwrap_or_else(|e| e.into_inner()) = s;
    ime_watch::spawn();
    hook::spawn(lexicon);
    update::spawn();
    // returns on "Thoát"; leaving main ends the process, which removes both hooks
    tray::run();
    save_learned(true);
}

/// learned.txt, read after `load_settings` (migrate needs the folder not to exist before).
/// Missing: created with the header only, when not elevated. Unusable (not UTF-8, over the
/// cap, locked, refused by `open_checked`): the words live in memory only and the file is never
/// written. Elevated, the read is checked too: a link must not pull admin-only text into the
/// map, which the next save would write where the user can read it.
fn load_learned(dir: Option<&Path>, elevated: bool) -> (Learned, Option<Saver>) {
    let Some(dir) = dir else { return (Learned::default(), None) };
    let mut buf = Vec::new();
    let file = if elevated { open_checked(dir, LEARNED_FILE, false).map(|(_, f)| f) } else { fs::File::open(dir.join(LEARNED_FILE)) };
    let learned = match file.and_then(|f| f.take(LEARNED_CAP + 1).read_to_end(&mut buf)) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let learned = Learned::default();
            if !elevated {
                let _ = write_file(dir, LEARNED_FILE, &learned.render(), false);
            }
            learned
        }
        Ok(n) if n as u64 <= LEARNED_CAP => match String::from_utf8(buf) {
            Ok(text) => Learned::parse(&text),
            Err(_) => return (Learned::default(), None),
        },
        _ => return (Learned::default(), None),
    };
    let saver = Saver { learned: learned.clone(), dir: dir.to_path_buf(), saved: 0, seen: 0, still_since: Instant::now() };
    (learned, Some(saver))
}

/// Watcher tick, and once at exit (`now`): write learned.txt when the remembered words changed
/// and have been still for `LEARNED_STILL`. A failed write (an elevated copy refusing the
/// folder or file) is not retried until the next change.
pub fn save_learned(now: bool) {
    let mut g = SAVER.lock().unwrap_or_else(|e| e.into_inner());
    let Some(s) = g.as_mut() else { return };
    let changes = s.learned.changes();
    if changes != s.seen {
        (s.seen, s.still_since) = (changes, Instant::now());
    }
    if changes != s.saved && (now || s.still_since.elapsed() >= LEARNED_STILL) {
        s.saved = changes;
        let _ = write_file(&s.dir, LEARNED_FILE, &s.learned.render(), admin::is_elevated());
    }
}

/// Settings window, right before the normal copy starts the elevated installer: an elevated
/// copy creates no file, so learned.txt must already exist when it runs.
pub fn create_learned_file() {
    let g = SAVER.lock().unwrap_or_else(|e| e.into_inner());
    if let (Some(s), false) = (g.as_ref(), admin::is_elevated()) {
        if !s.dir.join(LEARNED_FILE).exists() {
            let _ = write_file(&s.dir, LEARNED_FILE, &s.learned.render(), false);
        }
    }
}

/// `%APPDATA%\triokey\config.txt`, created with defaults only when it does not exist yet.
/// The flag is false when the file exists but cannot be read (locked, UTF-16): defaults apply
/// and the caller must keep every later write away from the user's file.
fn load_settings(dir: Option<&Path>, elevated: bool) -> (Settings, bool) {
    let s = Settings::default();
    let Some(dir) = dir else { return (s, true) };
    match fs::read_to_string(dir.join("config.txt")) {
        Ok(text) => (parse(&text), true),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let _ = write_config(dir, &render(&s), elevated);
            (s, true)
        }
        Err(_) => (s, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        // the long, real path: open_checked compares the folder's final path with it (TEMP may be 8.3)
        let base = fs::canonicalize(std::env::temp_dir()).unwrap();
        let base = PathBuf::from(base.to_str().unwrap().trim_start_matches(r"\\?\"));
        let root = base.join(format!("triokey-{name}-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn elevated_write_creates_nothing_and_follows_no_link() {
        let root = temp("write");
        let (dir, cfg) = (root.join("triokey"), root.join("triokey").join("config.txt"));
        // no folder, then no file: nothing is created
        assert!(write_config(&dir, "a", true).is_err());
        assert!(!dir.exists());
        fs::create_dir_all(&dir).unwrap();
        assert!(write_config(&dir, "a", true).is_err());
        assert!(!cfg.exists());
        // unelevated creates; elevated then rewrites the plain file (shorter text: truncated)
        write_config(&dir, "first text", false).unwrap();
        write_config(&dir, "second", true).unwrap();
        assert_eq!(fs::read_to_string(&cfg).unwrap(), "second");
        // a folder with config.txt as a directory is not a file to write
        let odd = root.join("odd");
        fs::create_dir_all(odd.join("config.txt")).unwrap();
        assert!(write_config(&odd, "x", true).is_err());
        // The attack: the folder swapped for a junction (needs no privilege) into another
        // folder that has a config.txt. Refused, and the target stays as it was.
        let junction = root.join("junction");
        let made = std::process::Command::new("cmd").args(["/c", "mklink", "/J"]).args([&junction, &dir]).output();
        assert!(made.is_ok_and(|o| o.status.success()) && junction.join("config.txt").is_file());
        assert!(write_config(&junction, "through the link", true).is_err());
        // symbolic links need a privilege or developer mode; refused where they can be made
        let (link_dir, link_file) = (root.join("link"), root.join("filelink"));
        if std::os::windows::fs::symlink_dir(&dir, &link_dir).is_ok() {
            assert!(write_config(&link_dir, "through the link", true).is_err());
        }
        fs::create_dir_all(&link_file).unwrap();
        if std::os::windows::fs::symlink_file(&cfg, link_file.join("config.txt")).is_ok() {
            assert!(write_config(&link_file, "through the link", true).is_err());
        }
        assert_eq!(fs::read_to_string(&cfg).unwrap(), "second");
        // a hard link is a plain file by its attributes: config.txt as a second name of a
        // file elsewhere is refused, and so is the real config.txt once it has a second name
        let (victim, hard) = (root.join("victim.txt"), root.join("hard"));
        fs::write(&victim, "not ours").unwrap();
        fs::create_dir_all(&hard).unwrap();
        fs::hard_link(&victim, hard.join("config.txt")).unwrap();
        assert!(write_config(&hard, "through the link", true).is_err());
        assert_eq!(fs::read_to_string(&victim).unwrap(), "not ours");
        fs::hard_link(&cfg, root.join("second-name.txt")).unwrap();
        assert!(write_config(&dir, "third", true).is_err());
        assert_eq!(fs::read_to_string(&cfg).unwrap(), "second");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn elevated_write_holds_the_folder_and_writes_learned() {
        let root = temp("hold");
        let (dir, file) = (root.join("triokey"), root.join("triokey").join(LEARNED_FILE));
        fs::create_dir_all(&dir).unwrap();
        assert!(write_file(&dir, LEARNED_FILE, "x", true).is_err());
        assert!(!file.exists());
        write_file(&dir, LEARNED_FILE, "first text", false).unwrap();
        write_file(&dir, "config.txt", "c", false).unwrap();
        write_file(&dir, LEARNED_FILE, "second", true).unwrap();
        write_file(&dir, "config.txt", "d", true).unwrap();
        assert_eq!((fs::read_to_string(&file).unwrap(), fs::read_to_string(dir.join("config.txt")).unwrap()), ("second".into(), "d".into()));
        // while held, the folder cannot be renamed away (and so not replaced); afterwards it can
        let held = hold_dir(&dir).unwrap();
        assert!(fs::rename(&dir, root.join("moved")).is_err());
        drop(held);
        fs::rename(&dir, root.join("moved")).unwrap();
        // a junction as the folder is refused, file name and all
        let junction = root.join("junction");
        let made = std::process::Command::new("cmd").args(["/c", "mklink", "/J"]).args([&junction, &root.join("moved")]).output();
        assert!(made.is_ok_and(|o| o.status.success()) && junction.join(LEARNED_FILE).is_file());
        assert!(hold_dir(&junction).is_err() && write_file(&junction, LEARNED_FILE, "through the link", true).is_err());
        // a junction higher up the path: the folder itself is plain, but not the one asked for;
        // neither written nor read (its text would be saved where the user reads it)
        let up = root.join("up");
        let made = std::process::Command::new("cmd").args(["/c", "mklink", "/J"]).args([&up, &root]).output();
        assert!(made.is_ok_and(|o| o.status.success()) && up.join("moved").join(LEARNED_FILE).is_file());
        assert!(write_file(&up.join("moved"), LEARNED_FILE, "through the link", true).is_err());
        assert!(load_learned(Some(&up.join("moved")), true).1.is_none());
        // the real path is fine, with a trailing backslash and in other case
        let upper = PathBuf::from(format!("{}\\", root.join("moved").to_str().unwrap().to_uppercase()));
        assert!(load_learned(Some(&upper), true).1.is_some());
        let hard = root.join("hard");
        fs::create_dir_all(&hard).unwrap();
        fs::hard_link(root.join("moved").join(LEARNED_FILE), hard.join(LEARNED_FILE)).unwrap();
        assert!(write_file(&hard, LEARNED_FILE, "through the link", true).is_err());
        assert_eq!(fs::read_to_string(root.join("moved").join(LEARNED_FILE)).unwrap(), "second");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn learned_file_is_created_read_capped_and_left_alone() {
        let root = temp("learned");
        let file = root.join(LEARNED_FILE);
        // missing and elevated: nothing created, but remembered in memory and saved later
        fs::create_dir_all(&root).unwrap();
        assert!(load_learned(Some(&root), true).1.is_some() && !file.exists());
        // missing: header only
        let (l, saver) = load_learned(Some(&root), false);
        assert!(saver.is_some() && fs::read_to_string(&file).unwrap() == l.render() && l.render().lines().all(|x| x.starts_with('#')));
        // round trip through the file
        l.set("this", Some("thí"));
        write_file(&root, LEARNED_FILE, &l.render(), true).unwrap();
        let (back, _) = load_learned(Some(&root), true);
        assert_eq!((back.get("this").as_deref(), back.render()), (Some("thí"), l.render()));
        // over the cap or not UTF-8: nothing read, no saver, bytes untouched
        let big = format!("this\tthí\n{}", "#".repeat(LEARNED_CAP as usize));
        fs::write(&file, &big).unwrap();
        let (l, saver) = load_learned(Some(&root), false);
        assert!(saver.is_none() && l.get("this").is_none());
        let utf16: Vec<u8> = "\u{FEFF}this\tthí\n".encode_utf16().flat_map(u16::to_le_bytes).collect();
        fs::write(&file, &utf16).unwrap();
        assert!(load_learned(Some(&root), false).1.is_none() && fs::read(&file).unwrap() == utf16);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn unreadable_config_is_reported_and_left_alone() {
        let root = temp("load");
        // missing: created with defaults (unelevated), readable
        assert_eq!(load_settings(Some(&root), false), (Settings::default(), true));
        assert_eq!(fs::read_to_string(root.join("config.txt")).unwrap(), render(&Settings::default()));
        fs::write(root.join("config.txt"), "tone_new = true\n").unwrap();
        assert_eq!(load_settings(Some(&root), false), (Settings { tone_new: true, ..Settings::default() }, true));
        // saved as UTF-16 in Notepad: defaults, flagged, bytes untouched
        let utf16: Vec<u8> = "\u{FEFF}tone_new = true\n".encode_utf16().flat_map(u16::to_le_bytes).collect();
        fs::write(root.join("config.txt"), &utf16).unwrap();
        assert_eq!(load_settings(Some(&root), false), (Settings::default(), false));
        assert_eq!(fs::read(root.join("config.txt")).unwrap(), utf16);
        // missing while elevated: defaults, and the file is not created
        let empty = root.join("empty");
        fs::create_dir(&empty).unwrap();
        assert_eq!(load_settings(Some(&empty), true), (Settings::default(), true));
        assert!(!empty.join("config.txt").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn migrate_once() {
        let root = temp("migrate");
        let (old, new) = (root.join("evj"), root.join("triokey"));
        // no old folder: nothing is created
        migrate(&old, &new);
        assert!(!new.exists());

        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("config.txt"), "tone_new = true\nrun_as_admin = true\n").unwrap();
        fs::write(old.join("force_vi.txt"), "has\n").unwrap();
        fs::write(old.join("other.txt"), "x").unwrap();
        fs::create_dir(old.join("force_en.txt")).unwrap();
        migrate(&old, &new);
        let s = parse(&fs::read_to_string(new.join("config.txt")).unwrap());
        assert!(s.tone_new && !s.run_as_admin);
        assert_eq!(fs::read_to_string(new.join("force_vi.txt")).unwrap(), "has\n");
        // only the three names, only regular files; the old folder stays
        assert!(!new.join("other.txt").exists() && !new.join("force_en.txt").exists());
        assert!(old.join("config.txt").exists());

        // new folder exists: never overwritten
        fs::write(old.join("force_vi.txt"), "changed\n").unwrap();
        migrate(&old, &new);
        assert_eq!(fs::read_to_string(new.join("force_vi.txt")).unwrap(), "has\n");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn force_file_entries_are_counted_as_the_lexicon_reads_them() {
        assert_eq!(entries("# comment\nthis\n\n  that  # note\n   \n#x\nmore"), 3);
        let learned = Learned::parse("# header\nthis\tthí\nwas\twá\n");
        assert_eq!((learned_count(&learned), learned_count(&Learned::default())), (2, 0));
    }
}
