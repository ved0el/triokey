//! Settings window, plain Win32 controls on the tray thread. Every change is saved
//! to config.txt and applied at once. One window at a time. Look, layout and the rebuild path:
//! docs/ARCHITECTURE.md, 2.6 Settings window.
//!
//! The embedded manifest (build.rs) gives the controls the current visual style and makes the
//! process per-monitor DPI aware: sizes come from the window's own DPI, never the system's.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, AtomicUsize, Ordering::Relaxed};

use windows_sys::Win32::Foundation::{GlobalFree, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Dwm::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows_sys::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOW};
use windows_sys::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows_sys::Win32::UI::Controls::{
    CloseThemeData, DrawThemeBackground, DrawThemeTextEx, GetThemePartSize, InitCommonControlsEx, SetWindowTheme, BP_CHECKBOX, BP_PUSHBUTTON, BP_RADIOBUTTON,
    CDDS_PREPAINT, CDIS_FOCUS, CDIS_HOT, CDIS_SELECTED, CDRF_DODEFAULT, CDRF_SKIPDEFAULT, DTTOPTS, DTT_TEXTCOLOR, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX,
    NMCUSTOMDRAW, NMHDR, NM_CUSTOMDRAW, PBS_DISABLED, TS_DRAW,
};
use windows_sys::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow, GetSystemMetricsForDpi, OpenThemeDataForDpi, SystemParametersInfoForDpi};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, GetActiveWindow, GetFocus, IsWindowEnabled, SetFocus};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use triokey_shared::glyph::{BLUE_DEEP, BLUE_LIGHT};
use triokey_shared::pick;

use crate::admin::{self, Elevated};
use crate::config::{render, Hotkey, HOTKEYS};
use crate::tray::{wide, wide_eq};
use crate::{app, autostart, hook, ime_watch, tray, update};

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
const ID_DEBUG: i32 = 110;
const ID_AUTO_UPDATE: i32 = 111;
const ID_CHECK: i32 = 112;
const ID_UPDATE_STATUS: i32 = 113;
const ID_DEV: i32 = 114;
const ID_FOOTER: i32 = 115;
const ID_UPDATE_LINK: i32 = 116;
const ID_GITHUB: i32 = 117;
const ID_LANG: i32 = 118;
/// The footer: `triokey_shared::footer` with this crate's version, and the GitHub link.
const REPO: &str = triokey_shared::REPOSITORY;
/// Posted by the developer-mode check box (its own click handler must not destroy it), by `sync_update` and, with
/// wParam 1, by a theme change, which can arrive inside a click handler's message box.
const WM_APP_REBUILD: u32 = WM_APP + 1;
const SS_RIGHT: u32 = 0x2;
const SS_NOPREFIX: u32 = 0x80;
/// Clipboard format of UTF-16 text (Win32_System_Ole, not otherwise needed).
const CF_UNICODETEXT: u32 = 13;
const BST_CHECKED: isize = 1;
const STYLE: u32 = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
/// Client width in 96-DPI pixels; the height follows the content (`build`).
const WIDTH: i32 = 400;

/// Colours as COLORREF (0x00BBGGRR), the only source of colour while a palette is in use. `bg`
/// is the client area and the caption alike, so header and body match by construction.
struct Palette {
    bg: u32,
    text: u32,
    /// secondary text: the footer
    dim: u32,
    /// disabled check boxes and radio buttons
    off: u32,
    /// the hairline and the window border
    line: u32,
    /// link text: the tray icon's deep blue (light) or light blue (dark)
    link: u32,
    dark: bool,
}
const fn colorref(c: [u8; 3]) -> u32 {
    c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16
}
static LIGHT: Palette = Palette { bg: 0xF3F3F3, text: 0x1A1A1A, dim: 0x5C5C5C, off: 0xA0A0A0, line: 0xD0D0D0, link: colorref(BLUE_DEEP), dark: false };
static DARK: Palette = Palette { bg: 0x202020, text: 0xF2F2F2, dim: 0xABABAB, off: 0x6E6E6E, line: 0x3D3D3D, link: colorref(BLUE_LIGHT), dark: true };
/// The palette in use; null is the system look (high contrast).
static PALETTE: AtomicPtr<Palette> = AtomicPtr::new(null_mut());
/// Solid brush of the palette's `bg`, handed to the controls in WM_CTLCOLOR*; 0 in the system look.
static BRUSH: AtomicUsize = AtomicUsize::new(0);

static HWND_SETTINGS: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
/// The message font and its semibold variant (section headings).
static FONT: AtomicUsize = AtomicUsize::new(0);
static BOLD: AtomicUsize = AtomicUsize::new(0);
/// `Status::key` of the update status the window was built for.
static SHAPE: AtomicU32 = AtomicU32::new(0);
/// The DPI the content was laid out for, and the hairline's top in client pixels.
static DPI: AtomicU32 = AtomicU32::new(96);
static LINE_Y: AtomicI32 = AtomicI32::new(0);
/// ID of the control that had the keyboard focus when the window was last deactivated (an ID,
/// not a handle: a rebuild replaces the controls).
static FOCUS: AtomicI32 = AtomicI32::new(0);
/// The language the window was built in: `triokey_shared::english()` for a user, a parameter for the preview test.
static ENGLISH: AtomicBool = AtomicBool::new(false);

pub fn hwnd() -> HWND {
    HWND_SETTINGS.load(Relaxed)
}

/// `vi` or `en` in the window's language; every text of the window and its message boxes goes through this.
fn tr(vi: &'static str, en: &'static str) -> &'static str {
    pick(ENGLISH.load(Relaxed), vi, en)
}

fn update_link() -> &'static str {
    tr("Cập nhật tại đây", "Update here")
}

/// Alt+Z is the same key on every keyboard; only "none" has words.
fn hotkey_label(h: Hotkey) -> &'static str {
    match h {
        Hotkey::CtrlShift => "Ctrl+Shift",
        Hotkey::AltZ => "Alt+Z",
        Hotkey::Off => tr("Không dùng", "None"),
    }
}

/// The language the language list shows now (its choice before the posted rebuild has run).
unsafe fn chosen_english(hwnd: HWND) -> bool {
    SendMessageW(GetDlgItem(hwnd, ID_LANG), CB_GETCURSEL, 0, 0) == 1
}

fn pal() -> Option<&'static Palette> {
    unsafe { PALETTE.load(Relaxed).as_ref() }
}

/// What Windows asks for now: the system look under high contrast, else the apps' light or dark mode.
fn wanted() -> Option<&'static Palette> {
    let mut hc = HIGHCONTRASTW { cbSize: size_of::<HIGHCONTRASTW>() as u32, dwFlags: 0, lpszDefaultScheme: null_mut() };
    let high = unsafe { SystemParametersInfoW(SPI_GETHIGHCONTRAST, hc.cbSize, (&mut hc as *mut HIGHCONTRASTW).cast(), 0) } != 0
        && hc.dwFlags & HCF_HIGHCONTRASTON != 0;
    match (high, apps_dark()) {
        (true, _) => None,
        (_, true) => Some(&DARK),
        _ => Some(&LIGHT),
    }
}

/// Windows apps are set to dark mode (Settings > Personalization > Colors).
fn apps_dark() -> bool {
    let (mut v, mut n) = (1u32, 4u32);
    let key = wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let r = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, key.as_ptr(), wide("AppsUseLightTheme").as_ptr(), RRF_RT_REG_DWORD, null_mut(), (&mut v as *mut u32).cast(), &mut n)
    };
    r == 0 && v == 0
}

/// Tray thread: the on/off switch changed (hotkey, tray click) while the window may be open.
pub fn sync_enabled() {
    let h = hwnd();
    if !h.is_null() {
        unsafe { set_check(h, ID_ENABLED, app::enabled()) };
    }
}

/// Tray thread: the updater has something new to say while the window may be open. A status that
/// needs other room (lines, the link or its place) rebuilds the window, posted: this may run inside
/// a click handler.
pub fn sync_update() {
    let h = hwnd();
    if !h.is_null() {
        unsafe {
            let s = status(h, DPI.load(Relaxed));
            if s.key() != SHAPE.load(Relaxed) {
                PostMessageW(h, WM_APP_REBUILD, 0, 0);
            } else {
                SetWindowTextW(GetDlgItem(h, ID_UPDATE_STATUS), wide(&s.text).as_ptr());
            }
        }
    }
}

/// The update status laid out in the 378 px column at `dpi` with the window's font.
struct Status {
    text: String,
    /// lines it takes (at least one, kept also when empty), the link's own line included, and one line's height in pixels
    lines: i32,
    line: i32,
    /// while a version waits: the link's left edge from the column's (pixels) and its line
    link: Option<(i32, i32)>,
}

impl Status {
    fn key(&self) -> u32 {
        self.lines as u32 | self.link.map_or(0, |(x, l)| (l as u32 + 1) << 4 | (x as u32) << 8)
    }
}

/// The link goes after the text when both fit on one line, else on a line of its own.
unsafe fn status(hwnd: HWND, dpi: u32) -> Status {
    let text = update::status(ENGLISH.load(Relaxed));
    let dc = GetDC(hwnd);
    let old = SelectObject(dc, FONT.load(Relaxed) as HGDIOBJ);
    let mut one = std::mem::zeroed::<RECT>();
    DrawTextW(dc, wide("Ág").as_ptr(), -1, &mut one, DT_CALCRECT | DT_SINGLELINE);
    let line = one.bottom.max(1);
    let mut rc = RECT { left: 0, top: 0, right: px(378, dpi), bottom: 0 };
    if !text.is_empty() {
        DrawTextW(dc, wide(&text).as_ptr(), -1, &mut rc, DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX);
    }
    SelectObject(dc, old);
    ReleaseDC(hwnd, dc);
    // one line is always reserved: the ordinary answers of `Kiểm tra bản mới` change no size
    let mut lines = ((rc.bottom + line - 1) / line).max(1);
    let link = update::waiting().map(|_| {
        let x = if text.is_empty() { 0 } else { text_width(hwnd, &text) + text_width(hwnd, " ") };
        if lines == 1 && x + text_width(hwnd, update_link()) + 2 <= px(378, dpi) {
            (x, 0)
        } else {
            lines += 1;
            (0, lines - 1)
        }
    });
    Status { text, lines, line, link }
}

