//! Settings window (spec 5.7), plain Win32 controls on the tray thread. Every change is saved
//! to config.txt and applied at once. One window at a time.
//!
//! The embedded manifest (build.rs) gives the controls the current visual style and makes the
//! process per-monitor DPI aware: sizes come from the window's own DPI, never the system's.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering::Relaxed};
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{InitCommonControlsEx, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX};
use windows_sys::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow, SystemParametersInfoForDpi};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, GetActiveWindow, GetFocus, IsWindowEnabled, SetFocus};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::admin::{self, Elevated};
use crate::config::HOTKEYS;
use crate::tray::wide;
use crate::{app, autostart, hook, tray, update};

const CLASS: &str = "triokey-settings";
const ID_ENABLED: i32 = 101;
const ID_HOTKEY: i32 = 102;
const ID_TONE_OLD: i32 = 103;
const ID_TONE_NEW: i32 = 104;
const ID_AUTO_EN: i32 = 105;
const ID_YIELD: i32 = 106;
const ID_STARTUP: i32 = 107;
const ID_ADMIN: i32 = 108;
const ID_FOLDER: i32 = 109;
const ID_LATENCY: i32 = 110;
const ID_AUTO_UPDATE: i32 = 111;
const ID_CHECK: i32 = 112;
const ID_UPDATE_STATUS: i32 = 113;
const SS_RIGHT: u32 = 0x2;
const BST_CHECKED: isize = 1;
const STYLE: u32 = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
/// Client area in 96-DPI pixels. The layout in `build` follows the Windows dialog metrics for
/// Segoe UI 9 pt: 11 px margins, 17 px check boxes, 23 px buttons and combo boxes, 7 px between
/// related rows, 11 px between groups.
const WIDTH: i32 = 400;
const HEIGHT: i32 = 296;

static HWND_SETTINGS: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
static FONT: AtomicUsize = AtomicUsize::new(0);
/// The control that had the keyboard focus when the window was last deactivated.
static FOCUS: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
/// Window icons (small, big), made once.
static ICONS: OnceLock<[usize; 2]> = OnceLock::new();

pub fn hwnd() -> HWND {
    HWND_SETTINGS.load(Relaxed)
}

/// Tray thread: the on/off switch changed (hotkey, tray click) while the window may be open.
pub fn sync_enabled() {
    let h = hwnd();
    if !h.is_null() {
        unsafe { set_check(h, ID_ENABLED, app::enabled()) };
    }
}

/// Tray thread: the updater has something new to say while the window may be open.
pub fn sync_update() {
    let h = hwnd();
    if !h.is_null() {
        unsafe { SetWindowTextW(GetDlgItem(h, ID_UPDATE_STATUS), wide(&update::status()).as_ptr()) };
    }
}

/// 96-DPI pixels to pixels at `dpi`.
fn px(v: i32, dpi: u32) -> i32 {
    v * dpi as i32 / 96
}

/// Outer window size for the client area at `dpi`.
unsafe fn window_size(dpi: u32) -> (i32, i32) {
    let mut rc = RECT { left: 0, top: 0, right: px(WIDTH, dpi), bottom: px(HEIGHT, dpi) };
    AdjustWindowRectExForDpi(&mut rc, STYLE, 0, 0, dpi);
    (rc.right - rc.left, rc.bottom - rc.top)
}

