//! What the menu bar item, the menu and the guide window say, in one place so it is tested and reviewed together.

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
pub const MENU_DIAG: &str = "Chẩn đoán (chép vào clipboard)";
pub const MENU_QUIT: &str = "Thoát";
pub const DIAG_COPIED: &str = "Đã chép thông tin chẩn đoán vào clipboard.";
pub const DIAG_COPIED_DETAIL: &str = "Dán vào tin nhắn hoặc báo lỗi. Thông tin này không chứa chữ bạn đã gõ.";

pub fn menu_version() -> String {
    format!("TrioKey {}", crate::VERSION)
}

pub const GUIDE_TITLE: &str = "Hướng dẫn cấp quyền";
pub const GUIDE_WHY: &str = "TrioKey phải thấy phím bạn gõ để bỏ dấu; nó không gửi gì ra ngoài.";
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
    }
}