/// An https address in the browser. An elevated TrioKey never calls ShellExecute (docs/ARCHITECTURE.md,
/// 8.3 What an elevated TrioKey must never do): an unelevated program can redirect "open", so it shows the address.
unsafe fn open_url(hwnd: HWND, url: &str) {
    if admin::is_elevated() {
        let t = tr(
            "TrioKey đang chạy quyền admin nên không tự mở trình duyệt. Địa chỉ:",
            "TrioKey runs as administrator, so it does not open the browser itself. Address:",
        );
        message(hwnd, &format!("{t}\n\n{url}"), MB_OK | MB_ICONINFORMATION);
    } else if url.starts_with("https://") {
        ShellExecuteW(hwnd, wide("open").as_ptr(), wide(url).as_ptr(), null(), null(), SW_SHOWNORMAL);
    }
}

/// 96-DPI pixels to pixels at `dpi`.
fn px(v: i32, dpi: u32) -> i32 {
    v * dpi as i32 / 96
}

/// Outer window size for a client area `height` 96-DPI pixels tall at `dpi`.
unsafe fn window_size(dpi: u32, height: i32) -> (i32, i32) {
    let mut rc = RECT { left: 0, top: 0, right: px(WIDTH, dpi), bottom: px(height, dpi) };
    AdjustWindowRectExForDpi(&mut rc, STYLE, 0, 0, dpi);
    (rc.right - rc.left, rc.bottom - rc.top)
}

unsafe fn top_left(hwnd: HWND) -> (i32, i32) {
    let mut rc = std::mem::zeroed::<RECT>();
    GetWindowRect(hwnd, &mut rc);
    (rc.left, rc.top)
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
        let hwnd = create(wanted(), app::settings().dev_mode, triokey_shared::english(), None, null_mut());
        if hwnd.is_null() {
            return;
        }
        let mut rc = std::mem::zeroed::<RECT>();
        GetWindowRect(hwnd, &mut rc);
        let (x, y) = ((GetSystemMetrics(SM_CXSCREEN) - rc.right + rc.left) / 2, (GetSystemMetrics(SM_CYSCREEN) - rc.bottom + rc.top) / 2);
        SetWindowPos(hwnd, null_mut(), x, y, 0, 0, SWP_NOZORDER | SWP_NOSIZE);
        // activation ends in WM_SETFOCUS, which puts the focus on the first control
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        if app::config_unreadable() {
            let t = tr(
                "Không đọc được config.txt (file phải là UTF-8). TrioKey đang dùng giá trị mặc định và giữ nguyên file đó: thay đổi ở đây chỉ có hiệu lực tới khi Thoát, không được lưu.\n\nSửa hoặc xoá config.txt rồi chạy lại TrioKey.",
                "config.txt could not be read (it must be UTF-8). TrioKey uses the defaults and leaves that file alone: changes made here last until you Quit and are not saved.\n\nFix or delete config.txt, then start TrioKey again.",
            );
            message(hwnd, t, MB_OK | MB_ICONWARNING);
        }
        warn_unsaved(hwnd);
    }
}

/// The window, hidden, filled and sized for `dpi` (default: its monitor's) in `theme` (None: the
/// system look), with the developer part when `dev`, in English when `en`. Shows nothing and asks nothing: `open` does
/// that. `theme`, `dev`, `en` and `owner` are parameters for the preview test.
unsafe fn create(theme: Option<&'static Palette>, dev: bool, en: bool, dpi: Option<u32>, owner: HWND) -> HWND {
    // makes sure comctl32 (version 6, chosen by the manifest) is loaded before the controls
    InitCommonControlsEx(&INITCOMMONCONTROLSEX { dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32, dwICC: ICC_STANDARD_CLASSES });
    let hinst = GetModuleHandleW(null());
    let class = wide(CLASS);
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: hinst,
        lpszClassName: class.as_ptr(),
        hCursor: LoadCursorW(null_mut(), IDC_ARROW),
        // the system look's background; a palette paints its own in WM_ERASEBKGND
        hbrBackground: (COLOR_BTNFACE + 1) as usize as HBRUSH,
        ..std::mem::zeroed()
    };
    // fails harmlessly when already registered
    RegisterClassW(&wc);
    // created empty at the primary monitor's origin, so its DPI is that monitor's
    let hwnd = CreateWindowExW(0, class.as_ptr(), null(), STYLE, 0, 0, 0, 0, owner, null_mut(), hinst, null());
    if hwnd.is_null() {
        return hwnd;
    }
    HWND_SETTINGS.store(hwnd, Relaxed);
    let dpi = dpi.unwrap_or(match GetDpiForWindow(hwnd) {
        0 => 96,
        d => d,
    });
    rebuild(hwnd, dpi, theme, dev, en, (0, 0));
    hwnd
}

/// The one path that makes the content: at creation, after a DPI change, a theme change, the
/// developer-mode switch and the language switch. All state lives in app::settings(), so making the controls again is less
/// code than moving, re-fonting and recolouring each one. The window is resized to the content
/// with its top-left at `at`; the focused control keeps the focus.
unsafe fn rebuild(hwnd: HWND, dpi: u32, theme: Option<&'static Palette>, dev: bool, en: bool, at: (i32, i32)) {
    let f = GetFocus();
    let focus = if IsChild(hwnd, f) != 0 { GetDlgCtrlID(f) } else { 0 };
    loop {
        let c = GetWindow(hwnd, GW_CHILD);
        if c.is_null() || DestroyWindow(c) == 0 {
            break;
        }
    }
    PALETTE.store(theme.map_or(null_mut(), |p| (p as *const Palette).cast_mut()), Relaxed);
    let brush = theme.map_or(null_mut(), |p| CreateSolidBrush(p.bg));
    let old = BRUSH.swap(brush as usize, Relaxed);
    if old != 0 {
        DeleteObject(old as HGDIOBJ);
    }
    DPI.store(dpi, Relaxed);
    ENGLISH.store(en, Relaxed);
    SetWindowTextW(hwnd, wide(tr("TrioKey – Cài đặt", "TrioKey – Settings")).as_ptr());
    caption(hwnd);
    let height = build(hwnd, dpi, dev);
    fill(hwnd);
    set_icons(hwnd, dpi);
    let (w, h) = window_size(dpi, height);
    SetWindowPos(hwnd, null_mut(), at.0, at.1, w, h, SWP_NOZORDER | SWP_NOACTIVATE);
    InvalidateRect(hwnd, null(), 1);
    if GetActiveWindow() == hwnd {
        let c = if focus != 0 { GetDlgItem(hwnd, focus) } else { null_mut() };
        SetFocus(if c.is_null() { GetDlgItem(hwnd, ID_ENABLED) } else { c });
    }
}

/// Caption, border and caption text from the palette: the caption is the client area's colour,
/// the border the hairline's, so the edge stays visible on a desktop of the same colour. The
/// system look resets all three. Returns each call's HRESULT (dark mode, caption, border, text);
/// a refusal is ignored (Windows 10 does not know 34-36).
unsafe fn caption(hwnd: HWND) -> [i32; 4] {
    let d = DWMWA_COLOR_DEFAULT;
    let (dark, bg, line, text) = pal().map_or((0, d, d, d), |p| (p.dark as u32, p.bg, p.line, p.text));
    let set = |a: i32, v: u32| DwmSetWindowAttribute(hwnd, a as u32, (&v as *const u32).cast(), 4);
    [set(DWMWA_USE_IMMERSIVE_DARK_MODE, dark), set(DWMWA_CAPTION_COLOR, bg), set(DWMWA_BORDER_COLOR, line), set(DWMWA_TEXT_COLOR, text)]
}

/// The app icon (resource 1, build.rs) at the sizes this window's DPI wants, replacing and
/// destroying the ones set before. `dpi` 0 only removes them (window closing).
unsafe fn set_icons(hwnd: HWND, dpi: u32) {
    let hinst = GetModuleHandleW(null());
    for (kind, metric) in [(ICON_SMALL, SM_CXSMICON), (ICON_BIG, SM_CXICON)] {
        let new = match dpi {
            0 => null_mut(),
            _ => {
                let n = GetSystemMetricsForDpi(metric, dpi);
                LoadImageW(hinst, 1 as _, IMAGE_ICON, n, n, LR_DEFAULTCOLOR)
            }
        };
        let old = SendMessageW(hwnd, WM_SETICON, kind as usize, new as isize);
        if old != 0 {
            DestroyIcon(old as HICON);
        }
    }
}

/// config.txt could not be written (at startup or by the last change): never silently.
unsafe fn warn_unsaved(hwnd: HWND) {
    if app::take_save_failed() {
        let t = tr(
            "Không lưu được config.txt: thay đổi chỉ có hiệu lực tới khi Thoát.\n\nBản chạy quyền admin chỉ ghi lại file config.txt đã có sẵn (không tạo mới, không ghi qua liên kết). Nếu chưa có file, hãy chạy TrioKey bản thường một lần.",
            "config.txt could not be saved: changes last until you Quit.\n\nThe administrator copy only rewrites a config.txt that already exists (it creates none and writes through no link). If there is no file yet, run the normal TrioKey once.",
        );
        message(hwnd, t, MB_OK | MB_ICONWARNING);
    }
}

