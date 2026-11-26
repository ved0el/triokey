//! TrioKey for macOS: the logic that needs no macOS call, kept here so it is tested on every system
//! (docs/ARCHITECTURE.md, 12. macOS app). The app itself is in `src/mac/`, built only on macOS.

pub mod diag;
pub mod hotkey;
pub mod keymap;
pub mod login;
pub mod plan;
pub mod source;
pub mod store;
pub mod text;
pub mod version;

/// Bundle identifier, fixed: the Accessibility grant and the single-instance check are tied to it.
pub const BUNDLE_ID: &str = "io.github.ved0el.triokey";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The argument "Khởi động lại" starts the new copy with: it waits for the old one to end before the single-instance
/// check gives up (docs/ARCHITECTURE.md, 12.8 Bundle identifier).
pub const RESTARTED: &str = "--restarted";

/// (program, arguments) that start a new copy of `exe`, the running executable: `open -n` on its bundle when it runs
/// from one (`…/X.app/Contents/MacOS/…`), so the new copy gets the bundle's identity, else `exe` itself.
pub fn restart_command(exe: &str) -> (String, Vec<String>) {
    const IN_BUNDLE: &str = ".app/Contents/MacOS/";
    match exe.rfind(IN_BUNDLE) {
        Some(i) if i > 0 => ("/usr/bin/open".into(), vec!["-n".into(), exe[..i + 4].into(), "--args".into(), RESTARTED.into()]),
        _ => (exe.into(), vec![RESTARTED.into()]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_opens_the_bundle_or_runs_the_program() {
        let (p, a) = restart_command("/Applications/TrioKey.app/Contents/MacOS/triokey-mac");
        assert_eq!((p.as_str(), a), ("/usr/bin/open", vec!["-n".to_string(), "/Applications/TrioKey.app".into(), "--args".into(), RESTARTED.into()]));
        assert_eq!(restart_command("/Users/x/My Apps/A.app/B.app/Contents/MacOS/t").1[1], "/Users/x/My Apps/A.app/B.app");
        for bare in ["/Users/x/src/target/debug/triokey-mac", ".app/Contents/MacOS/t"] {
            assert_eq!(restart_command(bare), (bare.to_string(), vec![RESTARTED.to_string()]));
        }
    }
}
