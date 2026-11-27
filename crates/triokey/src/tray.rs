//! Tray icon `V` (typing Vietnamese, red) / `E` (keys pass through, blue), shaded for the
//! taskbar theme; menu Tiếng Việt · Cài đặt… · Khởi động lại · Thoát (docs/ARCHITECTURE.md, 2.7 Tray menu).

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicUsize, Ordering::Relaxed};
use std::time::Duration;

use triokey_shared::glyph::{self, Glyph};
use triokey_shared::{english, t};
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemInformation::GetSystemWindowsDirectoryW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, GetSystemMetricsForDpi, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::ime::{cjk_name, ImeState};
use crate::{admin, app, autostart, icon, ime_watch, settings_ui, update};

const WM_APP_TRAY: u32 = WM_APP + 3;
const WM_APP_STATE: u32 = WM_APP + 4;
const WM_APP_UPDATE: u32 = WM_APP + 5;
const WM_APP_RESTART: u32 = WM_APP + 6;
const ID_TOGGLE: usize = 1;
const ID_QUIT: usize = 2;
const ID_SETTINGS: usize = 3;
const ID_UPDATE: usize = 4;
const ID_RESTART: usize = 5;
const ID_CHECK: usize = 6;
/// Menu item -> the system's own icon: file under the Windows folder and icon group resource id (docs/ARCHITECTURE.md,
/// 2.7 Tray menu). The ids are not documented; one that is missing leaves its item without an icon.
const MENU_ICONS: [(usize, SysIcon); 6] = [
    (ID_TOGGLE, (r"System32\shell32.dll", 16802)),                    // green check mark, no circle
    (ID_SETTINGS, (r"ImmersiveControlPanel\SystemSettings.exe", 10)), // the Settings app's gear
    (ID_UPDATE, (r"System32\imageres.dll", 184)),                     // download arrow
    (ID_CHECK, (r"System32\imageres.dll", 184)),                      // the same, in the update item's place
    (ID_RESTART, (r"System32\imageres.dll", 1401)),                   // circular arrows (Windows 11)
    (ID_QUIT, (r"System32\shell32.dll", 240)),                        // red X, no circle
];
/// The UAC shield, the same one `SHGetStockIconInfo(SIID_SHIELD)` names.
const ICON_UPDATE_ADMIN: SysIcon = (r"System32\imageres.dll", 78);
type SysIcon = (&'static str, u16);
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
        let old = slot.swap(icon::hicon(g, n, glyph::color(g, light)) as usize, Relaxed);
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

/// Updater thread: a new version was installed and the person paused, start it in place of this one.
pub fn restart() {
    let h = HWND_TRAY.load(Relaxed);
    if !h.is_null() {
        unsafe { PostMessageW(h, WM_APP_RESTART, 0, 0) };
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

fn tooltip(enabled: bool, state: ImeState, lang: u16, en: bool) -> String {
    match (enabled, state == ImeState::Yield, en) {
        (false, _, false) => "TrioKey: Tiếng Anh — đã tắt".into(),
        (false, _, true) => "TrioKey: English — off".into(),
        (true, true, false) => format!("TrioKey: Tiếng Anh — đang nhường IME {}", cjk_name(lang, en)),
        (true, true, true) => format!("TrioKey: English — yielding to the {} IME", cjk_name(lang, en)),
        (true, false, _) => triokey_shared::pick(en, "TrioKey: Tiếng Việt", "TrioKey: Vietnamese").into(),
    }
}

unsafe fn set_icon(hwnd: HWND, op: NOTIFY_ICON_MESSAGE) -> bool {
    let (on, state) = (app::enabled(), ime_watch::current());
    let i = if on && state == ImeState::Type { 0 } else { 1 };
    let tip = tooltip(on, state, ime_watch::lang(), english());
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
        // after an update; if the new copy does not start, this one keeps running and the new version waits for the next start
        WM_APP_RESTART => {
            if can_restart() && start_copy() {
                DestroyWindow(hwnd);
            }
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
    AppendMenuW(m, MF_STRING | check, ID_TOGGLE, wide(t("Tiếng Việt", "Vietnamese")).as_ptr());
    AppendMenuW(m, MF_STRING, ID_SETTINGS, wide(t("Cài đặt…", "Settings…")).as_ptr());
    // the waiting version while there is one, else a check (in a build that can update)
    if let Some(label) = update::menu_label(english()) {
        AppendMenuW(m, MF_STRING, ID_UPDATE, wide(&label).as_ptr());
    } else if update::available() {
        AppendMenuW(m, MF_STRING, ID_CHECK, wide(t("Kiểm tra bản mới", "Check for updates")).as_ptr());
    }
    AppendMenuW(m, MF_SEPARATOR, 0, null());
    if can_restart() {
        AppendMenuW(m, MF_STRING, ID_RESTART, wide(t("Khởi động lại", "Restart")).as_ptr());
    }
    AppendMenuW(m, MF_STRING, ID_QUIT, wide(t("Thoát", "Quit")).as_ptr());
    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    let icons = menu_icons(m, pt);
    // without these two the menu does not close when clicking elsewhere
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, null());
    PostMessageW(hwnd, WM_NULL, 0, 0);
    DestroyMenu(m);
    for b in icons {
        DeleteObject(b);
    }
    match cmd as usize {
        ID_TOGGLE => app::toggle(),
        ID_SETTINGS => settings_ui::open(),
        ID_UPDATE => update::click(),
        ID_CHECK => update::check_from_menu(),
        // the new copy waits up to 3 s for this one to let go of the mutex (`already_running`)
        ID_RESTART if start_copy() => {
            DestroyWindow(hwnd);
        }
        ID_RESTART => {
            let text = t(
                "Không khởi động lại được: bản mới của triokey.exe không chạy. TrioKey vẫn chạy tiếp.",
                "Could not restart: a new copy of triokey.exe did not start. TrioKey keeps running.",
            );
            MessageBoxW(null_mut(), wide(text).as_ptr(), wide("TrioKey").as_ptr(), MB_OK | MB_ICONWARNING);
        }
        ID_QUIT => {
            DestroyWindow(hwnd);
        }
        _ => {}
    }
}

/// Restart relaunches this exe by its path (docs/ARCHITECTURE.md, 8.3 What an elevated TrioKey must never do): fine
/// for a normal copy and for the admin copy, whose folder only admins can write. Any other elevated copy (Run as
/// administrator) may live in a folder unelevated programs can write, where the file could have been swapped since it
/// started: it offers no Restart.
fn can_restart() -> bool {
    !admin::is_elevated() || admin::is_admin_copy()
}

/// A new copy of this exe with the same arguments, through CreateProcessW (never ShellExecute); false when it did not start.
fn start_copy() -> bool {
    std::env::current_exe().and_then(|exe| std::process::Command::new(exe).args(std::env::args_os().skip(1)).spawn()).is_ok()
}

/// The system icon as an `n`×`n` premultiplied 32-bpp top-down DIB, the form a menu item bitmap needs to keep its
/// alpha; null when that file or id does not exist. `SHDefExtractIconW` (a negative index is a resource id) picks the
/// closest frame and scales it; the file is read as data by its full path under the Windows folder, nothing in it runs.
unsafe fn menu_bitmap(dc: HDC, (file, id): SysIcon, n: i32) -> HBITMAP {
    let mut dir = [0u16; 260];
    let k = GetSystemWindowsDirectoryW(dir.as_mut_ptr(), dir.len() as u32) as usize;
    if k == 0 || k >= dir.len() {
        return null_mut();
    }
    let path = wide(&format!("{}\\{file}", String::from_utf16_lossy(&dir[..k])));
    let mut icon: HICON = null_mut();
    if SHDefExtractIconW(path.as_ptr(), -(id as i32), 0, &mut icon, null_mut(), n as u32) != 0 || icon.is_null() {
        return null_mut();
    }
    let mut bi: BITMAPINFO = std::mem::zeroed();
    bi.bmiHeader =
        BITMAPINFOHEADER { biSize: size_of::<BITMAPINFOHEADER>() as u32, biWidth: n, biHeight: -n, biPlanes: 1, biBitCount: 32, ..std::mem::zeroed() };
    let mut bits = null_mut();
    let mut bmp = CreateDIBSection(dc, &bi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    if !bmp.is_null() && !bits.is_null() {
        // drawn onto transparent black, the icon's alpha blend leaves premultiplied pixels with its own alpha
        let px = std::slice::from_raw_parts_mut(bits as *mut u8, (n * n * 4) as usize);
        px.fill(0);
        let old = SelectObject(dc, bmp);
        DrawIconEx(dc, 0, 0, icon, n, n, 0, null_mut(), DI_NORMAL);
        GdiFlush();
        SelectObject(dc, old);
        // An icon without an alpha channel (older Windows has some under the same ids) comes out all transparent or as
        // an opaque square: no icon rather than a blank or a box.
        let alpha = px.as_chunks::<4>().0.iter().map(|p| p[3]);
        if !(alpha.clone().any(|a| a == 0) && alpha.clone().any(|a| a >= 128)) {
            DeleteObject(bmp);
            bmp = null_mut();
        }
    }
    DestroyIcon(icon);
    bmp
}

/// The toggle (while on), Settings, the update item, Restart and Quit get their system icon (`MENU_ICONS`) at the
/// small-icon size for the DPI of the monitor the menu opens on. The bitmaps are freed after the menu closed.
unsafe fn menu_icons(m: HMENU, at: POINT) -> Vec<HBITMAP> {
    let (mut dpi, mut dpi_y) = (96, 96);
    if GetDpiForMonitor(MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST), MDT_EFFECTIVE_DPI, &mut dpi, &mut dpi_y) != 0 {
        dpi = 96;
    }
    let n = GetSystemMetricsForDpi(SM_CXSMICON, dpi).max(16);
    let dc = CreateCompatibleDC(null_mut());
    let mut out = Vec::new();
    for (item, mut id) in MENU_ICONS {
        // the green check marks Vietnamese typing on; off, the toggle has no icon and no check mark
        if item == ID_TOGGLE && !app::enabled() {
            continue;
        }
        // an install that needs the admin's click gets the shield, as Windows marks such actions
        if item == ID_UPDATE && admin::is_elevated() {
            id = ICON_UPDATE_ADMIN;
        }
        let b = menu_bitmap(dc, id, n);
        let mii = MENUITEMINFOW { cbSize: size_of::<MENUITEMINFOW>() as u32, fMask: MIIM_BITMAP, hbmpItem: b, ..std::mem::zeroed() };
        // refused for an item this menu does not have (no update waiting, no Restart)
        if !b.is_null() && SetMenuItemInfoW(m, item as u32, 0, &mii) == 0 {
            DeleteObject(b);
        } else if !b.is_null() {
            out.push(b);
        }
    }
    DeleteDC(dc);
    out
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
        assert_eq!(tooltip(true, ImeState::Type, 0x11, false), "TrioKey: Tiếng Việt");
        assert_eq!(tooltip(false, ImeState::Yield, 0x11, false), "TrioKey: Tiếng Anh — đã tắt");
        assert_eq!(tooltip(true, ImeState::Yield, 0x11, false), "TrioKey: Tiếng Anh — đang nhường IME Nhật");
        assert_eq!(tooltip(true, ImeState::Yield, 0x04, false), "TrioKey: Tiếng Anh — đang nhường IME Trung");
        assert_eq!(tooltip(true, ImeState::Yield, 0x12, false), "TrioKey: Tiếng Anh — đang nhường IME Hàn");
        // before the first read
        assert_eq!(tooltip(true, ImeState::Yield, 0, false), "TrioKey: Tiếng Anh — đang nhường IME CJK");
        assert_eq!(tooltip(true, ImeState::Type, 0x11, true), "TrioKey: Vietnamese");
        assert_eq!(tooltip(false, ImeState::Type, 0x11, true), "TrioKey: English — off");
        assert_eq!(tooltip(true, ImeState::Yield, 0x11, true), "TrioKey: English — yielding to the Japanese IME");
        assert_eq!(tooltip(true, ImeState::Yield, 0, true), "TrioKey: English — yielding to the CJK IME");
    }

    /// The system menu icons load at the sizes 100 %..200 % ask for, as premultiplied pixels with real transparency. The
    /// Windows 11 ones may be missing or without alpha on another Windows (CI runs Windows Server): `menu_bitmap` gives
    /// none for those and they are skipped; the shield is never skipped.
    #[test]
    fn menu_icons_load_with_alpha() {
        unsafe {
            let dc = CreateCompatibleDC(null_mut());
            assert!(!menu_bitmap(dc, ICON_UPDATE_ADMIN, 16).is_null(), "the shield is on every Windows");
            assert!(menu_bitmap(dc, (r"System32\imageres.dll", 65000), 16).is_null(), "a missing id gives no icon");
            for id in MENU_ICONS.map(|(_, id)| id).into_iter().chain([ICON_UPDATE_ADMIN]) {
                for n in [16, 20, 24, 32] {
                    let b = menu_bitmap(dc, id, n);
                    if b.is_null() {
                        continue;
                    }
                    let mut px = vec![0u8; (n * n * 4) as usize];
                    let mut bm: BITMAP = std::mem::zeroed();
                    GetObjectW(b, size_of::<BITMAP>() as i32, (&mut bm as *mut BITMAP).cast());
                    std::ptr::copy_nonoverlapping(bm.bmBits as *const u8, px.as_mut_ptr(), px.len());
                    let p = px.as_chunks::<4>().0;
                    assert!(p.iter().any(|p| p[3] == 0) && p.iter().any(|p| p[3] >= 128), "icon {id:?} at {n}: no transparency or no ink");
                    assert!(p.iter().all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]), "icon {id:?} at {n}: not premultiplied");
                    DeleteObject(b);
                }
            }
            DeleteDC(dc);
        }
    }
}
