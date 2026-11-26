//! What the Windows and the macOS app share and no OS call is needed for: the settings format
//! (`config`), the credit and addresses both windows show, and masking the home folder in the
//! debug information both apps copy to the clipboard.

pub mod config;

/// The author's public handle and the year in the credit line.
pub const AUTHOR: &str = "ved0el";
pub const YEAR: u16 = 2026;
/// The crates' `repository` field, and the page of the latest release.
pub const REPOSITORY: &str = "https://github.com/ved0el/triokey";
pub const RELEASES: &str = "https://github.com/ved0el/triokey/releases/latest";

/// The footer credit, `TrioKey v<version> · © 2026 ved0el`; `version` is the app crate's own.
pub fn credit(version: &str) -> String {
    format!("TrioKey v{version} · © {YEAR} {AUTHOR}")
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
    fn credit_and_addresses() {
        assert_eq!(credit("1.2.3"), "TrioKey v1.2.3 · © 2026 ved0el");
        assert_eq!(REPOSITORY, env!("CARGO_PKG_REPOSITORY"));
        assert_eq!(RELEASES, format!("{REPOSITORY}/releases/latest"));
    }
}