/// Tray thread. Brings the open window to the front instead of opening a second one.
pub fn open() {
    unsafe {
        let old = hwnd();
        if !old.is_null() {
            ShowWindow(old, SW_RESTORE);
            SetForegroundWindow(old);
            return;
        }
        // makes sure comctl32 (version 6, chosen by the manifest) is loaded before the controls
        InitCommonControlsEx(&INITCOMMONCONTROLSEX { dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32, dwICC: ICC_STANDARD_CLASSES });
        let hinst = GetModuleHandleW(null());
        let class = wide(CLASS);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinst,
            lpszClassName: class.as_ptr(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_BTNFACE + 1) as usize as HBRUSH,
            ..std::mem::zeroed()
        };
        // fails harmlessly when already registered
        RegisterClassW(&wc);
        // created empty at the primary monitor's origin, so its DPI is that monitor's
        let hwnd = CreateWindowExW(0, class.as_ptr(), wide("TrioKey – Cài đặt").as_ptr(), STYLE, 0, 0, 0, 0, null_mut(), null_mut(), hinst, null());
        if hwnd.is_null() {
            return;
        }
        HWND_SETTINGS.store(hwnd, Relaxed);
        // the app icon (resource 1, build.rs) at the exact sizes this monitor wants
        let icons = ICONS.get_or_init(|| {
            [GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CXICON)].map(|n| LoadImageW(hinst, 1 as _, IMAGE_ICON, n, n, LR_DEFAULTCOLOR) as usize)
        });
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, icons[0] as isize);
        SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, icons[1] as isize);
        let dpi = match GetDpiForWindow(hwnd) {
            0 => 96,
            d => d,
        };
        let (w, h) = window_size(dpi);
        let (x, y) = ((GetSystemMetrics(SM_CXSCREEN) - w) / 2, (GetSystemMetrics(SM_CYSCREEN) - h) / 2);
        SetWindowPos(hwnd, null_mut(), x, y, w, h, SWP_NOZORDER);
        build(hwnd, dpi);
        fill(hwnd);
        SetTimer(hwnd, 1, 1000, None);
        // activation ends in WM_SETFOCUS, which puts the focus on the first control
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        if app::config_unreadable() {
            let t = "Không đọc được config.txt (file phải là UTF-8). TrioKey đang dùng giá trị mặc định và giữ nguyên file đó: thay đổi ở đây chỉ có hiệu lực tới khi Thoát, không được lưu.\n\nSửa hoặc xoá config.txt rồi chạy lại TrioKey.";
            message(hwnd, t, MB_OK | MB_ICONWARNING);
        }
        warn_unsaved(hwnd);
    }
}

/// config.txt could not be written (at startup or by the last change): never silently.
unsafe fn warn_unsaved(hwnd: HWND) {
    if app::take_save_failed() {
        let t = "Không lưu được config.txt: thay đổi chỉ có hiệu lực tới khi Thoát.\n\nBản chạy quyền admin chỉ ghi lại file config.txt đã có sẵn (không tạo mới, không ghi qua liên kết). Nếu chưa có file, hãy chạy TrioKey bản thường một lần.";
        message(hwnd, t, MB_OK | MB_ICONWARNING);
    }
}

/// Create the controls for `dpi`. Called again after a DPI change, once the old ones are gone.
unsafe fn build(hwnd: HWND, dpi: u32) {
    let mut ncm: NONCLIENTMETRICSW = std::mem::zeroed();
    ncm.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    // the system message font (Segoe UI 9 pt) at this window's DPI
    let font = if SystemParametersInfoForDpi(SPI_GETNONCLIENTMETRICS, ncm.cbSize, (&mut ncm as *mut NONCLIENTMETRICSW).cast(), 0, dpi) != 0 {
        CreateFontIndirectW(&ncm.lfMessageFont)
    } else {
        GetStockObject(DEFAULT_GUI_FONT)
    };
    let old = FONT.swap(font as usize, Relaxed);
    if old != 0 {
        // deleting the stock fallback font is a harmless no-op
        DeleteObject(old as HFONT);
    }
    let hinst = GetModuleHandleW(null());
    let ctl = |class: &str, text: &str, style: u32, id: i32, x: i32, y: i32, w: i32, h: i32| {
        let c = CreateWindowExW(
            0,
            wide(class).as_ptr(),
            wide(text).as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            px(x, dpi),
            px(y, dpi),
            px(w, dpi),
            px(h, dpi),
            hwnd,
            id as usize as HMENU,
            hinst,
            null(),
        );
        SendMessageW(c, WM_SETFONT, font as usize, 1);
        c
    };
    let check = (BS_AUTOCHECKBOX as u32) | WS_TABSTOP;
    // labels sit 4 px below the top of a 23 px control so the text baselines line up
    ctl("BUTTON", "Bật gõ tiếng Việt", check | WS_GROUP, ID_ENABLED, 11, 14, 200, 17);
    ctl("STATIC", "Phím tắt", SS_RIGHT, 0, 219, 15, 60, 15);
    let combo = ctl("COMBOBOX", "", (CBS_DROPDOWNLIST as u32) | WS_TABSTOP | WS_VSCROLL, ID_HOTKEY, 285, 11, 104, 200);
    for (_, _, label) in HOTKEYS {
        SendMessageW(combo, CB_ADDSTRING, 0, wide(label).as_ptr() as isize);
    }
    ctl("STATIC", "Kiểu bỏ dấu", 0, 0, 11, 46, 100, 15);
    let radio = (BS_AUTORADIOBUTTON as u32) | WS_TABSTOP;
    ctl("BUTTON", "hòa, thúy", radio | WS_GROUP, ID_TONE_OLD, 118, 45, 110, 17);
    ctl("BUTTON", "hoà, thuý", radio, ID_TONE_NEW, 234, 45, 110, 17);
    // WS_GROUP ends the radio group
    ctl("BUTTON", "Tự giữ nguyên từ tiếng Anh", check | WS_GROUP, ID_AUTO_EN, 11, 73, 378, 17);
    ctl("BUTTON", "Nhường khi IME Nhật/Trung/Hàn đang bật", check, ID_YIELD, 11, 97, 378, 17);
    ctl("BUTTON", "Khởi động cùng Windows", check, ID_STARTUP, 11, 121, 378, 17);
    ctl("BUTTON", "Chạy quyền admin (gõ được trong cửa sổ admin)", check, ID_ADMIN, 11, 145, 378, 17);
    ctl("BUTTON", "Tự động cập nhật", check, ID_AUTO_UPDATE, 11, 169, 200, 17);
    let button = (BS_PUSHBUTTON as u32) | WS_TABSTOP;
    ctl("BUTTON", "Kiểm tra bản mới", button, ID_CHECK, 259, 166, 130, 23);
    // two lines: the longest status carries the address of the releases page
    ctl("STATIC", "", 0, ID_UPDATE_STATUS, 11, 195, 378, 30);
    ctl("BUTTON", "Mở thư mục cấu hình…", button | WS_GROUP, ID_FOLDER, 11, 236, 160, 23);
    ctl("STATIC", concat!("v", env!("CARGO_PKG_VERSION")), SS_RIGHT, 0, 289, 240, 100, 15);
    ctl("STATIC", "", 0, ID_LATENCY, 11, 270, 378, 15);
}

