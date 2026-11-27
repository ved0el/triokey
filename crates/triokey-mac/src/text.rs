//! What the menu bar item, the menu, the guide and settings windows say, in one place so it is tested and reviewed
//! together. Every text is in Vietnamese and English and is read through the interface language
//! (`triokey_shared::t`, docs/ARCHITECTURE.md, 12.13 Settings, files and the window).

use triokey_shared::config::Hotkey;
use triokey_shared::glyph::{self, Glyph};
use triokey_shared::{english, pick};

use crate::source::Decision;
use crate::version::Answer;

/// A text in (Vietnamese, English): `get` gives it in the interface language, `pick` in the one asked for (tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Text(pub &'static str, pub &'static str);

impl Text {
    pub fn get(self) -> &'static str {
        self.pick(english())
    }

    pub fn pick(self, english: bool) -> &'static str {
        pick(english, self.0, self.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Vietnamese,
    Off,
    Yield,
    NoPermission,
    SecureInput,
    Error,
}

/// The first cause that stops typing wins: an error stops the tap for good, without a tap nothing else matters, and
/// Secure Input hides every key whatever the switch says.
pub fn state(failed: bool, tap: bool, secure: bool, on: bool, decision: Decision) -> State {
    match () {
        _ if failed => State::Error,
        _ if !tap => State::NoPermission,
        _ if secure => State::SecureInput,
        _ if !on => State::Off,
        _ if decision == Decision::Yield => State::Yield,
        _ => State::Vietnamese,
    }
}

/// The menu bar glyph: V while typing Vietnamese, E otherwise.
pub fn glyph(s: State) -> Glyph {
    if s == State::Vietnamese {
        Glyph::V
    } else {
        Glyph::E
    }
}

/// The glyph as a letter: the menu bar item's title when its image cannot be made.
pub fn title(s: State) -> &'static str {
    if glyph(s) == Glyph::V {
        "V"
    } else {
        "E"
    }
}

/// The menu bar image's pixels, `n`×`n` RGBA top row first, premultiplied as an `NSBitmapImageRep` without a format
/// flag holds them; `light_bar` picks the deep shades (Windows palette, `triokey_shared::glyph`).
pub fn icon(g: Glyph, n: u32, light_bar: bool) -> Vec<u8> {
    let mut px = glyph::render(g, n, glyph::color(g, light_bar));
    for p in px.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        for c in &mut p[..3] {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
    }
    px
}

pub fn tooltip(s: State) -> Text {
    match s {
        State::Vietnamese => Text("TrioKey: Tiếng Việt", "TrioKey: Vietnamese"),
        State::Off => Text("TrioKey: Tiếng Anh — đã tắt", "TrioKey: English — off"),
        State::Yield => Text("TrioKey: Tiếng Anh — đang nhường bộ gõ khác", "TrioKey: English — yielding to another input method"),
        State::NoPermission => Text("TrioKey: chưa có quyền Accessibility", "TrioKey: no Accessibility permission yet"),
        State::SecureInput => Text("TrioKey: bàn phím đang bị Secure Input chặn", "TrioKey: the keyboard is blocked by Secure Input"),
        State::Error => Text("TrioKey: lỗi, đã ngừng bắt phím", "TrioKey: error, stopped catching keys"),
    }
}

pub const MENU_TOGGLE: Text = Text("Tiếng Việt", "Vietnamese");
pub const MENU_GUIDE: Text = Text("Hướng dẫn cấp quyền…", "Permission guide…");
pub const MENU_SETTINGS: Text = Text("Cài đặt…", "Settings…");
pub const MENU_RESTART: Text = Text("Khởi động lại", "Restart");
pub const MENU_QUIT: Text = Text("Thoát", "Quit");
pub const DIAG_COPIED: Text = Text("Đã chép thông tin debug vào clipboard.", "Debug info copied to the clipboard.");
pub const DIAG_COPIED_DETAIL: Text = Text(
    "Dán vào tin nhắn hoặc báo lỗi. Thông tin này không chứa chữ bạn đã gõ.",
    "Paste it into a message or a bug report. It holds none of the text you typed.",
);

/// The menu item while a newer version waits.
pub fn menu_waiting(version: &str, english: bool) -> String {
    if english {
        format!("Version {version} is available — open the download page")
    } else {
        format!("Có bản {version} — mở trang tải về")
    }
}

