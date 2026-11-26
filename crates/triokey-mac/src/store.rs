//! The files under `~/Library/Application Support/triokey` (docs/ARCHITECTURE.md, 12.13 Settings, files and the window):
//! config.txt in the Windows format, learned.txt, force_vi.txt and force_en.txt. Plain `std::fs`, so it is tested on every
//! system; the app calls it on the main thread, never from the tap callback.

use std::fs;
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use triokey_engine::lexicon::Learned;
use triokey_shared::config::{parse, render, Settings};

pub const CONFIG: &str = "config.txt";
pub const LEARNED: &str = "learned.txt";
pub const FORCE_VI: &str = "force_vi.txt";
pub const FORCE_EN: &str = "force_en.txt";
/// learned.txt above this is not read, and never overwritten (as on Windows).
pub const LEARNED_CAP: u64 = 256 * 1024;
/// learned.txt is written once the remembered words have not changed for this long.
pub const LEARNED_STILL: Duration = Duration::from_secs(2);

/// (data folder, login item file) under the home folder `home`.
pub fn paths(home: &Path) -> (PathBuf, PathBuf) {
    let lib = home.join("Library");
    (lib.join("Application Support").join("triokey"), lib.join("LaunchAgents").join(crate::login::FILE))
}

/// Writes `dir/name` through a temporary file in the same folder renamed over it, so a failed write leaves the old file.
pub fn write(dir: &Path, name: &str, text: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(".{name}.tmp"));
    fs::write(&tmp, text).and_then(|()| fs::rename(&tmp, dir.join(name))).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// config.txt, created with defaults when missing. `false`: it exists but cannot be read (not UTF-8): defaults apply
/// and the file must never be written.
pub fn load_config(dir: &Path) -> (Settings, bool) {
    match fs::read_to_string(dir.join(CONFIG)) {
        Ok(text) => (parse(&text), true),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let _ = write(dir, CONFIG, &render(&Settings::default()));
            (Settings::default(), true)
        }
        Err(_) => (Settings::default(), false),
    }
}

/// force_vi.txt and force_en.txt, empty when missing or unreadable.
pub fn load_force(dir: &Path) -> (String, String) {
    let read = |n| fs::read_to_string(dir.join(n)).unwrap_or_default();
    (read(FORCE_VI), read(FORCE_EN))
}

/// Entries of a force file as `Lexicon::with_user` reads them: the text before `#`, trimmed, not empty.
pub fn entries(text: &str) -> usize {
    text.lines().filter(|l| !l.split('#').next().unwrap_or("").trim().is_empty()).count()
}

/// Remembered words, counted from the rendered file (the engine has no length accessor; header lines start with `#`).
pub fn learned_count(l: &Learned) -> usize {
    l.render().lines().filter(|l| !l.starts_with('#')).count()
}

/// Writes learned.txt for the remembered words.
pub struct Saver {
    learned: Learned,
    dir: PathBuf,
    /// Change count at the last write, the last one seen, and since when it is still.
    saved: u64,
    seen: u64,
    still_since: Instant,
}

/// learned.txt. Missing: created with the header only. Unusable (not UTF-8, over the cap, unreadable): the words live
/// in memory only and the file is never written (no saver).
pub fn load_learned(dir: &Path) -> (Learned, Option<Saver>) {
    let mut buf = Vec::new();
    let learned = match fs::File::open(dir.join(LEARNED)).and_then(|f| f.take(LEARNED_CAP + 1).read_to_end(&mut buf)) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let learned = Learned::default();
            let _ = write(dir, LEARNED, &learned.render());
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

impl Saver {
    /// The 2-second timer, and once at quit (`now`): write learned.txt when the remembered words changed and have been
    /// still for `LEARNED_STILL`. A failed write is not retried until the next change.
    pub fn tick(&mut self, now: bool) {
        let changes = self.learned.changes();
        if changes != self.seen {
            (self.seen, self.still_since) = (changes, Instant::now());
        }
        if changes != self.saved && (now || self.still_since.elapsed() >= LEARNED_STILL) {
            self.saved = changes;
            let _ = write(&self.dir, LEARNED, &self.learned.render());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("triokey-mac-{name}-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn config_is_created_read_and_an_unreadable_one_left_alone() {
        let dir = temp("config").join("triokey");
        assert_eq!(load_config(&dir), (Settings::default(), true));
        assert_eq!(fs::read_to_string(dir.join(CONFIG)).unwrap(), render(&Settings::default()));
        write(&dir, CONFIG, "tone_new = true\nrun_as_admin = true\n").unwrap();
        let (s, ok) = load_config(&dir);
        assert!(ok && s.tone_new && s.run_as_admin);
        // run_as_admin means nothing on macOS but is written back as read
        assert!(render(&s).contains("run_as_admin = true\n"));
        let utf16: Vec<u8> = "\u{FEFF}tone_new = true\n".encode_utf16().flat_map(u16::to_le_bytes).collect();
        fs::write(dir.join(CONFIG), &utf16).unwrap();
        assert_eq!(load_config(&dir), (Settings::default(), false));
        assert_eq!(fs::read(dir.join(CONFIG)).unwrap(), utf16);
        // the temporary file does not stay behind
        assert!(fs::read_dir(&dir).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
        fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn learned_is_created_saved_when_still_and_capped() {
        let dir = temp("learned");
        let (l, saver) = load_learned(&dir);
        let mut saver = saver.unwrap();
        assert_eq!(fs::read_to_string(dir.join(LEARNED)).unwrap(), l.render());
        assert_eq!(learned_count(&l), 0);
        l.set("this", Some("thí"));
        saver.tick(false);
        assert_eq!(learned_count(&Learned::parse(&fs::read_to_string(dir.join(LEARNED)).unwrap())), 0, "not still yet");
        saver.tick(true);
        let back = load_learned(&dir).0;
        assert_eq!((back.get("this").as_deref(), learned_count(&back)), (Some("thí"), 1));
        // over the cap or not UTF-8: nothing read, no saver, bytes untouched
        let big = format!("this\tthí\n{}", "#".repeat(LEARNED_CAP as usize));
        fs::write(dir.join(LEARNED), &big).unwrap();
        let (l, saver) = load_learned(&dir);
        assert!(saver.is_none() && l.get("this").is_none());
        fs::write(dir.join(LEARNED), [0xff, 0xfe, 0x41]).unwrap();
        assert!(load_learned(&dir).1.is_none() && fs::read(dir.join(LEARNED)).unwrap() == [0xff, 0xfe, 0x41]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn force_files_and_paths() {
        let dir = temp("force");
        assert_eq!(load_force(&dir), (String::new(), String::new()));
        write(&dir, FORCE_EN, "# comment\nthis\n\n  that  # note\n").unwrap();
        let (vi, en) = load_force(&dir);
        assert_eq!((entries(&vi), entries(&en)), (0, 2));
        fs::remove_dir_all(&dir).unwrap();
        let (data, agent) = paths(Path::new("/Users/x"));
        assert_eq!(data, Path::new("/Users/x/Library/Application Support/triokey"));
        assert_eq!(agent, Path::new("/Users/x/Library/LaunchAgents/io.github.ved0el.triokey.plist"));
    }
}