/// Create the controls for `dpi`, top to bottom (the tab order), and return the client height in
/// 96-DPI pixels. Windows dialog metrics for Segoe UI 9 pt: 11 px margins, 17 px check boxes,
/// 23 px buttons and combo boxes, 7 px between related rows, 15 px before a section heading.
unsafe fn build(hwnd: HWND, dpi: u32, dev: bool) -> i32 {
    let mut ncm: NONCLIENTMETRICSW = std::mem::zeroed();
    ncm.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    // the system message font (Segoe UI 9 pt) at this window's DPI, and the same in semibold
    let (font, bold) = if SystemParametersInfoForDpi(SPI_GETNONCLIENTMETRICS, ncm.cbSize, (&mut ncm as *mut NONCLIENTMETRICSW).cast(), 0, dpi) != 0 {
        let mut lf = ncm.lfMessageFont;
        let font = CreateFontIndirectW(&lf);
        lf.lfWeight = FW_SEMIBOLD as i32;
        (font, CreateFontIndirectW(&lf))
    } else {
        let f = GetStockObject(DEFAULT_GUI_FONT);
        (f, f)
    };
    for (slot, new) in [(&FONT, font), (&BOLD, bold)] {
        let old = slot.swap(new as usize, Relaxed);
        if old != 0 {
            // deleting the stock fallback font is a harmless no-op
            DeleteObject(old as HGDIOBJ);
        }
    }
    let hinst = GetModuleHandleW(null());
    // in pixels; `ctl` takes 96-DPI pixels
    let ctl_px = |class: &str, text: &str, style: u32, id: i32, x: i32, y: i32, w: i32, h: i32| {
        let c =
            CreateWindowExW(0, wide(class).as_ptr(), wide(text).as_ptr(), WS_CHILD | WS_VISIBLE | style, x, y, w, h, hwnd, id as usize as HMENU, hinst, null());
        SendMessageW(c, WM_SETFONT, font as usize, 1);
        c
    };
    let ctl = |class: &str, text: &str, style: u32, id: i32, x: i32, y: i32, w: i32, h: i32| {
        ctl_px(class, text, style, id, px(x, dpi), px(y, dpi), px(w, dpi), px(h, dpi))
    };
    // a push button (a tab stop that Space, Enter and a click activate) drawn by `draw_link`, as wide
    // as its text plus one pixel each side and the underline below the line, for the focus
    // rectangle; (x, y) is where the text starts, as in a label
    let link = |text: &str, id: i32, x: i32, y: i32, line: i32| {
        ctl_px("BUTTON", text, (BS_PUSHBUTTON as u32) | WS_TABSTOP | WS_GROUP, id, x - 1, y, text_width(hwnd, text) + 2, line + px(2, dpi).max(2));
    };
    let heading = |text: &str, y: i32| SendMessageW(ctl("STATIC", text, 0, 0, 11, y, 378, 16), WM_SETFONT, bold as usize, 1);
    let check = (BS_AUTOCHECKBOX as u32) | WS_TABSTOP;
    let radio = (BS_AUTORADIOBUTTON as u32) | WS_TABSTOP;
    let button = (BS_PUSHBUTTON as u32) | WS_TABSTOP;
    // a row is 17 px (check box) or 23 px (button, combo); in a 23 px row the check box sits 3 px
    // and a label 4 px below its top, so the text baselines line up
    // a dropdown list of 23 px whose label sits right-aligned before it, as in a dialog
    let list = |label: &str, id: i32, y: i32, items: &[&str]| {
        ctl("STATIC", label, SS_RIGHT, 0, 179, y + 4, 100, 16);
        let combo = ctl("COMBOBOX", "", (CBS_DROPDOWNLIST as u32) | WS_TABSTOP | WS_VSCROLL, id, 285, y, 104, 200);
        for item in items {
            SendMessageW(combo, CB_ADDSTRING, 0, wide(item).as_ptr() as isize);
        }
        combo
    };
    let mut y = 11;
    ctl("BUTTON", tr("Bật gõ tiếng Việt", "Enable Vietnamese"), check | WS_GROUP, ID_ENABLED, 11, y + 3, 200, 17);
    let combo = list(tr("Phím tắt", "Shortcut"), ID_HOTKEY, y, &HOTKEYS.map(|(h, _)| hotkey_label(h)));
    y += 23 + 15;
    heading(tr("Gõ", "Typing"), y);
    y += 16 + 7;
    let tone = tr("Kiểu bỏ dấu", "Tone mark placement");
    let tone_w = text_width(hwnd, tone);
    ctl_px("STATIC", tone, 0, 0, px(11, dpi), px(y + 1, dpi), tone_w + 1, px(16, dpi));
    // the radios start 7 px after the label, at 118 px at the least
    let radio_x = px(118, dpi).max(px(11 + 7, dpi) + tone_w);
    ctl_px("BUTTON", "hòa, thúy", radio | WS_GROUP, ID_TONE_OLD, radio_x, px(y, dpi), px(110, dpi), px(17, dpi));
    ctl_px("BUTTON", "hoà, thuý", radio, ID_TONE_NEW, radio_x + px(116, dpi), px(y, dpi), px(110, dpi), px(17, dpi));
    y += 17 + 7;
    // WS_GROUP ends the radio group
    ctl("BUTTON", tr("Tự giữ nguyên từ tiếng Anh", "Keep English words as typed"), check | WS_GROUP, ID_AUTO_EN, 11, y, 378, 17);
    y += 17 + 7;
    ctl("BUTTON", tr("Nhường khi IME Nhật/Trung/Hàn đang bật", "Step aside while a Japanese/Chinese/Korean IME is on"), check, ID_YIELD, 11, y, 378, 17);
    y += 17 + 15;
    heading(tr("Hệ thống", "System"), y);
    y += 16 + 7;
    // each language in its own words
    let lang = list(tr("Ngôn ngữ", "Language"), ID_LANG, y, &["Tiếng Việt", "English"]);
    y += 23 + 7;
    ctl("BUTTON", tr("Khởi động cùng Windows", "Start with Windows"), check, ID_STARTUP, 11, y, 378, 17);
    y += 17 + 7;
    ctl("BUTTON", tr("Chạy quyền admin (gõ được trong cửa sổ admin)", "Run as administrator (types in admin windows too)"), check, ID_ADMIN, 11, y, 378, 17);
    y += 17 + 7;
    let dev_box = ctl("BUTTON", tr("Chế độ phát triển", "Developer mode"), check, ID_DEV, 11, y, 200, 17);
    set_check(hwnd, ID_DEV, dev);
    y += 17;
    if dev {
        // under the check box's label: the glyph plus a quarter of it, where `draw_check` puts the label
        let theme = OpenThemeDataForDpi(dev_box, wide("Button").as_ptr(), dpi);
        let mut glyph = SIZE { cx: px(13, dpi), cy: 0 };
        if theme != 0 {
            GetThemePartSize(theme, null_mut(), BP_CHECKBOX, 1, null(), TS_DRAW, &mut glyph);
            CloseThemeData(theme);
        }
        let x = px(11, dpi) + glyph.cx + glyph.cx / 4;
        y += 7;
        // two buttons of one size on one row, 7 px apart
        ctl_px("BUTTON", tr("Mở thư mục cấu hình…", "Open config folder…"), button | WS_GROUP, ID_FOLDER, x, px(y, dpi), px(160, dpi), px(23, dpi));
        ctl_px("BUTTON", tr("Chép thông tin debug", "Copy debug info"), button, ID_DEBUG, x + px(160 + 7, dpi), px(y, dpi), px(160, dpi), px(23, dpi));
        y += 23;
    }
    y += 15;
    heading(tr("Cập nhật", "Updates"), y);
    y += 16 + 7;
    ctl("BUTTON", tr("Tự động cập nhật", "Update automatically"), check | WS_GROUP, ID_AUTO_UPDATE, 11, y + 3, 200, 17);
    ctl("BUTTON", tr("Kiểm tra bản mới", "Check for updates"), button, ID_CHECK, 259, y, 130, 23);
    y += 23;
    // as many lines as the status needs, one also when empty; another need rebuilds (`sync_update`)
    let st = status(hwnd, dpi);
    SHAPE.store(st.key(), Relaxed);
    y += 7;
    let (top, h) = (px(y, dpi), st.lines * st.line);
    // the text beside or above the link, never under it: a sibling that repaints (a new text)
    // would paint over an overlapped link
    let (w, th) = match st.link {
        Some((x, 0)) => (x - 1, h),
        Some(_) => (px(378, dpi), h - st.line),
        None => (px(378, dpi), h),
    };
    ctl_px("STATIC", &st.text, SS_NOPREFIX, ID_UPDATE_STATUS, px(11, dpi), top, w.max(0), th);
    if let Some((x, l)) = st.link {
        link(update_link(), ID_UPDATE_LINK, px(11, dpi) + x, top + l * st.line, st.line);
    }
    y += (h * 96 + dpi as i32 - 1) / dpi as i32;
    y += 11;
    LINE_Y.store(px(y, dpi), Relaxed);
    y += 1 + 11;
    ctl("STATIC", &triokey_shared::footer(env!("CARGO_PKG_VERSION")), 0, ID_FOOTER, 11, y, 280, 16);
    link("GitHub", ID_GITHUB, px(389, dpi) - text_width(hwnd, "GitHub"), px(y, dpi), st.line);
    y += 16;
    if pal().is_some_and(|p| p.dark) {
        // the system's dark themes for these classes; a name that does not resolve leaves the usual theme
        for id in [ID_ENABLED, ID_TONE_OLD, ID_TONE_NEW, ID_AUTO_EN, ID_YIELD, ID_STARTUP, ID_ADMIN, ID_DEV, ID_FOLDER, ID_DEBUG, ID_AUTO_UPDATE, ID_CHECK] {
            SetWindowTheme(GetDlgItem(hwnd, id), wide("DarkMode_Explorer").as_ptr(), null());
        }
        for c in [combo, lang] {
            SetWindowTheme(c, wide("DarkMode_CFD").as_ptr(), null());
        }
    }
    y + 11
}

/// Width of one line of `text` in pixels in the message font.
unsafe fn text_width(hwnd: HWND, text: &str) -> i32 {
    let dc = GetDC(hwnd);
    let old = SelectObject(dc, FONT.load(Relaxed) as HGDIOBJ);
    let w: Vec<u16> = text.encode_utf16().collect();
    let mut size = std::mem::zeroed::<SIZE>();
    GetTextExtentPoint32W(dc, w.as_ptr(), w.len() as i32, &mut size);
    SelectObject(dc, old);
    ReleaseDC(hwnd, dc);
    size.cx
}

/// `rc` filled with `color` through the DC's own brush: nothing to create or free.
unsafe fn paint_rect(dc: HDC, rc: &RECT, color: u32) {
    SetDCBrushColor(dc, color);
    FillRect(dc, rc, GetStockObject(DC_BRUSH) as HBRUSH);
}