/// The settings window, top to bottom (the same texts as crates/triokey/src/settings_ui.rs where macOS has the same thing).
pub const SETTINGS_TITLE: Text = Text("TrioKey – Cài đặt", "TrioKey – Settings");
pub const SET_ENABLED: Text = Text("Bật gõ tiếng Việt", "Enable Vietnamese");
pub const SET_HOTKEY: Text = Text("Phím tắt", "Shortcut");
pub const SET_TYPING: Text = Text("Gõ", "Typing");
pub const SET_TONE: Text = Text("Kiểu bỏ dấu", "Tone mark placement");
pub const SET_TONE_OLD: Text = Text("hòa, thúy", "hòa, thúy");
pub const SET_TONE_NEW: Text = Text("hoà, thuý", "hoà, thuý");
pub const SET_AUTO_EN: Text = Text("Tự giữ nguyên từ tiếng Anh", "Keep English words as typed");
pub const SET_YIELD: Text = Text("Nhường khi bộ gõ Nhật/Trung/Hàn đang bật", "Step aside while a Japanese/Chinese/Korean input method is on");
pub const SET_SYSTEM: Text = Text("Hệ thống", "System");
pub const SET_LOGIN: Text = Text("Khởi động cùng máy", "Start at login");
pub const SET_LOGIN_ONLY_APPS: Text = Text("Chỉ dùng được khi TrioKey nằm trong Applications", "Only works when TrioKey is in Applications");
pub const SET_LANGUAGE: Text = Text("Ngôn ngữ", "Language");
pub const SET_DEV: Text = Text("Chế độ phát triển", "Developer mode");
pub const SET_FOLDER: Text = Text("Mở thư mục cấu hình…", "Open config folder…");
pub const SET_DEBUG: Text = Text("Chép thông tin debug", "Copy debug info");
pub const SET_UPDATES: Text = Text("Cập nhật", "Updates");
/// Not "Tự động cập nhật" as on Windows: on macOS nothing is installed, only checked (config key `auto_update`).
pub const SET_AUTO_CHECK: Text = Text("Tự động kiểm tra bản mới", "Check for new versions automatically");
pub const SET_CHECK: Text = Text("Kiểm tra bản mới", "Check for updates");
pub const SET_LINK: Text = Text("Tải tại đây", "Download here");
pub const SET_GITHUB: Text = Text("GitHub", "GitHub");
pub const CONFIG_UNREADABLE: Text = Text(
    "Không đọc được config.txt (file phải là UTF-8). TrioKey đang dùng giá trị mặc định và giữ nguyên file đó: thay đổi ở đây chỉ có hiệu lực tới khi Thoát, không được lưu.\n\nSửa hoặc xoá config.txt rồi mở lại TrioKey.",
    "Could not read config.txt (the file must be UTF-8). TrioKey is using the default values and leaves that file as it is: changes made here last only until you quit and are not saved.\n\nFix or delete config.txt, then open TrioKey again.",
);

/// The language pop-up, `english` false then true; each name in its own language.
pub const LANGUAGES: [&str; 2] = ["Tiếng Việt", "English"];

/// (setting, label) in pop-up order. No Control+Space: it is macOS's own input-source switch.
pub const HOTKEYS: [(Hotkey, Text); 4] = [
    (Hotkey::CtrlShift, Text("Control+Shift", "Control+Shift")),
    (Hotkey::AltShift, Text("Option+Shift", "Option+Shift")),
    (Hotkey::AltZ, Text("Option+Z", "Option+Z")),
    (Hotkey::Off, Text("Không dùng", "None")),
];

/// Pop-up row of a hotkey.
pub fn hotkey_index(h: Hotkey) -> usize {
    HOTKEYS.iter().position(|(k, _)| *k == h).unwrap_or(0)
}

pub const UPDATE_CHECKING: Text = Text("Đang kiểm tra…", "Checking…");
pub const UPDATE_LATEST: Text = Text("Đang dùng bản mới nhất.", "You have the latest version.");
pub const UPDATE_FAILED: Text = Text("Chưa kiểm tra được: không tải được từ GitHub.", "Could not check: GitHub could not be reached.");

