//! IME watcher thread (docs/ARCHITECTURE.md, 3.1 Reading the state from another process): keeps the cached `ImeState` the hook reads, and the
//! "focus is a Win32 password box" flag. The key path never asks the IME itself.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, Ordering::Relaxed};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows_sys::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::ime::{decide, is_cjk, ImeState};
use crate::{app, hook, tray};

const WM_APP_RECHECK: u32 = WM_APP + 1;
const IMC_GETCONVERSIONMODE: usize = 0x1;
const IMC_GETOPENSTATUS: usize = 0x5;
const LANG_JAPANESE: u16 = 0x11;
/// The app handles the toggle key after the hook saw it; read again this much later.
const RECHECK_DELAY_MS: u32 = 15;
const TICK_MS: u32 = 100;
/// One read measured 0.1-3 ms, worst seen 28 ms.
const READ_TIMEOUT_MS: u32 = 50;

const TYPE: u8 = 0;
const YIELD: u8 = 1;
/// Unknown until the first read: yield.
static STATE: AtomicU8 = AtomicU8::new(YIELD);
/// Primary language of the foreground layout at the last read (0 before it), for the tooltip.
static LANG: AtomicU16 = AtomicU16::new(0);
static PASSWORD: AtomicBool = AtomicBool::new(false);
/// A recheck was asked for and has not finished yet.
static PENDING: AtomicBool = AtomicBool::new(false);
/// Focus, IME state or the on/off switch changed: the hook drops the word being typed.
static WORD_BREAK: AtomicBool = AtomicBool::new(false);
static TID: AtomicU32 = AtomicU32::new(0);

#[derive(Default)]
struct Watch {
    /// Last known state per top-level window, used when a read fails.
    last: HashMap<usize, ImeState>,
    focus: usize,
}

thread_local! {
    static WATCH: RefCell<Watch> = RefCell::default();
    static BUSY: Cell<bool> = const { Cell::new(false) };
    static AGAIN: Cell<bool> = const { Cell::new(false) };
}

pub fn current() -> ImeState {
    if STATE.load(Relaxed) == YIELD {
        ImeState::Yield
    } else {
        ImeState::Type
    }
}

pub fn lang() -> u16 {
    LANG.load(Relaxed)
}

pub fn password() -> bool {
    PASSWORD.load(Relaxed)
}

pub fn pending() -> bool {
    PENDING.load(Relaxed)
}

pub fn break_word() {
    WORD_BREAK.store(true, Relaxed);
}

/// Consumed by the hook thread.
pub fn take_word_break() -> bool {
    WORD_BREAK.swap(false, Relaxed)
}

/// Something may have toggled the IME: read it again once the app has handled the input.
pub fn request_recheck() {
    PENDING.store(true, Relaxed);
    let tid = TID.load(Relaxed);
    if tid == 0 || unsafe { PostThreadMessageW(tid, WM_APP_RECHECK, 0, 0) } == 0 {
        PENDING.store(false, Relaxed);
    }
}

/// `ES_PASSWORD` (0x20) means something else on non-edit classes, so the class must say edit.
pub fn is_password(class: &str, style: u32) -> bool {
    style & ES_PASSWORD as u32 != 0 && class.to_ascii_lowercase().contains("edit")
}

pub fn spawn() {
    std::thread::spawn(|| unsafe { run() });
}

unsafe fn run() {
    let mut msg: MSG = std::mem::zeroed();
    // create the message queue before anyone can post to this thread
    PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
    TID.store(GetCurrentThreadId(), Relaxed);
    let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
    for ev in [EVENT_SYSTEM_FOREGROUND, EVENT_OBJECT_FOCUS] {
        SetWinEventHook(ev, ev, null_mut(), Some(on_event), 0, 0, flags);
    }
    let tick = SetTimer(null_mut(), 0, TICK_MS, None);
    let mut recheck = 0;
    refresh();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        match msg.message {
            // same id again restarts the one-shot timer
            WM_APP_RECHECK => recheck = SetTimer(null_mut(), recheck, RECHECK_DELAY_MS, None),
            WM_TIMER if recheck != 0 && msg.wParam == recheck => {
                KillTimer(null_mut(), recheck);
                recheck = 0;
                refresh();
                PENDING.store(false, Relaxed);
                hook::wake();
            }
            WM_TIMER if msg.wParam == tick => {
                refresh();
                let mut lii = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
                if GetLastInputInfo(&mut lii) != 0 {
                    hook::check_alive(lii.dwTime);
                }
                app::save_learned(false);
            }
            _ => {}
        }
    }
}

