//! Startup: settings, engine, threads, tray. Files are read and written only here, never on
//! the hook thread (spec 5.8).

use std::fs;
use std::io::{self, ErrorKind, Write};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Mutex, RwLock};

use triokey_engine::lexicon::Lexicon;
use triokey_engine::{Config, ToneStyle};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
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

pub fn config_unreadable() -> bool {
    CONFIG_UNREADABLE.load(Relaxed)
}

/// True once per failed write, for the settings window to report.
pub fn take_save_failed() -> bool {
    SAVE_FAILED.swap(false, Relaxed)
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
/// `write_config`, and never runs or opens anything in it.
pub fn config_dir() -> Option<PathBuf> {
    admin::roaming_appdata().map(|a| a.join("triokey"))
}

/// Write `config.txt`. Unelevated programs of the same user own this folder and can swap it
/// for a junction into an admin-only place, so an elevated TrioKey creates nothing: it only
/// rewrites an existing file in a plain existing folder. The file is opened without following
/// a link and without truncating, and checked on the handle before anything is written: a
/// regular file, no reparse point, and no second name (a hard link to another file).
/// shortcut: a swap of the folder between its check and the open is still possible; closing
/// that needs handle-relative opens. Upgrade if the elevated copy ever writes more than this
/// fixed text, or writes without the user changing a setting.
fn write_config(dir: &Path, text: &str, elevated: bool) -> io::Result<()> {
    let path = dir.join("config.txt");
    if !elevated {
        fs::create_dir_all(dir)?;
        return fs::write(path, text);
    }
    if !fs::symlink_metadata(dir).is_ok_and(|m| m.is_dir() && m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0) {
        return Err(ErrorKind::PermissionDenied.into());
    }
    let mut f = fs::OpenOptions::new().write(true).custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(path)?;
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(f.as_raw_handle(), &mut info) } == 0
        || info.dwFileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0
        || info.nNumberOfLinks != 1
    {
        return Err(ErrorKind::PermissionDenied.into());
    }
    f.set_len(0)?;
    f.write_all(text.as_bytes())
}

/// One-time carry-over from the working name (`%APPDATA%\evj`): copy the three data files when
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
/// An elevated copy keeps the time in memory only: it writes under the user-writable
/// `%APPDATA%` when the user changes a setting, never by itself on a timer.
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
    let lexicon = Lexicon::builtin().with_user(&read("force_vi.txt"), &read("force_en.txt"));
    ENABLED.store(s.enabled, Relaxed);
    *SETTINGS.write().unwrap_or_else(|e| e.into_inner()) = s;
    ime_watch::spawn();
    hook::spawn(lexicon);
    update::spawn();
    // returns on "Thoát"; leaving main ends the process, which removes both hooks
    tray::run();
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
        let root = std::env::temp_dir().join(format!("triokey-{name}-test-{}", std::process::id()));
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
}
