//! Settings file (`%APPDATA%\triokey\config.txt`, `key = value` lines). Read and written in app.rs.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Hotkey {
    #[default]
    CtrlShift,
    AltZ,
    CtrlSpace,
    /// No hotkey: toggle from the tray only.
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub enabled: bool,
    pub tone_new: bool,
    pub auto_english: bool,
    pub yield_cjk: bool,
    pub hotkey: Hotkey,
    pub run_at_startup: bool,
    pub run_as_admin: bool,
    /// Check GitHub for a signed newer version once a day (update.rs). Off: no network use
    /// unless the user presses the check-now button.
    pub auto_update: bool,
    /// Unix seconds of the last check that got an answer; written by the updater.
    pub last_update_check: u64,
    /// Settings window shows the config folder button and the hook timing line (display only).
    pub dev_mode: bool,
}

impl Settings {
    pub const DEFAULT: Settings = Settings {
        enabled: true,
        tone_new: false,
        auto_english: true,
        yield_cjk: true,
        hotkey: Hotkey::CtrlShift,
        // the user's choice; in normal mode the settings window shows the Run value itself.
        run_at_startup: true,
        run_as_admin: false,
        auto_update: true,
        last_update_check: 0,
        dev_mode: false,
    };
}

impl Default for Settings {
    fn default() -> Self {
        Settings::DEFAULT
    }
}

/// (preset, config value, label in the settings window), in dropdown order.
pub const HOTKEYS: [(Hotkey, &str, &str); 4] = [
    (Hotkey::CtrlShift, "ctrl_shift", "Ctrl+Shift"),
    (Hotkey::AltZ, "alt_z", "Alt+Z"),
    (Hotkey::CtrlSpace, "ctrl_space", "Ctrl+Space"),
    (Hotkey::Off, "none", "Không dùng"),
];

/// Unknown keys, malformed lines and bad values are ignored; missing keys keep their default.
pub fn parse(text: &str) -> Settings {
    let mut s = Settings::default();
    // Notepad's "UTF-8 with BOM": trim() keeps it, and the first key would not be recognised
    for line in text.trim_start_matches('\u{FEFF}').lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let (k, v) = (k.trim(), v.trim());
        if k == "hotkey" {
            if let Some(&(h, _, _)) = HOTKEYS.iter().find(|(_, n, _)| *n == v) {
                s.hotkey = h;
            }
            continue;
        }
        if k == "last_update_check" {
            s.last_update_check = v.parse().unwrap_or(0);
            continue;
        }
        let Ok(b) = v.parse::<bool>() else { continue };
        match k {
            "enabled" => s.enabled = b,
            "tone_new" => s.tone_new = b,
            "auto_english" => s.auto_english = b,
            "yield_cjk" => s.yield_cjk = b,
            "run_at_startup" => s.run_at_startup = b,
            "run_as_admin" => s.run_as_admin = b,
            "auto_update" => s.auto_update = b,
            "dev_mode" => s.dev_mode = b,
            _ => {}
        }
    }
    s
}

pub fn render(s: &Settings) -> String {
    let hotkey = HOTKEYS.iter().find(|(h, _, _)| *h == s.hotkey).map_or("ctrl_shift", |(_, n, _)| n);
    format!(
        "enabled = {}\ntone_new = {}\nauto_english = {}\nyield_cjk = {}\nhotkey = {}\nrun_at_startup = {}\nrun_as_admin = {}\nauto_update = {}\nlast_update_check = {}\ndev_mode = {}\n",
        s.enabled, s.tone_new, s.auto_english, s.yield_cjk, hotkey, s.run_at_startup, s.run_as_admin, s.auto_update, s.last_update_check, s.dev_mode
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_default() {
        assert_eq!(parse(""), Settings::default());
        assert!(!Settings::default().dev_mode && render(&Settings::default()).contains("dev_mode = false\n"));
    }

    #[test]
    fn round_trip() {
        let d = Settings::default();
        assert_eq!(parse(&render(&d)), d);
        let s = Settings {
            enabled: false,
            tone_new: true,
            auto_english: false,
            yield_cjk: false,
            hotkey: Hotkey::CtrlSpace,
            run_at_startup: false,
            run_as_admin: true,
            auto_update: false,
            last_update_check: 1_795_000_000,
            dev_mode: true,
        };
        assert_eq!(parse(&render(&s)), s);
        assert_eq!(
            parse(
                "last_update_check = soon
auto_update = false"
            )
            .last_update_check,
            0
        );
        for (h, name, _) in HOTKEYS {
            let a = Settings { hotkey: h, ..d };
            assert!(render(&a).contains(&format!(
                "hotkey = {name}
"
            )));
            assert_eq!(parse(&render(&a)), a);
        }
        assert_eq!(parse("hotkey = none").hotkey, Hotkey::Off);
        assert_eq!(parse("hotkey = ctrl+alt+v").hotkey, Hotkey::CtrlShift);
    }

    #[test]
    fn bom_does_not_hide_the_first_key() {
        assert!(!parse("\u{FEFF}enabled = false\ntone_new = true\n").enabled);
    }

    #[test]
    fn junk_lines_ignored() {
        let s = parse("garbage\n# comment\nfoo = bar\n  tone_new =  true \r\nenabled = maybe\nhotkey = ctrl_space\n=\n");
        assert_eq!(s, Settings { tone_new: true, hotkey: Hotkey::CtrlSpace, ..Settings::default() });
    }
}