/// Check boxes and radio buttons in a palette, drawn whole: under visual styles they take their
/// label colour from the theme, not from WM_CTLCOLORSTATIC, which leaves black text on a dark
/// background. The glyph is still the control's own theme part, opened for the layout DPI.
unsafe fn draw_check(cd: &NMCUSTOMDRAW, p: &Palette) -> LRESULT {
    let (h, dc, rc) = (cd.hdr.hwndFrom, cd.hdc, cd.rc);
    if cd.dwDrawStage != CDDS_PREPAINT {
        return CDRF_DODEFAULT as LRESULT;
    }
    // not GetWindowTheme: it returns no handle for radio buttons here. The open honours the
    // control's SetWindowTheme name.
    let theme = OpenThemeDataForDpi(h, wide("Button").as_ptr(), DPI.load(Relaxed));
    if theme == 0 {
        return CDRF_DODEFAULT as LRESULT;
    }
    let part = if GetWindowLongW(h, GWL_STYLE) as u32 & BS_TYPEMASK as u32 == BS_AUTORADIOBUTTON as u32 { BP_RADIOBUTTON } else { BP_CHECKBOX };
    let enabled = IsWindowEnabled(h) != 0;
    // CBS_* and RBS_* share the numbering: 1-4 unchecked normal/hot/pressed/disabled, 5-8 checked
    let state = if SendMessageW(h, BM_GETCHECK, 0, 0) == BST_CHECKED { 4 } else { 0 }
        + if !enabled {
            4
        } else if cd.uItemState & CDIS_SELECTED != 0 {
            3
        } else if cd.uItemState & CDIS_HOT != 0 {
            2
        } else {
            1
        };
    paint_rect(dc, &rc, p.bg);
    let mut size = std::mem::zeroed::<SIZE>();
    GetThemePartSize(theme, dc, part, state, null(), TS_DRAW, &mut size);
    let top = rc.top + (rc.bottom - rc.top - size.cy) / 2;
    DrawThemeBackground(theme, dc, part, state, &RECT { left: rc.left, top, right: rc.left + size.cx, bottom: top + size.cy }, null());
    let mut text = [0u16; 128];
    let n = GetWindowTextW(h, text.as_mut_ptr(), text.len() as i32);
    let old = SelectObject(dc, SendMessageW(h, WM_GETFONT, 0, 0) as HGDIOBJ);
    let opt = DTTOPTS { dwSize: size_of::<DTTOPTS>() as u32, dwFlags: DTT_TEXTCOLOR, crText: if enabled { p.text } else { p.off }, ..std::mem::zeroed() };
    let mut t = RECT { left: rc.left + size.cx + size.cx / 4, ..rc };
    DrawThemeTextEx(theme, dc, part, state, text.as_ptr(), n, DT_SINGLELINE | DT_VCENTER, &mut t, &opt);
    // the focus cue only while the keyboard is in use (WM_CHANGEUISTATE)
    if cd.uItemState & CDIS_FOCUS != 0 && SendMessageW(h, WM_QUERYUISTATE, 0, 0) as u32 & UISF_HIDEFOCUS == 0 {
        DrawTextW(dc, text.as_ptr(), n, &mut t, DT_SINGLELINE | DT_CALCRECT);
        // around the text, one pixel out, kept inside the control so no side is clipped
        let top = ((rc.top + rc.bottom - (t.bottom - t.top)) / 2 - 1).max(rc.top);
        DrawFocusRect(dc, &RECT { left: t.left - 1, top, right: (t.right + 1).min(rc.right), bottom: (top + t.bottom - t.top + 2).min(rc.bottom) });
    }
    SelectObject(dc, old);
    CloseThemeData(theme);
    CDRF_SKIPDEFAULT as LRESULT
}

/// A link: underlined text in the message font in the palette's link colour or, in the system look, the system's
/// hyperlink colour (COLOR_HOTLIGHT) on the button face, and the focus rectangle around the control
/// (the text and one pixel) while the keyboard is in use.
unsafe fn draw_link(cd: &NMCUSTOMDRAW) -> LRESULT {
    let (h, dc, rc) = (cd.hdr.hwndFrom, cd.hdc, cd.rc);
    if cd.dwDrawStage != CDDS_PREPAINT {
        return CDRF_DODEFAULT as LRESULT;
    }
    let (bg, fg) = pal().map_or((GetSysColor(COLOR_BTNFACE), GetSysColor(COLOR_HOTLIGHT)), |p| (p.bg, p.link));
    paint_rect(dc, &rc, bg);
    let mut text = [0u16; 64];
    let n = GetWindowTextW(h, text.as_mut_ptr(), text.len() as i32);
    let old = SelectObject(dc, SendMessageW(h, WM_GETFONT, 0, 0) as HGDIOBJ);
    SetTextColor(dc, fg);
    SetBkMode(dc, TRANSPARENT as _);
    let mut t = RECT { left: rc.left + 1, ..rc };
    DrawTextW(dc, text.as_ptr(), n, &mut t, DT_SINGLELINE | DT_NOPREFIX);
    // its own underline, under the line: an underlined font's runs through the dot below of ạ and ậ
    let mut tm = std::mem::zeroed::<TEXTMETRICW>();
    GetTextMetricsW(dc, &mut tm);
    let top = rc.top + tm.tmHeight;
    // twice as thick while pressed: the only press feedback a link has
    let thick = px(if cd.uItemState & CDIS_SELECTED != 0 { 2 } else { 1 }, DPI.load(Relaxed)).max(1);
    paint_rect(dc, &RECT { left: rc.left + 1, top, right: rc.right - 1, bottom: top + thick }, fg);
    if cd.uItemState & CDIS_FOCUS != 0 && SendMessageW(h, WM_QUERYUISTATE, 0, 0) as u32 & UISF_HIDEFOCUS == 0 {
        DrawFocusRect(dc, &rc);
    }
    SelectObject(dc, old);
    CDRF_SKIPDEFAULT as LRESULT
}

/// A disabled push button in a palette, drawn whole: the dark theme's disabled face differs from
/// the enabled one only by a slightly greyer label, so the label takes the palette's disabled colour.
unsafe fn draw_disabled_button(cd: &NMCUSTOMDRAW, p: &Palette) -> LRESULT {
    let (h, dc, mut rc) = (cd.hdr.hwndFrom, cd.hdc, cd.rc);
    if cd.dwDrawStage != CDDS_PREPAINT {
        return CDRF_DODEFAULT as LRESULT;
    }
    let theme = OpenThemeDataForDpi(h, wide("Button").as_ptr(), DPI.load(Relaxed));
    if theme == 0 {
        return CDRF_DODEFAULT as LRESULT;
    }
    paint_rect(dc, &rc, p.bg);
    DrawThemeBackground(theme, dc, BP_PUSHBUTTON, PBS_DISABLED, &rc, null());
    let mut text = [0u16; 128];
    let n = GetWindowTextW(h, text.as_mut_ptr(), text.len() as i32);
    let old = SelectObject(dc, SendMessageW(h, WM_GETFONT, 0, 0) as HGDIOBJ);
    let opt = DTTOPTS { dwSize: size_of::<DTTOPTS>() as u32, dwFlags: DTT_TEXTCOLOR, crText: p.off, ..std::mem::zeroed() };
    DrawThemeTextEx(theme, dc, BP_PUSHBUTTON, PBS_DISABLED, text.as_ptr(), n, DT_CENTER | DT_VCENTER | DT_SINGLELINE, &mut rc, &opt);
    SelectObject(dc, old);
    CloseThemeData(theme);
    CDRF_SKIPDEFAULT as LRESULT
}

unsafe fn set_check(hwnd: HWND, id: i32, on: bool) {
    SendMessageW(GetDlgItem(hwnd, id), BM_SETCHECK, on as usize, 0);
}

unsafe fn checked(hwnd: HWND, id: i32) -> bool {
    SendMessageW(GetDlgItem(hwnd, id), BM_GETCHECK, 0, 0) == BST_CHECKED
}

/// Show the real state: live on/off switch, Run value, config for the rest. The developer-mode
/// box is set by `build`, which knows whether its part exists.
unsafe fn fill(hwnd: HWND) {
    let s = app::settings();
    set_check(hwnd, ID_ENABLED, app::enabled());
    let i = HOTKEYS.iter().position(|(h, _)| *h == s.hotkey).unwrap_or(0);
    SendMessageW(GetDlgItem(hwnd, ID_HOTKEY), CB_SETCURSEL, i, 0);
    SendMessageW(GetDlgItem(hwnd, ID_LANG), CB_SETCURSEL, ENGLISH.load(Relaxed) as usize, 0);
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
}

/// What `Chép thông tin debug` copies, as plain values (docs/ARCHITECTURE.md, 2.6 Settings window).
struct DebugInfo<'a> {
    version: &'a str,
    /// Windows major, minor, build
    windows: (u32, u32, u32),
    elevated: bool,
    admin_copy: bool,
    exe: &'a str,
    config_dir: &'a str,
    config_readable: bool,
    /// config.txt as written (`config::render`)
    config: &'a str,
    enabled: bool,
    typing: bool,
    /// primary language of the foreground keyboard layout at the watcher's last read
    lang: u16,
    /// `hook::latency()`
    latency: ((u32, usize), (u32, usize)),
    status: &'a str,
    waiting: Option<&'a str>,
    /// remembered words, force_vi.txt entries, force_en.txt entries
    counts: (usize, usize, usize),
    look: &'a str,
    dpi: u32,
    /// the profile folder, replaced by `~` everywhere
    home: &'a str,
}

/// The debug information, in English, one fact per line. Counts only: never typed text, a remembered
/// word or the user's folder name (the home folder is masked over the whole text).
fn debug_text(d: &DebugInfo) -> String {
    let yes = |b: bool| if b { "yes" } else { "no" };
    let ((sent, ns), (pass, np)) = d.latency;
    let (learned, vi, en) = d.counts;
    let (major, minor, build) = d.windows;
    let settings = d.config.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("; ");
    // a path outside the profile folder (redirected AppData, an exe on another drive) could name the user
    let place =
        |p: &str| if p == "unknown" || triokey_shared::mask_home(p, d.home).starts_with('~') { p.to_string() } else { "outside the profile folder".into() };
    let text = [
        format!("TrioKey {} (Windows {major}.{minor}.{build}, x64)", d.version),
        format!("copy: {}; admin-mode copy {}", if d.elevated { "elevated" } else { "normal" }, yes(d.admin_copy)),
        format!("exe: {}", place(d.exe)),
        format!("config folder: {}; config.txt readable {}", place(d.config_dir), yes(d.config_readable)),
        format!("settings: {settings}"),
        format!(
            "state: {}; IME decision {}; keyboard layout primary language 0x{:04X}",
            if d.enabled { "on" } else { "off" },
            if d.typing { "Type" } else { "Yield" },
            d.lang
        ),
        format!("hook: p99 {sent} µs over {ns} keys that sent text; {pass} µs over {np} keys passed through"),
        format!("update: {}; waiting version {}", if d.status.is_empty() { "(no status)" } else { d.status }, d.waiting.unwrap_or("none")),
        format!("remembered words: {learned}; force_vi.txt: {vi} entries; force_en.txt: {en} entries"),
        format!("window: look {}, DPI {}", d.look, d.dpi),
    ]
    .join("\n");
    triokey_shared::mask_home(&(text + "\n"), d.home)
}