unsafe extern "system" fn on_event(_: HWINEVENTHOOK, _: u32, _: HWND, _: i32, _: i32, _: u32, _: u32) {
    refresh();
}

unsafe fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32).max(0) as usize;
    String::from_utf16_lossy(&buf[..n])
}

/// `WM_IME_CONTROL` probe (`IMC_GETOPENSTATUS` or `IMC_GETCONVERSIONMODE`); `None` when refused, hung or timed out.
unsafe fn query(ime: HWND, cmd: usize) -> Option<usize> {
    if ime.is_null() {
        return None;
    }
    let mut r = 0;
    let ok = SendMessageTimeoutW(ime, WM_IME_CONTROL, cmd, 0, SMTO_ABORTIFHUNG, READ_TIMEOUT_MS, &mut r);
    (ok != 0).then_some(r)
}

/// `SendMessageTimeoutW` inside a read can deliver another WinEvent, re-entering here:
/// the nested call only asks for one more pass instead of borrowing `WATCH` twice.
unsafe fn refresh() {
    if BUSY.replace(true) {
        AGAIN.set(true);
        return;
    }
    loop {
        AGAIN.set(false);
        refresh_once();
        if !AGAIN.get() {
            break;
        }
    }
    BUSY.set(false);
}

unsafe fn refresh_once() {
    let fg = GetForegroundWindow();
    if fg.is_null() {
        return;
    }
    let tid = GetWindowThreadProcessId(fg, null_mut());
    let mut gti: GUITHREADINFO = std::mem::zeroed();
    gti.cbSize = size_of::<GUITHREADINFO>() as u32;
    let focus = if GetGUIThreadInfo(tid, &mut gti) != 0 && !gti.hwndFocus.is_null() { gti.hwndFocus } else { fg };
    PASSWORD.store(is_password(&class_name(focus), GetWindowLongW(focus, GWL_STYLE) as u32), Relaxed);
    let lang = (GetKeyboardLayout(tid) as usize & 0x3FF) as u16;
    let state = WATCH.with_borrow_mut(|w| {
        let focus_changed = w.focus != focus as usize;
        w.focus = focus as usize;
        if focus_changed {
            break_word();
        }
        if !app::settings().yield_cjk || !is_cjk(lang) {
            return ImeState::Type;
        }
        let ime = ImmGetDefaultIMEWnd(focus);
        let open = query(ime, IMC_GETOPENSTATUS).map(|v| v != 0);
        let conv = if lang != LANG_JAPANESE && open == Some(true) { query(ime, IMC_GETCONVERSIONMODE).map(|v| v as u32) } else { None };
        let s = decide(lang, open, conv, w.last.get(&(fg as usize)).copied());
        // shortcut: forget all windows past 256 entries; fine for a desktop session
        if w.last.len() > 256 {
            w.last.clear();
        }
        // a failed read yields the old value (or Yield), so storing it changes nothing
        w.last.insert(fg as usize, s);
        s
    });
    let new = if state == ImeState::Yield { YIELD } else { TYPE };
    let lang_changed = LANG.swap(lang, Relaxed) != lang;
    let flipped = STATE.swap(new, Relaxed) != new;
    if flipped {
        break_word();
    }
    // still yielding, but to another language: the tooltip names it
    if flipped || (lang_changed && new == YIELD) {
        tray::notify();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_needs_edit_class() {
        assert!(is_password("Edit", 0x20));
        assert!(is_password("RICHEDIT50W", 0x50010020));
        assert!(is_password("TEdit", 0x20));
        assert!(!is_password("Edit", 0x10));
        // 0x20 means something else on other classes (BS_LEFTTEXT on buttons)
        assert!(!is_password("Button", 0x20));
        assert!(!is_password("Chrome_RenderWidgetHostHWND", 0x20));
    }
}
