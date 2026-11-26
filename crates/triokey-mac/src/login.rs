//! "Khởi động cùng máy": a LaunchAgent property list in `~/Library/LaunchAgents` (docs/ARCHITECTURE.md, 12.14 Start at
//! login). No KeepAlive: a TrioKey that launchd restarts could not be quit before its Accessibility entry is touched.

/// File name of the login item; its Label is the bundle identifier.
pub const FILE: &str = "io.github.ved0el.triokey.plist";

const HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
\t<key>Label</key>
\t<string>io.github.ved0el.triokey</string>
\t<key>ProgramArguments</key>
\t<array>
\t\t<string>";
const TAIL: &str = "</string>
\t</array>
\t<key>RunAtLoad</key>
\t<true/>
</dict>
</plist>
";

/// The property list that starts `exe` at login. `None` for a path XML cannot hold (a control character).
pub fn plist(exe: &str) -> Option<String> {
    if exe.is_empty() || exe.chars().any(char::is_control) {
        return None;
    }
    let esc = exe.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;");
    Some(format!("{HEAD}{esc}{TAIL}"))
}

/// The program `text` starts, only when `text` is exactly what `plist` writes for it: anything else is not ours.
pub fn program(text: &str) -> Option<String> {
    let esc = text.strip_prefix(HEAD)?.strip_suffix(TAIL)?;
    let exe = esc.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&");
    (plist(&exe)?.as_str() == text).then_some(exe)
}

/// The login item can be set only for a copy in /Applications: one run from Downloads or a translocated copy would
/// leave a login item pointing at a place that goes away.
pub fn usable(exe: &str) -> bool {
    exe.starts_with("/Applications/") && plist(exe).is_some()
}

/// The login item for the debug information. `text`: the file's contents, `None` when there is none.
pub fn state(text: Option<&str>, exe: &str) -> &'static str {
    match text {
        _ if !usable(exe) => "unavailable (not in /Applications)",
        None => "off",
        Some(t) if program(t).as_deref() == Some(exe) => "on",
        Some(_) => "off (the file starts another program)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXE: &str = "/Applications/TrioKey.app/Contents/MacOS/triokey-mac";

    #[test]
    fn plist_names_the_program_and_keeps_nothing_alive() {
        let p = plist(EXE).unwrap();
        assert!(p.contains("\t\t<string>/Applications/TrioKey.app/Contents/MacOS/triokey-mac</string>\n"));
        assert!(p.contains("<key>Label</key>\n\t<string>io.github.ved0el.triokey</string>") && p.contains("<key>RunAtLoad</key>\n\t<true/>"));
        assert!(!p.contains("KeepAlive"));
        assert_eq!(program(&p).as_deref(), Some(EXE));
    }

    #[test]
    fn odd_paths_round_trip_and_foreign_files_are_not_ours() {
        let odd = "/Applications/A & <B> \"c\" 'd'/x";
        let p = plist(odd).unwrap();
        assert!(p.contains("A &amp; &lt;B&gt; &quot;c&quot; &apos;d&apos;/x"));
        assert_eq!(program(&p).as_deref(), Some(odd));
        assert_eq!(plist("/Applications/a\nb"), None);
        assert_eq!(plist(""), None);
        // not escaped, an extra key, another layout: refused
        assert_eq!(program(&format!("{HEAD}/a<b{TAIL}")), None);
        assert_eq!(program(&p.replace("<true/>", "<true/>\n\t<key>KeepAlive</key>\n\t<true/>")), None);
        assert_eq!(program(&p.replace('\t', "  ")), None);
        assert_eq!(program(""), None);
    }

    #[test]
    fn only_from_applications() {
        assert!(usable(EXE));
        assert!(!usable("/Users/x/Downloads/TrioKey.app/Contents/MacOS/triokey-mac"));
        assert!(!usable("/private/var/folders/x/AppTranslocation/ABC/d/TrioKey.app/Contents/MacOS/triokey-mac"));
        let p = plist(EXE).unwrap();
        assert_eq!(state(Some(&p), EXE), "on");
        assert_eq!(state(None, EXE), "off");
        assert_eq!(state(Some(&plist("/Applications/Old.app/x").unwrap()), EXE), "off (the file starts another program)");
        assert_eq!(state(Some(&p), "/Users/x/Downloads/t"), "unavailable (not in /Applications)");
    }
}