pub fn update_newer(version: &str, english: bool) -> String {
    if english {
        format!("Version {version} is available.")
    } else {
        format!("Có bản mới v{version}.")
    }
}

/// The reserved line of the settings window: a check in flight, the last answer, or nothing before the first check.
pub fn update_status(checking: bool, answer: Option<&Answer>, english: bool) -> String {
    match answer {
        _ if checking => UPDATE_CHECKING.pick(english).into(),
        Some(Answer::Newer(v)) => update_newer(v, english),
        Some(Answer::Latest) => UPDATE_LATEST.pick(english).into(),
        Some(Answer::Failed) => UPDATE_FAILED.pick(english).into(),
        None => String::new(),
    }
}

pub const GUIDE_TITLE: Text = Text("Hướng dẫn cấp quyền", "Permission guide");
pub const GUIDE_WHY: Text = Text(
    "TrioKey phải thấy phím bạn gõ để bỏ dấu; nó không gửi chữ bạn gõ đi đâu cả.",
    "TrioKey has to see the keys you type to add the tone marks; it sends what you type nowhere.",
);
pub const GUIDE_STEPS: Text = Text(
    "1. Bấm nút bên dưới để mở Cài đặt hệ thống → Quyền riêng tư & Bảo mật → Trợ năng (Accessibility).\n\
2. Bật công tắc TrioKey.\n\
3. Nếu không thấy TrioKey trong danh sách thì bấm dấu + và chọn TrioKey trong thư mục Applications.",
    "1. Click the button below to open System Settings → Privacy & Security → Accessibility.\n\
2. Turn on the TrioKey switch.\n\
3. If TrioKey is not in the list, click + and choose TrioKey in the Applications folder.",
);
pub const GUIDE_BUTTON: Text = Text("Mở cài đặt Accessibility", "Open Accessibility settings");
pub const GUIDE_RELAUNCH: Text = Text(
    "Nếu đã bật mà vẫn báo chưa có quyền: Thoát TrioKey rồi mở lại.",
    "If it is on and still says there is no permission: quit TrioKey and open it again.",
);
pub const GUIDE_WARNING: Text = Text(
    "Trước khi tắt hoặc xoá TrioKey khỏi danh sách Accessibility, hãy THOÁT TrioKey trước. Gỡ quyền khi TrioKey đang chạy có thể làm bàn phím và chuột đứng cho tới khi khởi động lại máy.\n\n\
Sau khi cài bản mới: Thoát TrioKey, chép bản mới đè lên, rồi xoá TrioKey khỏi danh sách (nút −) và thêm lại.",
    "Before you turn TrioKey off or remove it from the Accessibility list, QUIT TrioKey first. Removing the permission while TrioKey is running can freeze the keyboard and the mouse until you restart the Mac.\n\n\
After installing a new version: quit TrioKey, copy the new version over the old one, then remove TrioKey from the list (the − button) and add it again.",
);

pub fn guide_status(tap: bool) -> Text {
    if tap {
        Text("Đã có quyền, TrioKey đang chạy", "Permission granted, TrioKey is running")
    } else {
        Text("Chưa có quyền", "No permission yet")
    }
}

/// Opens Privacy & Security → Accessibility. Widely used, not documented by Apple.
pub const SETTINGS_URL: &str = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

#[cfg(test)]
mod tests {
    use super::*;
    use Decision::*;

    #[test]
    fn state_priority() {
        assert_eq!(state(true, false, true, false, Yield), State::Error);
        assert_eq!(state(false, false, true, true, Type), State::NoPermission);
        assert_eq!(state(false, true, true, false, Yield), State::SecureInput);
        assert_eq!(state(false, true, false, false, Type), State::Off);
        assert_eq!(state(false, true, false, true, Yield), State::Yield);
        assert_eq!(state(false, true, false, true, Type), State::Vietnamese);
    }