unsafe fn set_check(hwnd: HWND, id: i32, on: bool) {
    SendMessageW(GetDlgItem(hwnd, id), BM_SETCHECK, on as usize, 0);
}

unsafe fn checked(hwnd: HWND, id: i32) -> bool {
    SendMessageW(GetDlgItem(hwnd, id), BM_GETCHECK, 0, 0) == BST_CHECKED
}

/// Show the real state: live on/off switch, Run value, config for the rest.
unsafe fn fill(hwnd: HWND) {
    let s = app::settings();
    set_check(hwnd, ID_ENABLED, app::enabled());
    let i = HOTKEYS.iter().position(|(h, _, _)| *h == s.hotkey).unwrap_or(0);
    SendMessageW(GetDlgItem(hwnd, ID_HOTKEY), CB_SETCURSEL, i, 0);
    set_check(hwnd, ID_TONE_OLD, !s.tone_new);
    set_check(hwnd, ID_TONE_NEW, s.tone_new);
    set_check(hwnd, ID_AUTO_EN, s.auto_english);
    set_check(hwnd, ID_YIELD, s.yield_cjk);
    // admin mode: the logon task starts TrioKey, and the admin copy must not end up in Run
    let locked = s.run_as_admin || admin::is_admin_copy();
    set_check(hwnd, ID_STARTUP, if s.run_as_admin { true } else { autostart::is_on() });
    EnableWindow(GetDlgItem(hwnd, ID_STARTUP), (!locked) as i32);
    set_check(hwnd, ID_ADMIN, s.run_as_admin);
    // a build without an update key (update-pubkey.txt) cannot update: say so, offer nothing
    set_check(hwnd, ID_AUTO_UPDATE, s.auto_update && update::available());
    EnableWindow(GetDlgItem(hwnd, ID_AUTO_UPDATE), update::available() as i32);
    EnableWindow(GetDlgItem(hwnd, ID_CHECK), update::available() as i32);
    sync_update();
    latency(hwnd);
}

unsafe fn latency(hwnd: HWND) {
    let ((sent, ns), (pass, np)) = hook::latency();
    let t = format!("Hook p99: gửi {sent} µs · không gửi {pass} µs ({} phím)", ns + np);
    SetWindowTextW(GetDlgItem(hwnd, ID_LATENCY), wide(&t).as_ptr());
}

unsafe fn message(hwnd: HWND, text: &str, flags: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    MessageBoxW(hwnd, wide(text).as_ptr(), wide("TrioKey").as_ptr(), flags)
}