/// The debug information of this copy now.
unsafe fn debug_now() -> String {
    let mut v = OSVERSIONINFOW { dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32, ..std::mem::zeroed() };
    // the manifest declares Windows 10, so the real version comes back
    GetVersionExW(&mut v);
    let path = |p: Option<std::path::PathBuf>| p.map(|p| p.display().to_string());
    let (exe, dir, home) = (path(std::env::current_exe().ok()), path(app::config_dir()), path(admin::profile_dir()));
    // without a profile folder to mask, no path is given
    let (exe, dir) = if home.is_some() { (exe, dir) } else { (None, None) };
    // the status in Vietnamese whatever the window shows: the debug information is for the developer
    let (status, waiting) = (update::status(false), update::waiting());
    debug_text(&DebugInfo {
        version: env!("CARGO_PKG_VERSION"),
        windows: (v.dwMajorVersion, v.dwMinorVersion, v.dwBuildNumber),
        elevated: admin::is_elevated(),
        admin_copy: admin::is_admin_copy(),
        exe: exe.as_deref().unwrap_or("unknown"),
        config_dir: dir.as_deref().unwrap_or("unknown"),
        config_readable: !app::config_unreadable(),
        config: &render(&app::settings()),
        enabled: app::enabled(),
        typing: ime_watch::current() == crate::ime::ImeState::Type,
        lang: ime_watch::lang(),
        latency: hook::latency(),
        status: &status,
        waiting: waiting.as_deref(),
        counts: app::user_data_counts(),
        look: match pal() {
            None => "system",
            Some(p) if p.dark => "dark",
            _ => "light",
        },
        dpi: DPI.load(Relaxed),
        home: home.as_deref().unwrap_or(""),
    })
}