    #[test]
    fn glyph_and_tooltip() {
        assert_eq!((glyph(State::Vietnamese), title(State::Vietnamese)), (Glyph::V, "V"));
        for s in [State::Off, State::Yield, State::NoPermission, State::SecureInput, State::Error] {
            assert_eq!((glyph(s), title(s)), (Glyph::E, "E"));
            assert!(tooltip(s).get().starts_with("TrioKey: ") && tooltip(s).pick(true).starts_with("TrioKey: "));
        }
        // Vietnamese is the default, and what the tests see
        assert_eq!(tooltip(State::Yield).get(), "TrioKey: Tiếng Anh — đang nhường bộ gõ khác");
        assert_eq!(tooltip(State::Yield).pick(true), "TrioKey: English — yielding to another input method");
        assert_eq!(tooltip(State::Off).pick(true), "TrioKey: English — off");
        assert_eq!(tooltip(State::Vietnamese).pick(true), "TrioKey: Vietnamese");
        assert_eq!(guide_status(false).get(), "Chưa có quyền");
        assert_eq!(menu_waiting("0.3.1", false), "Có bản 0.3.1 — mở trang tải về");
        assert_eq!(update_newer("0.3.1", false), "Có bản mới v0.3.1.");
        assert_eq!(update_newer("0.3.1", true), "Version 0.3.1 is available.");
    }

    #[test]
    fn english_wording() {
        let pairs = [
            (MENU_TOGGLE, "Vietnamese"),
            (MENU_SETTINGS, "Settings…"),
            (MENU_GUIDE, "Permission guide…"),
            (MENU_RESTART, "Restart"),
            (MENU_QUIT, "Quit"),
            (SET_ENABLED, "Enable Vietnamese"),
            (SET_HOTKEY, "Shortcut"),
            (HOTKEYS[3].1, "None"),
            (SET_TYPING, "Typing"),
            (SET_TONE, "Tone mark placement"),
            (SET_AUTO_EN, "Keep English words as typed"),
            (SET_YIELD, "Step aside while a Japanese/Chinese/Korean input method is on"),
            (SET_SYSTEM, "System"),
            (SET_LOGIN, "Start at login"),
            (SET_LANGUAGE, "Language"),
            (SET_DEV, "Developer mode"),
            (SET_FOLDER, "Open config folder…"),
            (SET_DEBUG, "Copy debug info"),
            (SET_UPDATES, "Updates"),
            (SET_AUTO_CHECK, "Check for new versions automatically"),
            (SET_CHECK, "Check for updates"),
            (UPDATE_CHECKING, "Checking…"),
            (UPDATE_LATEST, "You have the latest version."),
            (SET_LINK, "Download here"),
            (UPDATE_FAILED, "Could not check: GitHub could not be reached."),
        ];
        for (text, en) in pairs {
            assert_eq!(text.pick(true), en);
            assert_eq!(text.get(), text.0);
        }
        assert_eq!((MENU_QUIT.get(), SET_LANGUAGE.get()), ("Thoát", "Ngôn ngữ"));
        assert!(GUIDE_WARNING.pick(true).contains("QUIT TrioKey first") && GUIDE_WARNING.get().contains("THOÁT TrioKey trước"));
        assert_eq!(update_status(true, Some(&Answer::Latest), true), "Checking…");
        assert_eq!(update_status(false, Some(&Answer::Newer("9.9.9".into())), false), "Có bản mới v9.9.9.");
        assert_eq!(update_status(false, Some(&Answer::Failed), true), UPDATE_FAILED.1);
        assert_eq!(update_status(false, None, false), "");
    }

    #[test]
    fn hotkey_rows() {
        assert_eq!([Hotkey::CtrlShift, Hotkey::AltShift, Hotkey::AltZ, Hotkey::Off].map(hotkey_index), [0, 1, 2, 3]);
        assert_eq!(HOTKEYS.map(|(_, t)| t.get()), ["Control+Shift", "Option+Shift", "Option+Z", "Không dùng"]);
    }

    #[test]
    fn icon_is_premultiplied() {
        for (g, light) in [(Glyph::V, true), (Glyph::E, false)] {
            let straight = glyph::render(g, 36, glyph::color(g, light));
            let pre = icon(g, 36, light);
            assert_eq!(pre.len(), straight.len());
            for (p, s) in pre.as_chunks::<4>().0.iter().zip(straight.as_chunks::<4>().0) {
                assert_eq!(p[3], s[3]);
                match s[3] {
                    0 => assert_eq!(&p[..3], &[0, 0, 0]),
                    255 => assert_eq!(p[..3], s[..3]),
                    a => assert!(p[..3].iter().all(|&c| c <= a)),
                }
            }
        }
    }
}
