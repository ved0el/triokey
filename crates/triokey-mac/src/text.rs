//! What the menu bar item, the menu and the guide window say, in one place so it is tested and reviewed together.

use triokey_shared::config::Hotkey;

use crate::source::Decision;

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

pub fn title(s: State) -> &'static str {
    if s == State::Vietnamese {
        "V"
    } else {
        "E"
    }
}

pub fn tooltip(s: State) -> &'static str {
    match s {
        State::Vietnamese => "TrioKey: Tiếng Việt",
        State::Off => "TrioKey: Tiếng Anh — đã tắt",
        State::Yield => "TrioKey: Tiếng Anh — đang nhường bộ gõ khác",
        State::NoPermission => "TrioKey: chưa có quyền Accessibility",
        State::SecureInput => "TrioKey: bàn phím đang bị Secure Input chặn",
        State::Error => "TrioKey: lỗi, đã ngừng bắt phím",
    }
}

pub const MENU_TOGGLE: &str = "Tiếng Việt";
pub const MENU_GUIDE: &str = "Hướng dẫn cấp quyền…";
pub const MENU_SETTINGS: &str = "Cài đặt…";
pub const MENU_QUIT: &str = "Thoát";
pub const DIAG_COPIED: &str = "Đã chép thông tin debug vào clipboard.";
pub const DIAG_COPIED_DETAIL: &str = "Dán vào tin nhắn hoặc báo lỗi. Thông tin này không chứa chữ bạn đã gõ.";

/// The menu item while a newer version waits.
pub fn menu_waiting(version: &str) -> String {
    format!("Có bản {version} — mở trang tải về")
}

/// The settings window, top to bottom (the same texts as crates/triokey/src/settings_ui.rs where macOS has the same thing).
pub const SETTINGS_TITLE: &str = "TrioKey – Cài đặt";
pub const SET_ENABLED: &str = "Bật gõ tiếng Việt";
pub const SET_HOTKEY: &str = "Phím tắt";
pub const SET_TYPING: &str = "Gõ";
pub const SET_TONE: &str = "Kiểu bỏ dấu";
pub const SET_TONE_OLD: &str = "hòa, thúy";
pub const SET_TONE_NEW: &str = "hoà, thuý";
pub const SET_AUTO_EN: &str = "Tự giữ nguyên từ tiếng Anh";
pub const SET_YIELD: &str = "Nhường khi bộ gõ Nhật/Trung/Hàn đang bật";
pub const SET_SYSTEM: &str = "Hệ thống";
pub const SET_LOGIN: &str = "Khởi động cùng máy";
pub const SET_LOGIN_ONLY_APPS: &str = "Chỉ dùng được khi TrioKey nằm trong Applications";
pub const SET_DEV: &str = "Chế độ phát triển";
pub const SET_FOLDER: &str = "Mở thư mục cấu hình…";
pub const SET_DEBUG: &str = "Chép thông tin debug";
pub const SET_UPDATES: &str = "Cập nhật";
/// Not "Tự động cập nhật" as on Windows: on macOS nothing is installed, only checked (config key `auto_update`).
pub const SET_AUTO_CHECK: &str = "Tự động kiểm tra bản mới";
pub const SET_CHECK: &str = "Kiểm tra bản mới";
pub const SET_LINK: &str = "Tải tại đây";
pub const SET_GITHUB: &str = "GitHub";
pub const CONFIG_UNREADABLE: &str = "Không đọc được config.txt (file phải là UTF-8). TrioKey đang dùng giá trị mặc định và giữ nguyên file đó: thay đổi ở đây chỉ có hiệu lực tới khi Thoát, không được lưu.\n\nSửa hoặc xoá config.txt rồi mở lại TrioKey.";

/// (setting, label) in pop-up order. No Control+Space: it is macOS's own input-source switch.
pub const HOTKEYS: [(Hotkey, &str); 3] = [(Hotkey::CtrlShift, "Control+Shift"), (Hotkey::AltZ, "Option+Z"), (Hotkey::Off, "Không dùng")];

/// Pop-up row of a stored hotkey; `ctrl_space` shows as "Không dùng".
pub fn hotkey_index(h: Hotkey) -> usize {
    HOTKEYS.iter().position(|(k, _)| *k == crate::hotkey::effective(h)).unwrap_or(HOTKEYS.len() - 1)
}

pub const UPDATE_CHECKING: &str = "Đang kiểm tra…";
pub const UPDATE_LATEST: &str = "Đang dùng bản mới nhất.";
pub const UPDATE_FAILED: &str = "Chưa kiểm tra được: không tải được từ GitHub.";

pub fn update_newer(version: &str) -> String {
    format!("Có bản mới v{version}.")
}

pub fn menu_version() -> String {
    format!("TrioKey {}", crate::VERSION)
}

pub const GUIDE_TITLE: &str = "Hướng dẫn cấp quyền";
pub const GUIDE_WHY: &str = "TrioKey phải thấy phím bạn gõ để bỏ dấu; nó không gửi chữ bạn gõ đi đâu cả.";
pub const GUIDE_STEPS: &str = "1. Bấm nút bên dưới để mở Cài đặt hệ thống → Quyền riêng tư & Bảo mật → Trợ năng (Accessibility).\n\
2. Bật công tắc TrioKey.\n\
3. Nếu không thấy TrioKey trong danh sách thì bấm dấu + và chọn TrioKey trong thư mục Applications.";
pub const GUIDE_BUTTON: &str = "Mở cài đặt Accessibility";
pub const GUIDE_RELAUNCH: &str = "Nếu đã bật mà vẫn báo chưa có quyền: Thoát TrioKey rồi mở lại.";
pub const GUIDE_WARNING: &str = "Trước khi tắt hoặc xoá TrioKey khỏi danh sách Accessibility, hãy THOÁT TrioKey trước. Gỡ quyền khi TrioKey đang chạy có thể làm bàn phím và chuột đứng cho tới khi khởi động lại máy.\n\n\
Sau khi cài bản mới: Thoát TrioKey, chép bản mới đè lên, rồi xoá TrioKey khỏi danh sách (nút −) và thêm lại.";

pub fn guide_status(tap: bool) -> &'static str {
    if tap {
        "Đã có quyền, TrioKey đang chạy"
    } else {
        "Chưa có quyền"
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
    fn title_and_tooltip() {
        assert_eq!(title(State::Vietnamese), "V");
        for s in [State::Off, State::Yield, State::NoPermission, State::SecureInput, State::Error] {
            assert_eq!(title(s), "E");
            assert!(tooltip(s).starts_with("TrioKey: "));
        }
        assert_eq!(tooltip(State::Yield), "TrioKey: Tiếng Anh — đang nhường bộ gõ khác");
        assert_eq!(menu_version(), format!("TrioKey {}", env!("CARGO_PKG_VERSION")));
        assert_eq!(guide_status(false), "Chưa có quyền");
        assert_eq!(menu_waiting("0.3.1"), "Có bản 0.3.1 — mở trang tải về");
        assert_eq!(update_newer("0.3.1"), "Có bản mới v0.3.1.");
    }

    #[test]
    fn hotkey_rows() {
        assert_eq!([Hotkey::CtrlShift, Hotkey::AltZ, Hotkey::Off, Hotkey::CtrlSpace].map(hotkey_index), [0, 1, 2, 2]);
    }
}