unsafe fn command(hwnd: HWND, id: i32, code: u32) {
    match (id, code) {
        (ID_ENABLED, BN_CLICKED) => {
            let on = checked(hwnd, ID_ENABLED);
            app::set_enabled(on);
            app::update(|s| s.enabled = on);
        }
        (ID_HOTKEY, CBN_SELCHANGE) => {
            let i = SendMessageW(GetDlgItem(hwnd, ID_HOTKEY), CB_GETCURSEL, 0, 0);
            if let Some(&(h, _, _)) = usize::try_from(i).ok().and_then(|i| HOTKEYS.get(i)) {
                app::update(|s| s.hotkey = h);
            }
        }
        (ID_TONE_OLD | ID_TONE_NEW, BN_CLICKED) => app::update(|s| s.tone_new = id == ID_TONE_NEW),
        (ID_AUTO_EN, BN_CLICKED) => {
            let on = checked(hwnd, id);
            app::update(|s| s.auto_english = on);
        }
        (ID_YIELD, BN_CLICKED) => {
            let on = checked(hwnd, id);
            app::update(|s| s.yield_cjk = on);
        }
        (ID_STARTUP, BN_CLICKED) => {
            let on = checked(hwnd, id);
            match std::env::current_exe() {
                Ok(exe) if autostart::set(on, &exe) => app::update(|s| s.run_at_startup = on),
                _ => {
                    message(hwnd, "Không ghi được mục tự khởi động trong registry.", MB_OK | MB_ICONWARNING);
                }
            }
            fill(hwnd);
        }
        (ID_ADMIN, BN_CLICKED) => {
            switch_admin(hwnd, checked(hwnd, id));
            fill(hwnd);
        }
        (ID_AUTO_UPDATE, BN_CLICKED) => {
            let on = checked(hwnd, id);
            app::update(|s| s.auto_update = on);
        }
        // the answer arrives through `sync_update`; the request runs on the updater thread
        (ID_CHECK, BN_CLICKED) => update::check_now(),
        (ID_FOLDER, BN_CLICKED) => {
            let Some(dir) = app::config_dir() else { return };
            // The shell looks up the "open" verb in HKCU\Software\Classes, where an unelevated
            // program can register its own command, which would then run with this token:
            // an elevated TrioKey starts nothing and creates nothing here, it shows the path.
            if admin::is_elevated() {
                let t = format!("TrioKey đang chạy quyền admin nên không tự mở Explorer. Thư mục cấu hình:\n\n{}", dir.display());
                message(hwnd, &t, MB_OK | MB_ICONINFORMATION);
                return;
            }
            let _ = std::fs::create_dir_all(&dir);
            if dir.is_dir() {
                let p = wide(&dir.to_string_lossy());
                ShellExecuteW(hwnd, wide("open").as_ptr(), p.as_ptr(), null(), null(), SW_SHOWNORMAL);
            }
        }
        _ => {}
    }
}