/// `text` on the clipboard as CF_UNICODETEXT (CRLF lines); false when any step fails. After a
/// successful SetClipboardData the system owns the memory, else it is freed here.
unsafe fn copy_text(hwnd: HWND, text: &str) -> bool {
    let w = wide(&text.replace('\n', "\r\n"));
    let mem = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2);
    if mem.is_null() {
        return false;
    }
    let p = GlobalLock(mem);
    if p.is_null() {
        GlobalFree(mem);
        return false;
    }
    std::ptr::copy_nonoverlapping(w.as_ptr(), p.cast::<u16>(), w.len());
    GlobalUnlock(mem);
    let ok = OpenClipboard(hwnd) != 0 && {
        let set = EmptyClipboard() != 0 && !SetClipboardData(CF_UNICODETEXT, mem).is_null();
        CloseClipboard();
        set
    };
    if !ok {
        GlobalFree(mem);
    }
    ok
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
            if let Some(&(h, _)) = usize::try_from(i).ok().and_then(|i| HOTKEYS.get(i)) {
                app::update(|s| s.hotkey = h);
            }
        }
        // saved and applied (`app::update` sets the language for the tray too), then the window is made again in it,
        // posted like the developer-mode switch. Not while the list is dropped (an arrow key there would rebuild it
        // away under the user): the choice counts when it closes.
        (ID_LANG, CBN_SELCHANGE | CBN_CLOSEUP) if SendMessageW(GetDlgItem(hwnd, ID_LANG), CB_GETDROPPEDSTATE, 0, 0) == 0 => {
            let en = chosen_english(hwnd);
            if en != ENGLISH.load(Relaxed) {
                app::update(|s| s.english = en);
                PostMessageW(hwnd, WM_APP_REBUILD, 0, 0);
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
                    message(
                        hwnd,
                        tr("Không ghi được mục tự khởi động trong registry.", "The startup entry could not be written to the registry."),
                        MB_OK | MB_ICONWARNING,
                    );
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
        // installs, or opens the download page (the address in a message box when elevated)
        (ID_UPDATE_LINK, BN_CLICKED) => update::click(),
        (ID_GITHUB, BN_CLICKED) => open_url(hwnd, REPO),
        (ID_DEV, BN_CLICKED) => {
            let on = checked(hwnd, id);
            app::update(|s| s.dev_mode = on);
            PostMessageW(hwnd, WM_APP_REBUILD, 0, 0);
        }
        (ID_DEBUG, BN_CLICKED) => {
            if copy_text(hwnd, &debug_now()) {
                message(hwnd, tr("Đã chép thông tin debug vào clipboard.", "The debug information is on the clipboard."), MB_OK | MB_ICONINFORMATION);
            } else {
                message(
                    hwnd,
                    tr("Không chép được thông tin debug vào clipboard.", "The debug information could not be copied to the clipboard."),
                    MB_OK | MB_ICONWARNING,
                );
            }
        }
        (ID_FOLDER, BN_CLICKED) => {
            let Some(dir) = app::config_dir() else { return };
            // The shell looks up the "open" verb in HKCU\Software\Classes, where an unelevated
            // program can register its own command, which would then run with this token:
            // an elevated TrioKey starts nothing and creates nothing here, it shows the path.
            if admin::is_elevated() {
                let t = tr(
                    "TrioKey đang chạy quyền admin nên không tự mở Explorer. Thư mục cấu hình:",
                    "TrioKey runs as administrator, so it does not open Explorer itself. Config folder:",
                );
                let t = format!("{t}\n\n{}", dir.display());
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
    if on {
        app::create_learned_file();
    }
    let en = ENGLISH.load(Relaxed);
    let code = match admin::run_elevated(hwnd, on) {
        Elevated::Cancelled => return,
        Elevated::Exit(c) if c == admin::OK || c == admin::OK_AFTER_REBOOT || c == admin::OK_FOLDER_LEFT => c,
        Elevated::Timeout => {
            let t = tr(
                "Bản quyền admin chưa chạy xong sau 2 phút nên chưa biết kết quả. Cấu hình chưa đổi; hãy kiểm tra tác vụ TrioKey trong Task Scheduler rồi thử lại.",
                "The administrator copy had not finished after 2 minutes, so the result is unknown. The settings are unchanged; check the TrioKey task in Task Scheduler, then try again.",
            );
            message(hwnd, t, MB_OK | MB_ICONWARNING);
            return;
        }
        r => {
            let why = match r {
                Elevated::Exit(admin::COPY_FAILED) => {
                    tr("không chép được triokey.exe vào Program Files (mã 3)", "triokey.exe could not be copied to Program Files (code 3)").to_string()
                }
                Elevated::Exit(admin::TASK_FAILED) => {
                    tr("schtasks không tạo được tác vụ TrioKey (mã 4)", "schtasks could not create the TrioKey task (code 4)").to_string()
                }
                Elevated::Exit(c) => format!("{} {c}", tr("mã", "code")),
                _ => tr("không chạy được bản quyền admin", "the administrator copy did not run").to_string(),
            };
            let t = match (on, en) {
                (true, false) => format!("Không bật được chế độ admin: {why}. Cấu hình không đổi."),
                (false, false) => format!("Không tắt được chế độ admin: {why}. Cấu hình không đổi."),
                (true, true) => format!("Could not turn admin mode on: {why}. The settings are unchanged."),
                (false, true) => format!("Could not turn admin mode off: {why}. The settings are unchanged."),
            };
            message(hwnd, &t, MB_OK | MB_ICONWARNING);
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
        let q = if en {
            format!(
                "Admin mode is on. From the next sign-in, TrioKey runs as administrator from\n{admin_exe}\n\nStart the admin copy now? This copy will quit."
            )
        } else {
            format!("Đã bật chế độ admin. Từ lần đăng nhập sau, TrioKey tự chạy quyền admin từ\n{admin_exe}\n\nChạy bản admin ngay bây giờ? Bản đang chạy sẽ thoát.")
        };
        if message(hwnd, &q, MB_YESNO | MB_ICONQUESTION) == IDYES {
            if admin::start_task() {
                tray::quit();
            } else {
                let t = tr(
                    "Không chạy được tác vụ TrioKey. Bản admin sẽ chạy ở lần đăng nhập sau.",
                    "The TrioKey task did not start. The admin copy will run at the next sign-in.",
                );
                message(hwnd, t, MB_OK | MB_ICONWARNING);
            }
        }
        return;
    }
    let s = app::settings();
    app::update(|s| s.run_as_admin = false);
    let later = match code {
        admin::OK_AFTER_REBOOT => tr(" Thư mục sẽ được xoá sau khi khởi động lại máy.", " The folder will be deleted after the computer restarts."),
        admin::OK_FOLDER_LEFT => tr(
            " Không xoá được thư mục Program Files\\TrioKey; hãy xoá tay bằng quyền admin.",
            " The folder Program Files\\TrioKey could not be deleted; delete it yourself as administrator.",
        ),
        _ => "",
    };
    if admin::is_admin_copy() {
        // this exe is about to go: Run must point at the normal copy, which only the user knows
        let t = if en {
            format!("Admin mode is off and the TrioKey task is deleted.{later}\n\nThis admin copy runs until you Quit. Quit, start the normal triokey.exe again, then turn \"Start with Windows\" back on if you want it.")
        } else {
            format!("Đã tắt chế độ admin và xoá tác vụ TrioKey.{later}\n\nBản admin này chạy tới khi bạn Thoát. Hãy Thoát, chạy lại triokey.exe bản thường, rồi bật lại \"Khởi động cùng Windows\" nếu cần.")
        };
        message(hwnd, &t, MB_OK | MB_ICONINFORMATION);
        return;
    }
    // the task is gone: without the Run value nothing starts TrioKey at logon, so say so
    let restored = !s.run_at_startup || std::env::current_exe().is_ok_and(|exe| autostart::set(true, &exe));
    let warn = if restored {
        ""
    } else {
        tr(
            "\n\nKhông ghi được mục tự khởi động trong registry: TrioKey sẽ không tự chạy khi đăng nhập.",
            "\n\nThe startup entry could not be written to the registry: TrioKey will not start at sign-in.",
        )
    };
    let icon = if restored && code != admin::OK_FOLDER_LEFT { MB_ICONINFORMATION } else { MB_ICONWARNING };
    let gone = if code == admin::OK_FOLDER_LEFT { tr("tác vụ TrioKey", "the TrioKey task") } else { &admin_exe };
    let t = if en {
        format!("Admin mode is off and {gone} is deleted.{later}{warn}")
    } else {
        format!("Đã tắt chế độ admin và xoá {gone}.{later}{warn}")
    };
    message(hwnd, &t, MB_OK | icon);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            command(hwnd, (wp & 0xFFFF) as i32, ((wp >> 16) & 0xFFFF) as u32);
            warn_unsaved(hwnd);
            0
        }
        // wParam 1: a theme change, applied here so no click handler (one waiting on a message box)
        // has its control destroyed; several posts for one switch rebuild once
        WM_APP_REBUILD => {
            let theme = if wp != 0 { wanted() } else { pal() };
            if wp == 0 || theme.map_or(null(), |p| p as *const Palette) != PALETTE.load(Relaxed).cast_const() {
                rebuild(hwnd, DPI.load(Relaxed), theme, checked(hwnd, ID_DEV), chosen_english(hwnd), top_left(hwnd));
            }
            0
        }
        // Moved to a monitor with another scale (or the scale changed): the position Windows
        // suggests, our own size for this DPI.
        WM_DPICHANGED => {
            let to = &*(lp as *const RECT);
            rebuild(hwnd, (wp & 0xFFFF) as u32, pal(), checked(hwnd, ID_DEV), chosen_english(hwnd), (to.left, to.top));
            0
        }
        // The apps' light/dark mode ("ImmersiveColorSet", sent several times per switch) or high
        // contrast changed: rebuilt only when the look it asks for differs from the one shown.
        WM_SETTINGCHANGE | WM_THEMECHANGED | WM_SYSCOLORCHANGE => {
            let relevant = msg != WM_SETTINGCHANGE || wp == SPI_SETHIGHCONTRAST as usize || (lp != 0 && wide_eq(lp as *const u16, "ImmersiveColorSet"));
            if relevant && wanted().map_or(null(), |p| p as *const Palette) != PALETTE.load(Relaxed).cast_const() {
                PostMessageW(hwnd, WM_APP_REBUILD, 1, 0);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        // A plain window, not a dialog: nothing else remembers which control had the focus
        // and gives it back after Alt+Tab, and a focus left on the frame makes Space do nothing.
        WM_ACTIVATE => {
            let f = GetFocus();
            if (wp & 0xFFFF) as u32 == WA_INACTIVE && IsChild(hwnd, f) != 0 {
                FOCUS.store(GetDlgCtrlID(f), Relaxed);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_SETFOCUS => {
            let id = FOCUS.load(Relaxed);
            let f = if id != 0 { GetDlgItem(hwnd, id) } else { null_mut() };
            SetFocus(if !f.is_null() && IsWindowEnabled(f) != 0 { f } else { GetDlgItem(hwnd, ID_ENABLED) });
            0
        }
        // IsDialogMessageW asks this on Enter: the focused push button (a link too) is the default
        // one, as in a dialog, so Enter activates it like Space
        DM_GETDEFID => {
            let f = GetFocus();
            let push = IsChild(hwnd, f) != 0 && SendMessageW(f, WM_GETDLGCODE, 0, 0) as u32 & (DLGC_DEFPUSHBUTTON | DLGC_UNDEFPUSHBUTTON) != 0;
            if push {
                (DC_HASDEFID << 16 | GetDlgCtrlID(f) as u32) as LRESULT
            } else {
                0
            }
        }
        WM_SETCURSOR if matches!(GetDlgCtrlID(wp as HWND), ID_UPDATE_LINK | ID_GITHUB) => {
            SetCursor(LoadCursorW(null_mut(), IDC_HAND));
            1
        }
        WM_NOTIFY => {
            // only NM_CUSTOMDRAW carries the larger NMCUSTOMDRAW
            let hdr = &*(lp as *const NMHDR);
            let kind = GetWindowLongW(hdr.hwndFrom, GWL_STYLE) as u32 & BS_TYPEMASK as u32;
            if hdr.code == NM_CUSTOMDRAW && matches!(GetDlgCtrlID(hdr.hwndFrom), ID_UPDATE_LINK | ID_GITHUB) {
                return draw_link(&*(lp as *const NMCUSTOMDRAW));
            }
            match pal() {
                Some(p) if hdr.code == NM_CUSTOMDRAW && (kind == BS_AUTOCHECKBOX as u32 || kind == BS_AUTORADIOBUTTON as u32) => {
                    draw_check(&*(lp as *const NMCUSTOMDRAW), p)
                }
                Some(p) if hdr.code == NM_CUSTOMDRAW && kind == BS_PUSHBUTTON as u32 && IsWindowEnabled(hdr.hwndFrom) == 0 => {
                    draw_disabled_button(&*(lp as *const NMCUSTOMDRAW), p)
                }
                _ => DefWindowProcW(hwnd, msg, wp, lp),
            }
        }
        // the background (the class brush in the system look) and the hairline above the footer
        WM_ERASEBKGND => {
            let dc = wp as HDC;
            let mut rc = std::mem::zeroed::<RECT>();
            GetClientRect(hwnd, &mut rc);
            match pal() {
                Some(p) => paint_rect(dc, &rc, p.bg),
                None => {
                    DefWindowProcW(hwnd, msg, wp, lp);
                }
            }
            let top = LINE_Y.load(Relaxed);
            let line = RECT { top, bottom: top + px(1, DPI.load(Relaxed)).max(1), ..rc };
            paint_rect(dc, &line, pal().map_or(GetSysColor(COLOR_BTNTEXT), |p| p.line));
            1
        }
        // labels, check boxes and radio buttons (their background), and the dropped hotkey list
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN | WM_CTLCOLORLISTBOX => match pal() {
            Some(p) => {
                let dc = wp as HDC;
                SetTextColor(dc, if GetDlgCtrlID(lp as HWND) == ID_FOOTER { p.dim } else { p.text });

                SetBkColor(dc, p.bg);
                BRUSH.load(Relaxed) as LRESULT
            }
            None => DefWindowProcW(hwnd, msg, wp, lp),
        },
        WM_DESTROY => {
            set_icons(hwnd, 0);
            HWND_SETTINGS.store(null_mut(), Relaxed);
            FOCUS.store(0, Relaxed);
            for h in [FONT.swap(0, Relaxed), BOLD.swap(0, Relaxed), BRUSH.swap(0, Relaxed)] {
                if h != 0 {
                    DeleteObject(h as HGDIOBJ);
                }
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

    #[test]
    fn debug_text_is_one_fact_per_line_and_names_no_user() {
        let config = render(&crate::config::Settings::default());
        let d = DebugInfo {
            version: "0.3.0",
            windows: (10, 0, 26100),
            elevated: false,
            admin_copy: false,
            exe: r"C:\Users\example-user\AppData\Local\TrioKey\triokey.exe",
            config_dir: r"C:\Users\Example-User\AppData\Roaming\triokey",
            config_readable: true,
            config: &config,
            enabled: true,
            typing: false,
            lang: 0x2A,
            latency: ((180, 40), (25, 900)),
            status: "Chưa kiểm tra được: không tải được từ GitHub.",
            waiting: None,
            counts: (3, 1, 0),
            look: "dark",
            dpi: 144,
            home: r"C:\Users\example-user",
        };
        let t = debug_text(&d);
        assert!(!t.to_lowercase().contains("example-user"), "{t}");
        let lines: Vec<_> = t.lines().collect();
        assert_eq!(lines.len(), 10, "{t}");
        assert_eq!(lines[0], "TrioKey 0.3.0 (Windows 10.0.26100, x64)");
        assert_eq!(lines[1], "copy: normal; admin-mode copy no");
        assert_eq!(lines[2], r"exe: ~\AppData\Local\TrioKey\triokey.exe");
        assert_eq!(lines[3], r"config folder: ~\AppData\Roaming\triokey; config.txt readable yes");
        // the last line of config.txt is the interface language (added in 0.4.0)
        assert!(lines[4].starts_with("settings: enabled = true; tone_new = false; ") && lines[4].ends_with("; dev_mode = false; language = vi"), "{t}");
        assert_eq!(lines[5], "state: on; IME decision Yield; keyboard layout primary language 0x002A");
        assert_eq!(lines[6], "hook: p99 180 µs over 40 keys that sent text; 25 µs over 900 keys passed through");
        assert_eq!(lines[7], "update: Chưa kiểm tra được: không tải được từ GitHub.; waiting version none");
        assert_eq!(lines[8], "remembered words: 3; force_vi.txt: 1 entries; force_en.txt: 0 entries");
        assert_eq!(lines[9], "window: look dark, DPI 144");
        let elevated = debug_text(&DebugInfo { elevated: true, admin_copy: true, waiting: Some("9.9.9"), status: "", ..d });
        assert!(elevated.contains("copy: elevated; admin-mode copy yes\n") && elevated.contains("update: (no status); waiting version 9.9.9\n"));
        let redirected = debug_text(&DebugInfo {
            exe: r"D:\Users\example-user\tools\triokey.exe",
            config_dir: r"\\server\users$\example-user\AppData\Roaming\triokey",
            ..d
        });
        assert!(
            redirected.contains("exe: outside the profile folder\nconfig folder: outside the profile folder; ") && !redirected.contains("example-user"),
            "{redirected}"
        );
    }

    #[test]
    fn palette_text_meets_wcag_aa() {
        let rgb = |c: u32| [c as u8, (c >> 8) as u8, (c >> 16) as u8];
        for p in [&LIGHT, &DARK] {
            for c in [p.text, p.dim, p.link] {
                let r = triokey_shared::glyph::contrast(rgb(c), rgb(p.bg));
                assert!(r >= 4.5, "{c:06x} on {:06x}: {r:.2}", p.bg);
            }
        }
    }

    /// Client area (or, `frame`, the whole window through DWM) as RGB rows. `WM_PRINT` draws the
    /// children at their offset from the window, not the client area, so the whole window is
    /// printed and the client area cut out.
    unsafe fn capture(hwnd: HWND, frame: bool) -> (u32, u32, Vec<u8>) {
        #[link(name = "user32")]
        extern "system" {
            fn PrintWindow(hwnd: HWND, dc: HDC, flags: u32) -> i32;
        }
        let (mut win, mut cl): (RECT, RECT) = std::mem::zeroed();
        GetWindowRect(hwnd, &mut win);
        GetClientRect(hwnd, &mut cl);
        let mut origin = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        ClientToScreen(hwnd, &mut origin);
        let (ww, wh) = ((win.right - win.left) as u32, (win.bottom - win.top) as u32);
        let dc = CreateCompatibleDC(null_mut());
        let mut bi: BITMAPINFO = std::mem::zeroed();
        bi.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: ww as i32,
            biHeight: -(wh as i32),
            biPlanes: 1,
            biBitCount: 32,
            ..std::mem::zeroed()
        };
        let mut bits = null_mut();
        let bmp = CreateDIBSection(dc, &bi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        assert!(!bmp.is_null());
        let old = SelectObject(dc, bmp);
        if frame {
            assert_ne!(PrintWindow(hwnd, dc, PW_RENDERFULLCONTENT), 0, "PrintWindow");
        } else {
            SendMessageW(hwnd, WM_PRINT, dc as usize, (PRF_NONCLIENT | PRF_CLIENT | PRF_CHILDREN | PRF_ERASEBKGND) as isize);
        }
        GdiFlush();
        let bgra = std::slice::from_raw_parts(bits as *const u8, (ww * wh * 4) as usize);
        let (x0, y0, w, h) =
            if frame { (0, 0, ww, wh) } else { ((origin.x - win.left) as u32, (origin.y - win.top) as u32, cl.right as u32, cl.bottom as u32) };
        let mut rgb = Vec::with_capacity((w * h * 3) as usize);
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                let i = ((y * ww + x) * 4) as usize;
                rgb.extend([bgra[i + 2], bgra[i + 1], bgra[i]]);
            }
        }
        SelectObject(dc, old);
        DeleteObject(bmp);
        DeleteDC(dc);
        (w, h, rgb)
    }

    /// The control's own comctl32 (the module that registered its class) reports version 6.
    unsafe fn comctl_major(c: HWND) -> u32 {
        use windows_sys::Win32::System::LibraryLoader::GetProcAddress;
        use windows_sys::Win32::UI::Shell::{DLLGETVERSIONPROC, DLLVERSIONINFO};
        let module = GetClassLongPtrW(c, GCLP_HMODULE) as windows_sys::Win32::Foundation::HMODULE;
        let f: DLLGETVERSIONPROC = std::mem::transmute(GetProcAddress(module, c"DllGetVersion".as_ptr().cast()));
        let mut v = DLLVERSIONINFO { cbSize: size_of::<DLLVERSIONINFO>() as u32, ..std::mem::zeroed() };
        f.map_or(0, |f| if f(&mut v) == 0 { v.dwMajorVersion } else { 0 })
    }

    use crate::update::tests::pretend;
    use crate::update::{Error, Status};

    /// The version that waits in the preview.
    const PENDING: Option<(&str, bool)> = Some(("9.9.9", true));
    fn waiting_status() -> Status {
        Status::Waiting("9.9.9".into())
    }
    /// A long status (an install stopped half-way), two lines at this width in either language.
    fn longest() -> Status {
        Status::InstallFailed("9.9.9".into(), Error::Stranded)
    }
    /// The ordinary answers of `Kiểm tra bản mới`, and no status yet: one line each, also beside
    /// the link of a version that already waits (a new check keeps it).
    fn one_line() -> [(Option<(&'static str, bool)>, Status); 7] {
        let net = Status::CheckFailed(Error::Net);
        [
            (None, Status::None),
            (None, Status::Checking),
            (None, Status::Latest),
            (PENDING, waiting_status()),
            (None, net.clone()),
            (PENDING, Status::Checking),
            (PENDING, net),
        ]
    }

    fn save(dir: &std::path::Path, name: &str, what: &str, (w, h, rgb): (u32, u32, Vec<u8>)) {
        std::fs::write(dir.join(name), crate::icon::tests::png_rgb(w, h, &rgb)).unwrap();
        println!("{name}: {what}");
    }

    /// `TRIOKEY_PREVIEW_DIR=<dir> cargo test -p triokey settings_preview -- --ignored --nocapture`
    /// writes, without ever showing or activating a window, and prints what each file shows:
    /// `client-<look>[-en]-dev-<off|on>-<latest|waiting>-96.png` (the client area of a hidden window as a
    /// user sees it, in Vietnamese or, `-en`, English, with the status "up to date" or a waiting version and its link),
    /// `frame-…-96.png` (the same with the caption, cloaked and off-screen), `client-<look>-dev-on-waiting-144.png`,
    /// `client-system-dev-off-waiting-96.png`, and `states-…` for the artificial states (disabled
    /// controls, simulated focus, the longest status). The update status is set
    /// directly (`pretend`): nothing is checked, fetched or opened. The look is forced by parameter and
    /// no Windows setting is changed; `triggers` reads the apps' light/dark mode to check the theme-change path.
    #[test]
    #[ignore]
    fn settings_preview() {
        let dir = std::path::PathBuf::from(std::env::var("TRIOKEY_PREVIEW_DIR").expect("TRIOKEY_PREVIEW_DIR"));
        std::fs::create_dir_all(&dir).unwrap();
        unsafe {
            let manifest = !windows_sys::Win32::System::LibraryLoader::FindResourceW(null_mut(), 1 as _, 24 as _).is_null();
            println!("manifest in test exe: {manifest}, monitor dpi: {}", windows_sys::Win32::UI::HiDpi::GetDpiForSystem());
            let on = |dev: bool| if dev { "on" } else { "off" };
            for (theme, name) in [(&LIGHT, "light"), (&DARK, "dark")] {
                for (en, lang) in [(false, ""), (true, "-en")] {
                    for dev in [false, true] {
                        for (pending, status, what) in [(None, Status::Latest, "latest"), (PENDING, waiting_status(), "waiting")] {
                            let text = status.text(en);
                            pretend(pending, status);
                            let tag = format!("{name}{lang}-dev-{}-{what}", on(dev));
                            let h = create(Some(theme), dev, en, Some(96), null_mut());
                            check(h, dev, &tag);
                            let link = if pending.is_some() { format!(" + link \"{}\"", update_link()) } else { String::new() };
                            let what = format!("{name}, {}, developer mode {}, status \"{text}\"{link}", pick(en, "Vietnamese", "English"), on(dev));
                            save(&dir, &format!("client-{tag}-96.png"), &format!("client area, {what}"), capture(h, false));
                            DestroyWindow(h);
                            save(&dir, &format!("frame-{tag}-96.png"), &format!("whole window with caption, {what}"), frame(theme, dev, en, &tag));
                        }
                    }
                }
                pretend(PENDING, waiting_status());
                let h = create(Some(theme), true, false, Some(144), null_mut());
                check(h, true, name);
                let what = format!("client area at 144 DPI (layout and fonts only), {name}, developer mode on, waiting version");
                save(&dir, &format!("client-{name}-dev-on-waiting-144.png"), &what, capture(h, false));
                DestroyWindow(h);
                for (en, lang) in [(false, ""), (true, "-en")] {
                    pretend(None, longest());
                    let h = create(Some(theme), true, en, Some(96), null_mut());
                    states(h);
                    let what = format!("ARTIFICIAL, {name}{lang}: disabled check box and buttons, simulated focus on a check box, a long two-line status");
                    save(&dir, &format!("states-{name}{lang}-error-dev-on-96.png"), &what, capture(h, false));
                    DestroyWindow(h);
                    pretend(PENDING, longest());
                    let h = create(Some(theme), false, en, Some(96), null_mut());
                    check(h, false, name);
                    let what = format!(
                        "ARTIFICIAL, {name}{lang}: a version waits while the status says something long (a failed install): the link on a line of its own"
                    );
                    save(&dir, &format!("states-{name}{lang}-waiting-long-status-96.png"), &what, capture(h, false));
                    DestroyWindow(h);
                }
                pretend(PENDING, waiting_status());
                let h = create(Some(theme), false, false, Some(96), null_mut());
                focus(h, &[ID_UPDATE_LINK, ID_GITHUB]);
                let what = format!("ARTIFICIAL, {name}: simulated keyboard focus on both links at once (for real only one has it)");
                save(&dir, &format!("states-{name}-focus-links-96.png"), &what, capture(h, false));
                DestroyWindow(h);
            }
            pretend(PENDING, waiting_status());
            let h = create(None, false, false, Some(96), null_mut());
            check(h, false, "system");
            let what = "client area, system look (high contrast off, so normal system colours), waiting version: the link in COLOR_HOTLIGHT";
            save(&dir, "client-system-dev-off-waiting-96.png", what, capture(h, false));
            DestroyWindow(h);
            heights();
            triggers();
            debug_info();
            leaks();
        }
    }

    /// Visual styles are active, and the developer part and the links exist exactly when they should.
    unsafe fn check(h: HWND, dev: bool, tag: &str) {
        use windows_sys::Win32::UI::Controls::{GetWindowTheme, IsAppThemed};
        assert!(!h.is_null());
        let first = GetDlgItem(h, ID_ENABLED);
        let major = comctl_major(first);
        println!(
            "{tag} {}: comctl32 {major}, themed app {}, DWM HRESULTs (dark mode, caption, border, text) {:x?}",
            DPI.load(Relaxed),
            IsAppThemed(),
            caption(h)
        );
        assert!(major == 6 && IsAppThemed() != 0 && GetWindowTheme(first) != 0, "visual styles are not active");
        assert_eq!([GetDlgItem(h, ID_FOLDER).is_null(), GetDlgItem(h, ID_DEBUG).is_null()], [!dev; 2]);
        assert_eq!(GetDlgItem(h, ID_UPDATE_LINK).is_null(), update::waiting().is_none());
        assert!(!GetDlgItem(h, ID_GITHUB).is_null() && !GetDlgItem(h, ID_UPDATE_STATUS).is_null());
        // the link and the status text do not overlap
        let rect = |id| {
            let mut rc = std::mem::zeroed::<RECT>();
            GetWindowRect(GetDlgItem(h, id), &mut rc);
            rc
        };
        if dev {
            // the two developer buttons: one row, one size, 7 px apart, below the check box
            let (a, b, d) = (rect(ID_FOLDER), rect(ID_DEBUG), DPI.load(Relaxed));
            assert!(a.top == b.top && a.bottom == b.bottom && a.right - a.left == b.right - b.left, "{tag}: developer buttons differ");
            assert!(b.left - a.right == px(7, d) && (a.top - rect(ID_DEV).bottom - px(7, d)).abs() <= 1, "{tag}: developer button gaps");
        }
        let mut both = std::mem::zeroed::<RECT>();
        assert_eq!(IntersectRect(&mut both, &rect(ID_UPDATE_STATUS), &rect(ID_UPDATE_LINK)), 0, "{tag}: the link overlaps the status");
    }

    /// The window is as tall as its content: one status line is always reserved, so the ordinary
    /// answers of the version check (`one_line`) give one height per DPI and developer mode, the same in both
    /// languages; two lines are taller, and developer mode adds its row.
    unsafe fn heights() {
        let client = |dev: bool, dpi: u32, en: bool, (pending, status): (Option<(&str, bool)>, Status)| {
            pretend(pending, status);
            let h = create(Some(&LIGHT), dev, en, Some(dpi), null_mut());
            let mut cl = std::mem::zeroed::<RECT>();
            GetClientRect(h, &mut cl);
            DestroyWindow(h);
            (cl.bottom, SHAPE.load(Relaxed) & 0xF)
        };
        for en in [false, true] {
            let mut one = Vec::new();
            for dev in [false, true] {
                for dpi in [96, 144] {
                    let hs: Vec<_> = one_line().into_iter().map(|st| client(dev, dpi, en, st)).collect();
                    println!("client heights and status lines at {dpi} DPI, developer mode {dev}, English {en} (empty, checking, latest, waiting, no network, checking and no network while a version waits): {hs:?}");
                    assert!(hs.iter().all(|&h| h == (hs[0].0, 1)), "the window size changes with an ordinary status: {hs:?}");
                    one.push(hs[0].0);
                }
            }
            assert!(one[0] < one[2] && one[1] < one[3], "developer mode adds no row: {one:?}");
            let hs = [client(false, 96, en, (None, longest())), client(true, 96, en, (None, longest())), client(false, 96, en, (PENDING, longest()))];
            println!("long, long with developer mode, long while a version waits, English {en}: {hs:?}");
            assert_eq!(hs.map(|h| h.1), [2, 2, 3]);
            assert!(one[0] < hs[0].0 && hs[0].0 < hs[1].0 && hs[0].0 < hs[2].0, "{hs:?}");
        }
    }

    /// The window procedure's rebuild triggers, on a hidden window and through messages sent to it
    /// alone: the developer-mode switch (posted message, not WM_COMMAND, which would save the
    /// setting), a theme change, a status that needs other room, and a DPI change.
    unsafe fn triggers() {
        let height = |h: HWND| {
            let mut rc = std::mem::zeroed::<RECT>();
            GetClientRect(h, &mut rc);
            (rc.right, rc.bottom)
        };
        pretend(None, Status::Latest);
        let other = if wanted().is_some_and(|p| p.dark) { &LIGHT } else { &DARK };
        let h = create(Some(other), false, false, Some(96), null_mut());
        let off = height(h);
        set_check(h, ID_DEV, true);
        SendMessageW(h, WM_APP_REBUILD, 0, 0);
        assert!(!GetDlgItem(h, ID_FOLDER).is_null() && height(h).1 > off.1, "developer mode on");
        set_check(h, ID_DEV, false);
        SendMessageW(h, WM_APP_REBUILD, 0, 0);
        assert!(GetDlgItem(h, ID_FOLDER).is_null() && height(h) == off, "developer mode off");
        let name = wide("ImmersiveColorSet");
        let mut m = std::mem::zeroed::<MSG>();
        let pump = |m: &mut MSG| {
            while PeekMessageW(m, h, 0, 0, PM_REMOVE) != 0 {
                DispatchMessageW(m);
            }
        };
        let first = GetDlgItem(h, ID_ENABLED);
        // twice, as Windows does: the rebuild is posted, not done inside the sent message, and done once
        SendMessageW(h, WM_SETTINGCHANGE, 0, name.as_ptr() as isize);
        SendMessageW(h, WM_SETTINGCHANGE, 0, name.as_ptr() as isize);
        assert_eq!(first, GetDlgItem(h, ID_ENABLED), "rebuilt inside WM_SETTINGCHANGE");
        pump(&mut m);
        assert_eq!(PALETTE.load(Relaxed).cast_const(), wanted().map_or(null(), |p| p as *const Palette), "theme change not applied");
        let first = GetDlgItem(h, ID_ENABLED);
        SendMessageW(h, WM_SETTINGCHANGE, 0, name.as_ptr() as isize);
        pump(&mut m);
        assert_eq!(first, GetDlgItem(h, ID_ENABLED), "rebuilt although the look did not change");
        // a status of the same room only changes the text, also from none at all; one that needs a
        // second line rebuilds, posted
        pretend(None, Status::None);
        sync_update();
        pump(&mut m);
        let (first, empty) = (GetDlgItem(h, ID_ENABLED), height(h));
        pretend(None, Status::Checking);
        sync_update();
        pump(&mut m);
        assert_eq!((first, height(h)), (GetDlgItem(h, ID_ENABLED), empty), "rebuilt or resized for a status of the same room");
        assert_eq!(empty, off, "an empty status took other room than a one-line one");
        pretend(None, longest());
        sync_update();
        assert_eq!(first, GetDlgItem(h, ID_ENABLED), "rebuilt inside sync_update");
        pump(&mut m);
        assert!(height(h).1 > off.1, "no second status line");
        pretend(PENDING, waiting_status());
        sync_update();
        pump(&mut m);
        assert!(!GetDlgItem(h, ID_UPDATE_LINK).is_null() && height(h) == off, "waiting version");
        let to = RECT { left: 0, top: 0, right: 1, bottom: 1 };
        SendMessageW(h, WM_DPICHANGED, 144 | (144 << 16), &to as *const RECT as isize);
        assert_eq!(height(h).0, px(WIDTH, 144), "DPI change");
        DestroyWindow(h);
    }

    /// The debug information of this test process, made the way the button makes it; the clipboard
    /// is not touched. Ten lines, and the profile folder's name nowhere.
    unsafe fn debug_info() {
        let t = debug_now();
        let name = admin::profile_dir().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase())).unwrap_or_default();
        assert!(t.lines().count() == 10 && t.starts_with("TrioKey ") && !name.is_empty() && !t.to_lowercase().contains(&name), "debug information");
        println!("debug information: {} lines, {} bytes, profile folder name absent", t.lines().count(), t.len());
    }

    /// Rebuilds (theme, developer mode, DPI) and paints leave the GDI and USER object counts as
    /// they were: fonts, brushes and icons are freed or replaced, theme handles closed.
    unsafe fn leaks() {
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetGuiResources, GR_GDIOBJECTS, GR_USEROBJECTS};
        let count = || [GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS), GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS)];
        pretend(PENDING, waiting_status());
        let h = create(Some(&LIGHT), false, false, Some(96), null_mut());
        let round = |i: usize| {
            rebuild(h, [96, 144][i % 2], [Some(&LIGHT), Some(&DARK), None][i % 3], i.is_multiple_of(2), i % 4 < 2, (0, 0));
            focus(h, &[ID_UPDATE_LINK, ID_GITHUB]);
            capture(h, false);
        };
        (0..6).for_each(round);
        let before = count();
        (0..30).for_each(round);
        println!("GDI, USER objects after 6 rounds {before:?}, after 36 {:?}", count());
        assert_eq!(before, count(), "rebuilding or painting leaks");
        DestroyWindow(h);
    }

    /// The artificial states: a checked box, disabled check box and buttons, and the focus cue on a
    /// check box. Nothing here sends WM_COMMAND: no setting, file, registry value or clipboard is touched.
    unsafe fn states(h: HWND) {
        set_check(h, ID_YIELD, true);
        EnableWindow(GetDlgItem(h, ID_STARTUP), 0);
        EnableWindow(GetDlgItem(h, ID_CHECK), 0);
        EnableWindow(GetDlgItem(h, ID_DEBUG), 0);
        focus(h, &[ID_AUTO_EN]);
    }

    /// The keyboard focus cue on these controls, simulated: SetFocus would activate the window.
    unsafe fn focus(h: HWND, ids: &[i32]) {
        SendMessageW(h, WM_CHANGEUISTATE, (UIS_CLEAR | (UISF_HIDEFOCUS << 16)) as usize, 0);
        for &id in ids {
            SendMessageW(GetDlgItem(h, id), WM_SETFOCUS, 0, 0);
        }
    }

    /// The window at 96 DPI shown far off-screen, cloaked and without activation, owned by a hidden
    /// window so it gets no taskbar button, captured with its caption through DWM, then destroyed.
    /// Checks that the caption is the client area's colour.
    unsafe fn frame(p: &'static Palette, dev: bool, en: bool, tag: &str) -> (u32, u32, Vec<u8>) {
        let owner = CreateWindowExW(0, wide("STATIC").as_ptr(), null(), WS_POPUP, 0, 0, 0, 0, null_mut(), null_mut(), GetModuleHandleW(null()), null());
        let h = create(Some(p), dev, en, Some(96), owner);

        let x = GetSystemMetrics(SM_XVIRTUALSCREEN) + GetSystemMetrics(SM_CXVIRTUALSCREEN) + 400;
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN) + GetSystemMetrics(SM_CYVIRTUALSCREEN) + 400;
        SetWindowPos(h, null_mut(), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        let on = 1u32;
        let tr = DwmSetWindowAttribute(h, DWMWA_TRANSITIONS_FORCEDISABLED as u32, (&on as *const u32).cast(), 4);
        // cloaked: DWM keeps rendering it for PrintWindow but never puts it on screen
        let hc = DwmSetWindowAttribute(h, DWMWA_CLOAK as u32, (&on as *const u32).cast(), 4);
        assert_eq!(hc, 0, "cloak refused: not showing the window");
        let fg = GetForegroundWindow();
        let t = std::time::Instant::now();
        ShowWindow(h, SW_SHOWNOACTIVATE);
        UpdateWindow(h);
        let (mut win, mut origin) = (std::mem::zeroed::<RECT>(), windows_sys::Win32::Foundation::POINT { x: 0, y: 0 });
        GetWindowRect(h, &mut win);
        ClientToScreen(h, &mut origin);
        let bg = vec![p.bg as u8, (p.bg >> 8) as u8, (p.bg >> 16) as u8];
        // the first window DWM composes can take longer than one pass: capture until the client
        // margin shows the palette, for at most a second
        let (mut w, mut ht, mut rgb, mut m) = (0, 0, Vec::new(), std::mem::zeroed::<MSG>());
        let at = |rgb: &[u8], w: u32, x: i32, y: i32| rgb[((y as u32 * w + x as u32) * 3) as usize..][..3].to_vec();
        let body = (origin.x - win.left + 4, origin.y - win.top + 4);
        while t.elapsed() < std::time::Duration::from_secs(1) {
            let pass = std::time::Instant::now();
            while pass.elapsed() < std::time::Duration::from_millis(100) {
                while PeekMessageW(&mut m, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    DispatchMessageW(&m);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            DwmFlush();
            (w, ht, rgb) = capture(h, true);
            if at(&rgb, w, body.0, body.1) == bg {
                break;
            }
        }
        ShowWindow(h, SW_HIDE);
        let shown = t.elapsed();
        DestroyWindow(h);
        DestroyWindow(owner);
        println!("  frame {tag}: transitions-off {tr:x}, cloak {hc:x}, shown {shown:?}");
        assert_eq!(fg, GetForegroundWindow(), "the preview took the foreground");
        // middle of the caption, right of the title and left of the buttons, against the client margin
        let (cap, body) = (at(&rgb, w, w as i32 * 3 / 5, (origin.y - win.top) / 2), at(&rgb, w, body.0, body.1));
        assert_eq!([&cap, &body], [&bg, &bg], "{tag}: caption and body are not the palette's background");
        (w, ht, rgb)
    }
}
