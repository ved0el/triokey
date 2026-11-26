//! What the Windows and the macOS app share and no OS call is needed for: the settings format
//! (`config`), the state glyphs (`glyph`), the interface language (`t`), the addresses both
//! windows show, and masking the home folder in the debug information both apps copy to the
//! clipboard.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

pub mod config;
pub mod glyph;

/// The crates' `repository` field, and the page of the latest release.
pub const REPOSITORY: &str = "https://github.com/ved0el/triokey";
pub const RELEASES: &str = "https://github.com/ved0el/triokey/releases/latest";

/// The footer line, `TrioKey v<version>`; `version` is the app crate's own.
pub fn footer(version: &str) -> String {
    format!("TrioKey v{version}")
}

/// The interface language (`Settings::english`), set by the app when it reads or changes the settings. Never set in
/// tests: they run in parallel and expect Vietnamese; test `pick` instead.
static ENGLISH: AtomicBool = AtomicBool::new(false);

pub fn set_english(on: bool) {
    ENGLISH.store(on, Relaxed);
}

pub fn english() -> bool {
    ENGLISH.load(Relaxed)
}

/// `vi` or `en` for the current interface language: every text a person sees goes through this (or `english()` for a
/// formatted one), so switching the language takes effect at the next redraw.
pub fn t(vi: &'static str, en: &'static str) -> &'static str {
    pick(english(), vi, en)
}

pub fn pick<T>(english: bool, vi: T, en: T) -> T {
    if english {
        en
    } else {
        vi
    }
}

/// `text` with every path inside `home` (the user's profile or home folder) starting with `~`
/// instead, so a copied path does not name the user. Characters compare case-insensitively and
/// `\` equals `/` (Windows paths; macOS home folders are case-insensitive too); a match counts
/// only when no letter, digit, `_` or `-` follows, so `C:\Users\abc` leaves `C:\Users\abcd`
/// alone. An empty or root `home` changes nothing.
pub fn mask_home(text: &str, home: &str) -> String {
    let sep = |c: char| c == '\\' || c == '/';
    let home: Vec<char> = home.trim_end_matches(sep).chars().collect();
    if home.is_empty() || home.last() == Some(&':') {
        return text.to_string();
    }
    let same = |a: char, b: char| (sep(a) && sep(b)) || a.to_lowercase().eq(b.to_lowercase());
    let t: Vec<char> = text.chars().collect();
    let (mut out, mut i) = (String::with_capacity(text.len()), 0);
    while i < t.len() {
        let end = i + home.len();
        // `abc` must not match `abcd`; anything else after it ends the name (over-masking is harmless)
        let ends_name = !matches!(t.get(end), Some(&c) if c.is_alphanumeric() || c == '_' || c == '-');
        if end <= t.len() && t[i..end].iter().zip(&home).all(|(&a, &b)| same(a, b)) && ends_name {
            out.push('~');
            i = end;
        } else {
            out.push(t[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_is_masked_in_both_slash_directions_and_any_case() {
        let home = r"C:\Users\example-user";
        assert_eq!(mask_home(r"exe: C:\Users\example-user\AppData\Local\triokey.exe", home), r"exe: ~\AppData\Local\triokey.exe");
        assert_eq!(mask_home(r"c:/users/EXAMPLE-USER/AppData/Roaming/triokey", home), "~/AppData/Roaming/triokey");
        assert_eq!(mask_home(r"a C:\Users\example-user; b C:\Users\Example-User\x", r"C:\Users\example-user\"), r"a ~; b ~\x");
        assert_eq!(mask_home(r"C:\Users\example-user.", home), "~.");
        // another user whose name starts with this one, and paths outside the home folder
        assert_eq!(mask_home(r"C:\Users\example-users\x C:\Users\example-user2", home), r"C:\Users\example-users\x C:\Users\example-user2");
        assert_eq!(mask_home(r"D:\tools\triokey.exe", home), r"D:\tools\triokey.exe");
        // non-ASCII names compare by character
        assert_eq!(mask_home(r"C:\Users\Người-Dùng\AppData", r"c:\users\người-dùng"), r"~\AppData");
        assert!(!mask_home(r"C:\USERS\NGƯỜI-DÙNG\x", r"C:\Users\Người-Dùng").contains("NGƯỜI"));
        // macOS
        assert_eq!(mask_home("/Users/example-user/Library/Logs and /Users/example-user", "/Users/example-user/"), "~/Library/Logs and ~");
        for empty in ["", "/", r"C:\", "C:"] {
            assert_eq!(mask_home(r"C:\Users\x", empty), r"C:\Users\x");
        }
    }

    #[test]
    fn footer_language_and_addresses() {
        assert_eq!(footer("1.2.3"), "TrioKey v1.2.3");
        assert_eq!((pick(false, "Thoát", "Quit"), pick(true, "Thoát", "Quit")), ("Thoát", "Quit"));
        // the default the tests rely on
        assert!(!english() && t("Thoát", "Quit") == "Thoát");
        assert_eq!(REPOSITORY, env!("CARGO_PKG_REPOSITORY"));
        assert_eq!(RELEASES, format!("{REPOSITORY}/releases/latest"));
    }
}
