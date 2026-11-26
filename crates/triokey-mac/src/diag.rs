//! The debug information ("Chép thông tin debug"): what this Mac reports, for a person to paste into a bug report
//! (docs/ARCHITECTURE.md, 12.7 Diagnostics). Never holds typed text, a remembered word or the user's name.

use std::fmt::Write;

use crate::source::{Decision, Source};
use crate::text::{tooltip, State};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub keys: u64,
    pub replaced: u64,
    pub chars: u64,
    pub backspaces: u64,
    pub reenabled_timeout: u64,
    pub reenabled_user: u64,
    pub panics: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostics {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub bundle: String,
    /// `$HOME`, shown as `~` in every path.
    pub home: Option<String>,
    pub tap_created: bool,
    pub tap_enabled: bool,
    pub ax_trusted: bool,
    pub secure: bool,
    pub state: State,
    pub current: Source,
    pub seen: Vec<Source>,
    pub counters: Counters,
    pub last_error: Option<String>,
    /// config.txt as it would be written (`config::render`), and its folder.
    pub config: String,
    pub config_dir: Option<String>,
    pub config_readable: bool,
    /// Remembered words, force_vi.txt entries, force_en.txt entries: numbers only.
    pub counts: (usize, usize, usize),
    /// `login::state`.
    pub login: String,
    pub update: String,
    pub waiting: Option<String>,
}

fn yes(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

fn source_line(s: &Source) -> String {
    let opt = |v: &Option<String>| v.clone().unwrap_or_else(|| "?".into());
    let ascii = s.ascii.map_or("?", yes);
    let d = if s.decision() == Decision::Type { "Type" } else { "Yield" };
    format!("id {}, type {}, mode {}, ASCII capable {ascii} -> {d}", opt(&s.id), opt(&s.kind), opt(&s.mode))
}

impl Diagnostics {
    pub fn render(&self) -> String {
        let c = &self.counters;
        let mut t = String::new();
        let _ = writeln!(t, "TrioKey {} (macOS {}, {})", self.version, self.os, self.arch);
        let _ = writeln!(t, "bundle: {} (AppTranslocation: {})", self.bundle, yes(self.bundle.contains("AppTranslocation")));
        let _ = writeln!(t, "tap: created {}, enabled {}; AXIsProcessTrusted: {}", yes(self.tap_created), yes(self.tap_enabled), yes(self.ax_trusted));
        let _ = writeln!(t, "Secure Input: {}", if self.secure { "on" } else { "off" });
        let _ = writeln!(t, "state: {}", tooltip(self.state));
        let _ = writeln!(t, "current input source: {}", source_line(&self.current));
        let _ = writeln!(t, "input sources seen since launch: {}", self.seen.len());
        for s in &self.seen {
            let _ = writeln!(t, "  {}", source_line(s));
        }
        let _ = writeln!(
            t,
            "counters: key events {}, replaced {}, characters posted {}, backspaces posted {}, re-enabled after timeout {}, re-enabled after user input {}, panics caught {}",
            c.keys, c.replaced, c.chars, c.backspaces, c.reenabled_timeout, c.reenabled_user, c.panics
        );
        let _ = writeln!(t, "last error: {}", self.last_error.as_deref().unwrap_or("none"));
        let settings = self.config.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("; ");
        let _ = writeln!(t, "config folder: {}; config.txt readable {}", self.config_dir.as_deref().unwrap_or("unknown"), yes(self.config_readable));
        let _ = writeln!(t, "settings: {settings}");
        let (learned, vi, en) = self.counts;
        let _ = writeln!(t, "remembered words: {learned}; force_vi.txt: {vi} entries; force_en.txt: {en} entries");
        let _ = writeln!(t, "start at login: {}", self.login);
        let _ = writeln!(
            t,
            "update: {}; waiting version {}",
            if self.update.is_empty() { "(no status)" } else { &self.update },
            self.waiting.as_deref().unwrap_or("none")
        );
        // the home folder names the user: masked over the whole text, case-insensitively
        match self.home.as_deref() {
            Some(h) => triokey_shared::mask_home(&t, h),
            None => t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        let us = Source { id: Some("com.apple.keylayout.US".into()), kind: Some("TISTypeKeyboardLayout".into()), mode: None, ascii: Some(true) };
        let jp = Source {
            id: Some("com.apple.inputmethod.Kotoeri".into()),
            kind: Some("TISTypeKeyboardInputMode".into()),
            mode: Some("com.apple.inputmethod.Japanese".into()),
            ascii: Some(false),
        };
        let d = Diagnostics {
            version: "0.3.0".into(),
            os: "Version 15.1 (Build 24B83)".into(),
            arch: "aarch64".into(),
            bundle: "/private/var/folders/x/AppTranslocation/ABC/d/TrioKey.app".into(),
            home: Some("/Users/example-user".into()),
            tap_created: true,
            tap_enabled: false,
            ax_trusted: false,
            secure: true,
            state: State::SecureInput,
            current: us.clone(),
            seen: vec![us, jp],
            counters: Counters { keys: 9, replaced: 2, chars: 5, backspaces: 3, reenabled_timeout: 1, reenabled_user: 0, panics: 0 },
            last_error: Some("panic at /Users/example-user/src/a.rs".into()),
            config: "enabled = true
hotkey = ctrl_shift
"
            .into(),
            config_dir: Some("/Users/Example-User/Library/Application Support/triokey".into()),
            config_readable: true,
            counts: (3, 1, 0),
            login: "on".into(),
            update: "Có bản mới v0.3.1.".into(),
            waiting: Some("0.3.1".into()),
        };
        let want = "TrioKey 0.3.0 (macOS Version 15.1 (Build 24B83), aarch64)
bundle: /private/var/folders/x/AppTranslocation/ABC/d/TrioKey.app (AppTranslocation: yes)
tap: created yes, enabled no; AXIsProcessTrusted: no
Secure Input: on
state: TrioKey: bàn phím đang bị Secure Input chặn
current input source: id com.apple.keylayout.US, type TISTypeKeyboardLayout, mode ?, ASCII capable yes -> Type
input sources seen since launch: 2
  id com.apple.keylayout.US, type TISTypeKeyboardLayout, mode ?, ASCII capable yes -> Type
  id com.apple.inputmethod.Kotoeri, type TISTypeKeyboardInputMode, mode com.apple.inputmethod.Japanese, ASCII capable no -> Yield
counters: key events 9, replaced 2, characters posted 5, backspaces posted 3, re-enabled after timeout 1, re-enabled after user input 0, panics caught 0
last error: panic at ~/src/a.rs
config folder: ~/Library/Application Support/triokey; config.txt readable yes
settings: enabled = true; hotkey = ctrl_shift
remembered words: 3; force_vi.txt: 1 entries; force_en.txt: 0 entries
start at login: on
update: Có bản mới v0.3.1.; waiting version 0.3.1
";
        assert_eq!(d.render(), want);
        let none = Diagnostics {
            bundle: "/Users/example-user/Applications/TrioKey.app".into(),
            current: Source::default(),
            seen: vec![],
            last_error: None,
            config_dir: None,
            update: String::new(),
            waiting: None,
            ..d
        };
        let r = none.render();
        assert!(r.contains("bundle: ~/Applications/TrioKey.app (AppTranslocation: no)"));
        assert!(r.contains("current input source: id ?, type ?, mode ?, ASCII capable ? -> Yield"));
        assert!(r.contains(
            "last error: none
config folder: unknown;"
        ));
        assert!(r.ends_with(
            "update: (no status); waiting version none
"
        ));
        assert!(!r.to_lowercase().contains("example-user"));
    }
}
