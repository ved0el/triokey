//! Tray icon `V` (typing Vietnamese, red) / `E` (keys pass through, blue), shaded for the
//! taskbar theme; menu Tiếng Việt · Cài đặt… · Khởi động lại · Thoát (docs/ARCHITECTURE.md, 2.7 Tray menu).

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicUsize, Ordering::Relaxed};
use std::time::Duration;

use triokey_shared::glyph::{self, Glyph};
use triokey_shared::{english, t};
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, GetSystemMetricsForDpi, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::ime::{cjk_name, ImeState};
use crate::{admin, app, autostart, icon, ime_watch, settings_ui, update};

const WM_APP_TRAY: u32 = WM_APP + 3;
const WM_APP_STATE: u32 = WM_APP + 4;
const WM_APP_UPDATE: u32 = WM_APP + 5;
const ID_TOGGLE: usize = 1;
const ID_QUIT: usize = 2;
const ID_SETTINGS: usize = 3;
const ID_UPDATE: usize = 4;
const ID_RESTART: usize = 5;
/// Menu glyphs (Segoe Fluent Icons / Segoe MDL2 Assets code points): Settings, Download, Refresh, PowerButton.
const MENU_ICONS: [(usize, u16); 4] = [(ID_SETTINGS, 0xE713), (ID_UPDATE, 0xE896), (ID_RESTART, 0xE72C), (ID_QUIT, 0xE7E8)];
const ICON_FONTS: [&str; 2] = ["Segoe Fluent Icons", "Segoe MDL2 Assets"];
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
    // only while a newer version waits for a click
    if let Some(label) = update::menu_label(english()) {
        AppendMenuW(m, MF_STRING, ID_UPDATE, wide(&label).as_ptr());
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

/// Icon font at `n` pixels, or null when neither is installed: GDI quietly substitutes another face for a missing one,
/// so the face it selected is compared with the one asked for.
unsafe fn icon_font(dc: HDC, n: i32) -> HFONT {
    for face in ICON_FONTS {
        let mut lf: LOGFONTW = std::mem::zeroed();
        lf.lfHeight = -n;
        lf.lfCharSet = DEFAULT_CHARSET as _;
        // grey anti-aliasing, not ClearType: one coverage value per pixel
        lf.lfQuality = ANTIALIASED_QUALITY as _;
        for (d, s) in lf.lfFaceName.iter_mut().take(31).zip(face.encode_utf16()) {
            *d = s;
        }
        let f = CreateFontIndirectW(&lf);
        if f.is_null() {
            continue;
        }
        let old = SelectObject(dc, f);
        let mut got = [0u16; 64];
        let k = GetTextFaceW(dc, got.len() as i32, got.as_mut_ptr());
        SelectObject(dc, old);
        if k > 0 && wide_eq(got.as_ptr(), face) {
            return f;
        }
        DeleteObject(f);
    }
    null_mut()
}

/// One pixel of a menu glyph: the menu text colour (R, G, B) at `coverage`, as a premultiplied B, G, R, A pixel.
fn tint(coverage: u8, rgb: [u8; 3]) -> [u8; 4] {
    let m = |c: u8| ((c as u32 * coverage as u32 + 127) / 255) as u8;
    [m(rgb[2]), m(rgb[1]), m(rgb[0]), coverage]
}

/// `ch` drawn white on black into an `n`×`n` 32-bpp top-down DIB, its coverage turned into the alpha of `rgb`.
unsafe fn menu_bitmap(dc: HDC, font: HFONT, ch: u16, n: i32, rgb: [u8; 3]) -> HBITMAP {
    let mut bi: BITMAPINFO = std::mem::zeroed();
    bi.bmiHeader =
        BITMAPINFOHEADER { biSize: size_of::<BITMAPINFOHEADER>() as u32, biWidth: n, biHeight: -n, biPlanes: 1, biBitCount: 32, ..std::mem::zeroed() };
    let mut bits = null_mut();
    let bmp = CreateDIBSection(dc, &bi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    if bmp.is_null() || bits.is_null() {
        return null_mut();
    }
    let px = std::slice::from_raw_parts_mut(bits as *mut u8, (n * n * 4) as usize);
    px.fill(0);
    let (old_bmp, old_font) = (SelectObject(dc, bmp), SelectObject(dc, font));
    SetTextColor(dc, 0xFF_FFFF);
    SetBkMode(dc, TRANSPARENT as _);
    let mut rc = RECT { left: 0, top: 0, right: n, bottom: n };
    DrawTextW(dc, [ch].as_ptr(), 1, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    GdiFlush();
    SelectObject(dc, old_font);
    SelectObject(dc, old_bmp);
    for p in px.as_chunks_mut::<4>().0 {
        *p = tint(p[1], rgb);
    }
    bmp
}

/// Settings, the update item, Restart and Quit get a glyph in the menu text colour, at the check-mark size for the DPI
/// of the monitor the menu opens on; none when no icon font is installed. The bitmaps are freed after the menu closed.
unsafe fn menu_icons(m: HMENU, at: POINT) -> Vec<HBITMAP> {
    let (mut dpi, mut dpi_y) = (96, 96);
    if GetDpiForMonitor(MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST), MDT_EFFECTIVE_DPI, &mut dpi, &mut dpi_y) != 0 {
        dpi = 96;
    }
    let n = GetSystemMetricsForDpi(SM_CXMENUCHECK, dpi).max(8);
    let dc = CreateCompatibleDC(null_mut());
    let font = icon_font(dc, n);
    let mut out = Vec::new();
    if !font.is_null() {
        let c = GetSysColor(COLOR_MENUTEXT);
        let rgb = [c as u8, (c >> 8) as u8, (c >> 16) as u8];
        for (id, ch) in MENU_ICONS {
            let b = menu_bitmap(dc, font, ch, n, rgb);
            let mii = MENUITEMINFOW { cbSize: size_of::<MENUITEMINFOW>() as u32, fMask: MIIM_BITMAP, hbmpItem: b, ..std::mem::zeroed() };
            // refused for an item this menu does not have (no update waiting, no Restart)
            if !b.is_null() && SetMenuItemInfoW(m, id as u32, 0, &mii) == 0 {
                DeleteObject(b);
            } else if !b.is_null() {
                out.push(b);
            }
        }
        DeleteObject(font);
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

    #[test]
    fn menu_glyph_pixels_are_premultiplied() {
        let text = [0x1A, 0x80, 0xFF];
        assert_eq!(tint(0, text), [0; 4]);
        // B, G, R, A
        assert_eq!(tint(255, text), [0xFF, 0x80, 0x1A, 255]);
        assert_eq!(tint(128, text), [128, 64, 13, 128]);
        assert!((0..=255).all(|a| tint(a, [255; 3]) == [a, a, a, a]));
    }

    /// Off screen: a memory DC only. Where an icon font is installed every glyph has ink inside its box, and no colour
    /// channel exceeds the alpha (premultiplied); without one the menu has no icons, which is fine too. With
    /// `TRIOKEY_PREVIEW_DIR` set it also writes `menu-icons.png`: the four glyphs at 16 and 24 px, four times enlarged,
    /// dark ink on a light menu and light ink on a dark one.
    #[test]
    fn menu_glyphs_draw_with_an_icon_font() {
        const Z: usize = 4;
        let (cell, mut img) = (24 * Z + 8, Vec::new());
        unsafe {
            let dc = CreateCompatibleDC(null_mut());
            for n in [16, 24] {
                let font = icon_font(dc, n);
                if font.is_null() {
                    println!("no icon font: the menu shows no icons");
                    break;
                }
                for (ink_rgb, bg) in [([0x1A; 3], 0xF3u8), ([0xF2; 3], 0x2B)] {
                    let mut row = vec![bg; cell * cell * MENU_ICONS.len() * 3];
                    for (k, (_, ch)) in MENU_ICONS.into_iter().enumerate() {
                        let b = menu_bitmap(dc, font, ch, n, ink_rgb);
                        assert!(!b.is_null());
                        let mut px = vec![0u8; (n * n * 4) as usize];
                        let mut bi: BITMAPINFO = std::mem::zeroed();
                        bi.bmiHeader = BITMAPINFOHEADER {
                            biSize: size_of::<BITMAPINFOHEADER>() as u32,
                            biWidth: n,
                            biHeight: -n,
                            biPlanes: 1,
                            biBitCount: 32,
                            ..std::mem::zeroed()
                        };
                        assert_eq!(GetDIBits(dc, b, 0, n as u32, px.as_mut_ptr().cast(), &mut bi, DIB_RGB_COLORS), n);
                        DeleteObject(b);
                        let p = px.as_chunks::<4>().0;
                        let ink = p.iter().filter(|p| p[3] > 0).count();
                        assert!(ink > 8 && p.iter().all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]), "glyph {ch:04X} at {n}: {ink} pixels");
                        // premultiplied over the background, enlarged
                        for (i, q) in p.iter().enumerate() {
                            let (x, y) = (i % n as usize, i / n as usize);
                            let c = |v: u8| (v as u32 + bg as u32 * (255 - q[3] as u32) / 255) as u8;
                            for dy in 0..Z {
                                for dx in 0..Z {
                                    let at = ((4 + y * Z + dy) * cell * MENU_ICONS.len() + k * cell + 4 + x * Z + dx) * 3;
                                    row[at..at + 3].copy_from_slice(&[c(q[2]), c(q[1]), c(q[0])]);
                                }
                            }
                        }
                    }
                    img.extend(row);
                }
                DeleteObject(font);
            }
            DeleteDC(dc);
        }
        if let (Ok(dir), false) = (std::env::var("TRIOKEY_PREVIEW_DIR"), img.is_empty()) {
            let (w, h) = (cell * MENU_ICONS.len(), img.len() / 3 / (cell * MENU_ICONS.len()));
            std::fs::write(std::path::Path::new(&dir).join("menu-icons.png"), crate::icon::tests::png_rgb(w as u32, h as u32, &img)).unwrap();
        }
    }
}