/// The admin-only part runs first (one UAC prompt from the normal copy, none when already
/// elevated), then this side updates HKCU and config.txt (see admin.rs).
unsafe fn switch_admin(hwnd: HWND, on: bool) {
    let code = match admin::run_elevated(hwnd, on) {
        Elevated::Cancelled => return,
        Elevated::Exit(c) if c == admin::OK || c == admin::OK_AFTER_REBOOT || c == admin::OK_FOLDER_LEFT => c,
        Elevated::Timeout => {
            let t = "Bản quyền admin chưa chạy xong sau 2 phút nên chưa biết kết quả. Cấu hình chưa đổi; hãy kiểm tra tác vụ TrioKey trong Task Scheduler rồi thử lại.";
            message(hwnd, t, MB_OK | MB_ICONWARNING);
            return;
        }
        r => {
            let what = if on { "bật" } else { "tắt" };
            let why = match r {
                Elevated::Exit(admin::COPY_FAILED) => "không chép được triokey.exe vào Program Files (mã 3)".to_string(),
                Elevated::Exit(admin::TASK_FAILED) => "schtasks không tạo được tác vụ TrioKey (mã 4)".to_string(),
                Elevated::Exit(c) => format!("mã {c}"),
                _ => "không chạy được bản quyền admin".to_string(),
            };
            message(hwnd, &format!("Không {what} được chế độ admin: {why}. Cấu hình không đổi."), MB_OK | MB_ICONWARNING);
            return;
        }
    };
    let admin_exe = admin::admin_exe().map(|p| p.display().to_string()).unwrap_or_default();
    if on {
        // remember the choice for when admin mode is turned off; the task replaces the Run value
        let startup = autostart::is_on();
        if let Ok(exe) = std::env::current_exe() {
            autostart::set(false, &exe);
        }
        app::update(|s| {
            s.run_as_admin = true;
            s.run_at_startup = startup;
        });
        if admin::is_admin_copy() {
            return;
        }
        let q = format!(
            "Đã bật chế độ admin. Từ lần đăng nhập sau, TrioKey tự chạy quyền admin từ\n{admin_exe}\n\nChạy bản admin ngay bây giờ? Bản đang chạy sẽ thoát."
        );
        if message(hwnd, &q, MB_YESNO | MB_ICONQUESTION) == IDYES {
            if admin::start_task() {
                tray::quit();
            } else {
                message(hwnd, "Không chạy được tác vụ TrioKey. Bản admin sẽ chạy ở lần đăng nhập sau.", MB_OK | MB_ICONWARNING);
            }
        }
        return;
    }
    let s = app::settings();
    app::update(|s| s.run_as_admin = false);
    let later = match code {
        admin::OK_AFTER_REBOOT => " Thư mục sẽ được xoá sau khi khởi động lại máy.",
        admin::OK_FOLDER_LEFT => " Không xoá được thư mục Program Files\\TrioKey; hãy xoá tay bằng quyền admin.",
        _ => "",
    };
    if admin::is_admin_copy() {
        // this exe is about to go: Run must point at the normal copy, which only the user knows
        let t = format!(
            "Đã tắt chế độ admin và xoá tác vụ TrioKey.{later}\n\nBản admin này chạy tới khi bạn Thoát. Hãy Thoát, chạy lại triokey.exe bản thường, rồi bật lại \"Khởi động cùng Windows\" nếu cần."
        );
        message(hwnd, &t, MB_OK | MB_ICONINFORMATION);
        return;
    }
    // the task is gone: without the Run value nothing starts TrioKey at logon, so say so
    let restored = !s.run_at_startup || std::env::current_exe().is_ok_and(|exe| autostart::set(true, &exe));
    let warn =
        if restored { "" } else { "\n\nKhông ghi được mục tự khởi động trong registry: TrioKey sẽ không tự chạy khi đăng nhập." };
    let icon = if restored && code != admin::OK_FOLDER_LEFT { MB_ICONINFORMATION } else { MB_ICONWARNING };
    let gone = if code == admin::OK_FOLDER_LEFT { "tác vụ TrioKey" } else { &admin_exe };
    message(hwnd, &format!("Đã tắt chế độ admin và xoá {gone}.{later}{warn}"), MB_OK | icon);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            command(hwnd, (wp & 0xFFFF) as i32, ((wp >> 16) & 0xFFFF) as u32);
            warn_unsaved(hwnd);
            0
        }
        WM_TIMER => {
            latency(hwnd);
            0
        }
        // Moved to a monitor with another scale (or the scale changed). All state lives in
        // app::settings(), so making the controls again at the new size is less code than
        // moving and re-fonting each one.
        WM_DPICHANGED => {
            let dpi = (wp & 0xFFFF) as u32;
            let to = &*(lp as *const RECT);
            loop {
                let c = GetWindow(hwnd, GW_CHILD);
                if c.is_null() || DestroyWindow(c) == 0 {
                    break;
                }
            }
            build(hwnd, dpi);
            fill(hwnd);
            // the position Windows suggests, our own size for this DPI
            let (w, h) = window_size(dpi);
            SetWindowPos(hwnd, null_mut(), to.left, to.top, w, h, SWP_NOZORDER | SWP_NOACTIVATE);
            if GetActiveWindow() == hwnd {
                SetFocus(GetDlgItem(hwnd, ID_ENABLED));
            }
            0
        }
        // A plain window, not a dialog: nothing else remembers which control had the focus
        // and gives it back after Alt+Tab, and a focus left on the frame makes Space do nothing.
        WM_ACTIVATE => {
            let f = GetFocus();
            if (wp & 0xFFFF) as u32 == WA_INACTIVE && IsChild(hwnd, f) != 0 {
                FOCUS.store(f, Relaxed);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_SETFOCUS => {
            let f = FOCUS.load(Relaxed);
            SetFocus(if IsChild(hwnd, f) != 0 && IsWindowEnabled(f) != 0 { f } else { GetDlgItem(hwnd, ID_ENABLED) });
            0
        }
        WM_CTLCOLORSTATIC if GetDlgCtrlID(lp as HWND) == ID_LATENCY => {
            let dc = wp as HDC;
            SetTextColor(dc, GetSysColor(COLOR_GRAYTEXT));
            SetBkMode(dc, TRANSPARENT as i32);
            GetSysColorBrush(COLOR_BTNFACE) as LRESULT
        }
        WM_DESTROY => {
            KillTimer(hwnd, 1);
            HWND_SETTINGS.store(null_mut(), Relaxed);
            FOCUS.store(null_mut(), Relaxed);
            let f = FONT.swap(0, Relaxed);
            if f != 0 {
                DeleteObject(f as HFONT);
            }
            // not PostQuitMessage: the tray loop keeps running
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_scales_with_the_window_dpi() {
        assert_eq!([px(23, 96), px(23, 120), px(23, 144), px(23, 192)], [23, 28, 34, 46]);
    }
}
