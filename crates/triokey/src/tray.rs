//! Tray icon `V` (typing Vietnamese, red) / `E` (keys pass through, blue), shaded for the
//! taskbar theme; menu Tiếng Việt · Cài đặt… · Thoát.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicUsize, Ordering::Relaxed};
use std::time::Duration;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::icon::{self, Glyph};
use crate::ime::{cjk_name, ImeState};
use crate::{app, autostart, ime_watch, settings_ui, update};

const WM_APP_TRAY: u32 = WM_APP + 3;
const WM_APP_STATE: u32 = WM_APP + 4;
const WM_APP_UPDATE: u32 = WM_APP + 5;
const ID_TOGGLE: usize = 1;
const ID_QUIT: usize = 2;
const ID_SETTINGS: usize = 3;
const ID_UPDATE: usize = 4;
/// Timer: the icon could not be shown, try again.
const RETRY: usize = 1;
const RETRY_MS: u32 = 2000;
const CLASS: &str = "triokey-tray";
/// Per session. Created by the elevated copy it denies the normal copy access, which
/// counts as "running" too; created by the normal copy the elevated one can open it.
const MUTEX: &str = r"Local\triokey-single-instance";
const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

static HWND_TRAY: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
/// V, E as HICON values for the current taskbar theme (raw pointers are not `Sync`).
static ICONS: [AtomicUsize; 2] = [AtomicUsize::new(0), AtomicUsize::new(0)];

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

