//! The "Chẩn đoán" text: what this Mac reports, for a person to paste into a bug report. Never holds typed text.

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
}

/// `text` with the `home` folder written as `~` wherever a path inside it starts there.
pub fn tilde(text: &str, home: Option<&str>) -> String {
    match home.map(|h| h.trim_end_matches('/')).filter(|h| !h.is_empty()) {
        Some(h) if text == h => "~".into(),
        Some(h) => text.replace(&format!("{h}/"), "~/"),
        None => text.into(),
    }
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
        let home = self.home.as_deref();
        let c = &self.counters;
        let mut t = String::new();
        let _ = writeln!(t, "TrioKey {} (macOS {}, {})", self.version, self.os, self.arch);
        let _ = writeln!(t, "bundle: {} (AppTranslocation: {})", tilde(&self.bundle, home), yes(self.bundle.contains("AppTranslocation")));
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
        let _ = writeln!(t, "last error: {}", self.last_error.as_deref().map_or("none".into(), |e| tilde(e, home)));
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_replaces_only_the_home_prefix() {
        assert_eq!(tilde("/Users/x/Downloads/TrioKey.app", Some("/Users/x")), "~/Downloads/TrioKey.app");
        assert_eq!(tilde("/Users/x/a", Some("/Users/x/")), "~/a");
        assert_eq!(tilde("/Users/x", Some("/Users/x")), "~");
        assert_eq!(tilde("/Users/xy/a", Some("/Users/x")), "/Users/xy/a");
        assert_eq!(tilde("/Applications/TrioKey.app", Some("/Users/x")), "/Applications/TrioKey.app");
        assert_eq!(tilde("/Users/x/a", None), "/Users/x/a");
        assert_eq!(tilde("/a", Some("")), "/a");
    }

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
            home: Some("/Users/x".into()),
            tap_created: true,
            tap_enabled: false,
            ax_trusted: false,
            secure: true,
            state: State::SecureInput,
            current: us.clone(),
            seen: vec![us, jp],
            counters: Counters { keys: 9, replaced: 2, chars: 5, backspaces: 3, reenabled_timeout: 1, reenabled_user: 0, panics: 0 },
            last_error: Some("panic at /Users/x/src/a.rs".into()),
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
";
        assert_eq!(d.render(), want);
        let none = Diagnostics { bundle: "/Users/x/Applications/TrioKey.app".into(), current: Source::default(), seen: vec![], last_error: None, ..d };
        let r = none.render();
        assert!(r.contains("bundle: ~/Applications/TrioKey.app (AppTranslocation: no)"));
        assert!(r.contains("current input source: id ?, type ?, mode ?, ASCII capable ? -> Yield"));
        assert!(r.ends_with("last error: none\n"));
    }
}