/// Another TrioKey (normal or admin copy) holds the mutex. Retried for 3 s: the copy that hands
/// over to the admin copy ("chạy ngay") quits right after starting it.
pub fn already_running() -> bool {
    for _ in 0..15 {
        unsafe {
            let h = CreateMutexW(null(), 0, wide(MUTEX).as_ptr());
            let exists = GetLastError() == ERROR_ALREADY_EXISTS;
            if !h.is_null() && !exists {
                // held until the process exits
                return false;
            }
            if !h.is_null() {
                CloseHandle(h);
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    true
}

/// Tray thread: end the message loop, which ends TrioKey.
pub fn quit() {
    let h = HWND_TRAY.load(Relaxed);
    if !h.is_null() {
        unsafe { DestroyWindow(h) };
    }
}

/// Taskbar theme: deep colours on a light taskbar, light ones on dark. Missing value: dark
/// (Windows 10).
fn light_taskbar() -> bool {
    autostart::hkcu_dword(PERSONALIZE, "SystemUsesLightTheme") == Some(1)
}

unsafe fn load_icons() {
    let (n, light) = (GetSystemMetrics(SM_CXSMICON), light_taskbar());
    for (slot, g) in ICONS.iter().zip([Glyph::V, Glyph::E]) {
        let old = slot.swap(icon::hicon(g, n, icon::color(g, light)) as usize, Relaxed);
        if old != 0 {
            DestroyIcon(old as HICON);
        }
    }
}

/// Any thread: the state changed, redraw the icon.
pub fn notify() {
    let h = HWND_TRAY.load(Relaxed);
    if !h.is_null() {
        unsafe { PostMessageW(h, WM_APP_STATE, 0, 0) };
    }
}

/// Updater thread: there is a notification to show or a status line to refresh.
pub fn notify_update() {
    let h = HWND_TRAY.load(Relaxed);
    if !h.is_null() {
        unsafe { PostMessageW(h, WM_APP_UPDATE, 0, 0) };
    }
}

pub fn run() {
    unsafe {
        let hinst = GetModuleHandleW(null());
        let class = wide(CLASS);
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst, lpszClassName: class.as_ptr(), ..std::mem::zeroed() };
        RegisterClassW(&wc);
        TASKBAR_CREATED.store(RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()), Relaxed);
        let hwnd = CreateWindowExW(0, class.as_ptr(), wide("TrioKey").as_ptr(), WS_OVERLAPPED, 0, 0, 0, 0, null_mut(), null_mut(), hinst, null());
        if hwnd.is_null() {
            return;
        }
        // The admin copy runs at high integrity and Explorer broadcasts this at medium: without
        // the filter UIPI drops it and the icon is gone for good after an Explorer restart.
        ChangeWindowMessageFilterEx(hwnd, TASKBAR_CREATED.load(Relaxed), MSGFLT_ALLOW, null_mut());
        load_icons();
        HWND_TRAY.store(hwnd, Relaxed);
        show(hwnd);
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            // Tab / arrow keys between the settings controls
            let dlg = settings_ui::hwnd();
            if !dlg.is_null() && IsDialogMessageW(dlg, &msg) != 0 {
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Show the current icon and tooltip. Modify first: an add that timed out may still have gone
/// through, and adding twice fails. While both fail (no taskbar yet at logon, Explorer busy
/// or restarting) a timer keeps trying, so TrioKey never runs on without its menu.
unsafe fn show(hwnd: HWND) {
    if set_icon(hwnd, NIM_MODIFY) || set_icon(hwnd, NIM_ADD) {
        KillTimer(hwnd, RETRY);
    } else {
        SetTimer(hwnd, RETRY, RETRY_MS, None);
    }
}

fn tooltip(enabled: bool, state: ImeState, lang: u16) -> String {
    if !enabled {
        "TrioKey: Tiếng Anh — đã tắt".into()
    } else if state == ImeState::Yield {
        format!("TrioKey: Tiếng Anh — đang nhường IME {}", cjk_name(lang))
    } else {
        "TrioKey: Tiếng Việt".into()
    }
}

unsafe fn set_icon(hwnd: HWND, op: NOTIFY_ICON_MESSAGE) -> bool {
    let (on, state) = (app::enabled(), ime_watch::current());
    let i = if on && state == ImeState::Type { 0 } else { 1 };
    let tip = tooltip(on, state, ime_watch::lang());
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_APP_TRAY;
    nid.hIcon = ICONS[i].load(Relaxed) as HICON;
    for (d, s) in nid.szTip.iter_mut().zip(tip.encode_utf16()) {
        *d = s;
    }
    Shell_NotifyIconW(op, &nid) != 0
}

/// One notification from the updater. Sent once, apart from `show`: with NIF_INFO in every
/// icon update it would pop up again at each state change.
unsafe fn balloon(hwnd: HWND, title: &str, text: &str) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = NIIF_INFO;
    // one element short: the zeroed last one stays as the terminator
    for (d, s) in nid.szInfo.iter_mut().take(255).zip(text.encode_utf16()) {
        *d = s;
    }
    for (d, s) in nid.szInfoTitle.iter_mut().take(63).zip(title.encode_utf16()) {
        *d = s;
    }
    Shell_NotifyIconW(NIM_MODIFY, &nid);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_APP_TRAY => {
            match lp as u32 {
                WM_LBUTTONUP => app::toggle(),
                WM_RBUTTONUP => menu(hwnd),
                NIN_BALLOONUSERCLICK => update::click(),
                _ => {}
            }
            0
        }
        WM_APP_STATE => {
            show(hwnd);
            settings_ui::sync_enabled();
            0
        }
        WM_APP_UPDATE => {
            if let Some((title, text)) = update::take_balloon() {
                balloon(hwnd, &title, &text);
            }
            settings_ui::sync_update();
            0
        }
        WM_TIMER if wp == RETRY => {
            show(hwnd);
            0
        }
        WM_SETTINGCHANGE if lp != 0 && wide_eq(lp as *const u16, "ImmersiveColorSet") => {
            load_icons();
            show(hwnd);
            0
        }
        // logoff or shutdown: the process is killed after this returns, before `run` gets back
        WM_ENDSESSION => {
            if wp != 0 {
                app::save_learned(true);
            }
            0
        }
        WM_DESTROY => {
            // while the window still exists: Explorer may ignore a delete for a dead handle
            HWND_TRAY.store(null_mut(), Relaxed);
            KillTimer(hwnd, RETRY);
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            Shell_NotifyIconW(NIM_DELETE, &nid);
            PostQuitMessage(0);
            0
        }
        // Explorer restarted: the icon is gone, add it again
        m if m != 0 && m == TASKBAR_CREATED.load(Relaxed) => {
            show(hwnd);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe fn menu(hwnd: HWND) {
    let m = CreatePopupMenu();
    let check = if app::enabled() { MF_CHECKED } else { 0 };
    AppendMenuW(m, MF_STRING | check, ID_TOGGLE, wide("Tiếng Việt").as_ptr());
    AppendMenuW(m, MF_STRING, ID_SETTINGS, wide("Cài đặt…").as_ptr());
    // only while a newer version waits for a click
    if let Some(label) = update::menu_label() {
        AppendMenuW(m, MF_STRING, ID_UPDATE, wide(&label).as_ptr());
    }
    AppendMenuW(m, MF_SEPARATOR, 0, null());
    AppendMenuW(m, MF_STRING, ID_QUIT, wide("Thoát").as_ptr());
    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    // without these two the menu does not close when clicking elsewhere
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, null());
    PostMessageW(hwnd, WM_NULL, 0, 0);
    DestroyMenu(m);
    match cmd as usize {
        ID_TOGGLE => app::toggle(),
        ID_SETTINGS => settings_ui::open(),
        ID_UPDATE => update::click(),
        ID_QUIT => {
            DestroyWindow(hwnd);
        }
        _ => {}
    }
}

/// The NUL-terminated wide string at `p` equals `s`.
pub(crate) unsafe fn wide_eq(p: *const u16, s: &str) -> bool {
    let mut i = 0;
    for c in s.encode_utf16() {
        if *p.add(i) != c {
            return false;
        }
        i += 1;
    }
    *p.add(i) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_names_the_ime_language() {
        assert_eq!(tooltip(true, ImeState::Type, 0x11), "TrioKey: Tiếng Việt");
        assert_eq!(tooltip(false, ImeState::Yield, 0x11), "TrioKey: Tiếng Anh — đã tắt");
        assert_eq!(tooltip(true, ImeState::Yield, 0x11), "TrioKey: Tiếng Anh — đang nhường IME Nhật");
        assert_eq!(tooltip(true, ImeState::Yield, 0x04), "TrioKey: Tiếng Anh — đang nhường IME Trung");
        assert_eq!(tooltip(true, ImeState::Yield, 0x12), "TrioKey: Tiếng Anh — đang nhường IME Hàn");
        // before the first read
        assert_eq!(tooltip(true, ImeState::Yield, 0), "TrioKey: Tiếng Anh — đang nhường IME CJK");
    }
}
