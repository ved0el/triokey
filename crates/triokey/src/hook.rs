//! Low-level keyboard (and mouse) hook on its own thread (spec 5.1, 5.5, 5.8).

use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering::Relaxed};
use std::sync::OnceLock;
use std::thread::Thread;
use std::time::{Duration, Instant};

use triokey_engine::lexicon::Lexicon;
use triokey_engine::{Edit, Engine, Outcome};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::config::Hotkey;
use crate::ime::{first_key_wait, ImeState, FIRST_KEY_WAIT_MS};
use crate::inject::{build_inputs, tap, TRIOKEY_MARK};
use crate::keymap::{classify, KeyClass};
use crate::{app, ime_watch};

const WM_APP_REINSTALL: u32 = WM_APP + 2;
const WM_APP_RECONFIGURE: u32 = WM_APP + 5;
/// Input newer than our last callback by this much: Windows has dropped the hook (spec 5.8).
const DEAD_AFTER_MS: i32 = 1000;
/// `ToUnicodeEx` flag: do not change the keyboard state (keeps dead keys intact).
const TU_NO_STATE_CHANGE: u32 = 0x4;
const RING: usize = 1024;

/// `time` of the last event either hook saw (tick clock, same as `GetLastInputInfo`).
static LAST_EVENT: AtomicU32 = AtomicU32::new(0);
static HOOK_TID: AtomicU32 = AtomicU32::new(0);
static HOOK_THREAD: OnceLock<Thread> = OnceLock::new();
/// Callback durations in µs for AT7, shown in the tray menu. Memory only. Keys that called
/// `SendInput` (time includes the call) and keys that did not are kept apart.
static SENT: Ring = Ring::new();
static PASSED: Ring = Ring::new();

struct Ring {
    times: [AtomicU32; RING],
    count: AtomicUsize,
}

impl Ring {
    const fn new() -> Ring {
        Ring { times: [const { AtomicU32::new(0) }; RING], count: AtomicUsize::new(0) }
    }

    fn record(&self, us: u32) {
        self.times[self.count.fetch_add(1, Relaxed) % RING].store(us, Relaxed);
    }

    /// (p99 in µs over the last `RING` keys, keys measured since start).
    fn p99(&self) -> (u32, usize) {
        let total = self.count.load(Relaxed);
        let mut v: Vec<u32> = self.times[..total.min(RING)].iter().map(|t| t.load(Relaxed)).collect();
        (p99(&mut v), total)
    }
}

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
    static HOOKS: Cell<(HHOOK, HHOOK)> = const { Cell::new((null_mut(), null_mut())) };
    static HOT: Cell<HotState> = const { Cell::new(HotState { combo: false, swallowed: 0 }) };
    /// When the last key/click that may have toggled the IME happened.
    static TOGGLED_AT: Cell<Option<Instant>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hot {
    None,
    /// Toggle, let the key through.
    Toggle,
    /// Toggle and swallow the key.
    ToggleSwallow,
    /// Swallow only: an auto-repeat or the release of a swallowed hotkey press.
    Swallow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HotState {
    /// Ctrl+Shift pressed together with nothing else so far.
    combo: bool,
    /// The key of a swallowed hotkey press (Z, Space) that has not been released yet, else 0.
    swallowed: u16,
}

/// Unassigned virtual key, sent while Alt is held after Alt+Z was swallowed. The app never saw
/// the Z, and Alt pressed and released with no key in between opens its menu bar.
const MASK_VK: u16 = 0xE8;

fn is_ctrl(vk: u16) -> bool {
    matches!(vk, VK_CONTROL | VK_LCONTROL | VK_RCONTROL)
}

fn is_shift(vk: u16) -> bool {
    matches!(vk, VK_SHIFT | VK_LSHIFT | VK_RSHIFT)
}

/// `m` is the state of the other modifiers when this event arrives. Ctrl+Shift fires on
/// release, only if no other key came in between (Ctrl+Shift+T stays a shortcut).
pub fn hotkey_step(st: &mut HotState, hk: Hotkey, vk: u16, down: bool, m: Mods) -> Hot {
    if st.swallowed != 0 {
        // Held: Windows repeats the key-down, which must not toggle again. The app never saw
        // the press, so it gets neither the repeats nor the release, even when the modifier
        // was released first.
        if vk == st.swallowed {
            if !down {
                st.swallowed = 0;
            }
            return Hot::Swallow;
        }
        // another key pressed: repeats have stopped; do not wait for a release that may be lost
        if down {
            st.swallowed = 0;
        }
    }
    // Shift excluded: Ctrl+Shift+Space and Alt+Shift+Z are other apps' shortcuts
    let only = |a: bool, b: bool| a && !b && !m.shift && !m.win;
    let hit = match hk {
        Hotkey::AltZ => down && vk == 0x5A && only(m.alt, m.ctrl),
        Hotkey::CtrlSpace => down && vk == VK_SPACE && only(m.ctrl, m.alt),
        _ => false,
    };
    if hit {
        st.swallowed = vk;
        return Hot::ToggleSwallow;
    }
    let combo = &mut st.combo;
    match hk {
        Hotkey::CtrlShift => {
            let (c, s) = (is_ctrl(vk), is_shift(vk));
            if down {
                if ((c && m.shift) || (s && m.ctrl)) && !m.alt && !m.win {
                    *combo = true;
                } else if !(c || s) {
                    *combo = false;
                }
                Hot::None
            } else if (c || s) && std::mem::take(combo) {
                Hot::Toggle
            } else {
                Hot::None
            }
        }
        _ => Hot::None,
    }
}

/// Enter/Tab sent as Unicode chars do not act as Enter/Tab in most apps: send the word
/// without them and a real key press after it.
pub fn split_boundary(mut edit: Edit, ch: char) -> (Edit, Option<u16>) {
    let vk = match ch {
        '\n' => VK_RETURN,
        '\t' => VK_TAB,
        _ => return (edit, None),
    };
    if !edit.insert.ends_with(ch) {
        return (edit, None);
    }
    edit.insert.pop();
    (edit, Some(vk))
}

pub fn looks_dead(last_input: u32, last_event: u32) -> bool {
    last_input.wrapping_sub(last_event) as i32 > DEAD_AFTER_MS
}

pub fn p99(samples: &mut [u32]) -> u32 {
    if samples.is_empty() {
        return 0;
    }
    samples.sort_unstable();
    samples[(samples.len() * 99).div_ceil(100) - 1]
}

/// ((p99, keys) that sent input, (p99, keys) that did not), p99 in µs.
pub fn latency() -> ((u32, usize), (u32, usize)) {
    (SENT.p99(), PASSED.p99())
}

/// The engine has no config setter: it is rebuilt from this copy when the settings change.
pub fn spawn(lexicon: Lexicon) {
    std::thread::spawn(move || unsafe {
        // builds the two lazy word tables (39k entries) now, not inside the first key's callback
        let _ = (lexicon.is_english(""), lexicon.tie_default(""));
        let build = || Engine::new(app::engine_config(&app::settings()), lexicon.clone());
        ENGINE.with_borrow_mut(|e| *e = Some(build()));
        let _ = HOOK_THREAD.set(std::thread::current());
        let mut msg: MSG = std::mem::zeroed();
        // create the message queue before anyone can post to this thread
        PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
        HOOK_TID.store(GetCurrentThreadId(), Relaxed);
        install();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if msg.message == WM_APP_REINSTALL {
                uninstall();
                install();
            } else if msg.message == WM_APP_RECONFIGURE {
                ENGINE.with_borrow_mut(|e| *e = Some(build()));
            }
        }
        uninstall();
    });
}

/// Called by the watcher thread every tick.
pub fn check_alive(last_input: u32) {
    let tid = HOOK_TID.load(Relaxed);
    if tid != 0 && looks_dead(last_input, LAST_EVENT.load(Relaxed)) {
        // one reinstall per burst of unseen input
        LAST_EVENT.store(last_input, Relaxed);
        unsafe { PostThreadMessageW(tid, WM_APP_REINSTALL, 0, 0) };
    }
}

/// Settings changed (tone style, auto English): rebuild the engine on the hook thread.
pub fn reconfigure() {
    let tid = HOOK_TID.load(Relaxed);
    if tid != 0 {
        unsafe { PostThreadMessageW(tid, WM_APP_RECONFIGURE, 0, 0) };
    }
}

/// The watcher finished a recheck: stop waiting.
pub fn wake() {
    if let Some(t) = HOOK_THREAD.get() {
        t.unpark();
    }
}

unsafe fn install() {
    let hmod = GetModuleHandleW(std::ptr::null());
    let kb = SetWindowsHookExW(WH_KEYBOARD_LL, Some(kb_proc), hmod, 0);
    let ms = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), hmod, 0);
    HOOKS.set((kb, ms));
}

unsafe fn uninstall() {
    let (kb, ms) = HOOKS.replace((null_mut(), null_mut()));
    for h in [kb, ms] {
        if !h.is_null() {
            UnhookWindowsHookEx(h);
        }
    }
}

fn held(vk: u16) -> bool {
    unsafe { GetAsyncKeyState(vk as i32) < 0 }
}

fn mods() -> Mods {
    Mods { ctrl: held(VK_CONTROL), shift: held(VK_SHIFT), alt: held(VK_MENU), win: held(VK_LWIN) || held(VK_RWIN) }
}

fn reset_engine() {
    ENGINE.with_borrow_mut(|e| {
        if let Some(e) = e.as_mut() {
            e.reset();
        }
    });
}

unsafe extern "system" fn mouse_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0 {
        LAST_EVENT.store((*(lp as *const MSLLHOOKSTRUCT)).time, Relaxed);
        if matches!(wp as u32, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN) {
            // a click may move the caret (spec 5.4) or toggle the IME (spec 5.3)
            ime_watch::break_word();
            HOT.set(HotState { combo: false, ..HOT.get() });
            TOGGLED_AT.set(Some(Instant::now()));
            ime_watch::request_recheck();
        }
    }
    CallNextHookEx(null_mut(), code, wp, lp)
}

unsafe extern "system" fn kb_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(null_mut(), code, wp, lp);
    }
    let k = &*(lp as *const KBDLLHOOKSTRUCT);
    LAST_EVENT.store(k.time, Relaxed);
    // our own SendInput comes back through here, before any state is touched
    if k.dwExtraInfo == TRIOKEY_MARK {
        return CallNextHookEx(null_mut(), code, wp, lp);
    }
    // keys injected by a lower-integrity process must never make an elevated TrioKey send text
    if k.flags & LLKHF_LOWER_IL_INJECTED != 0 {
        reset_engine();
        // it still breaks a Ctrl+Shift tap, as an unseen key would (keeps on_key's rule)
        HOT.set(HotState { combo: false, ..HOT.get() });
        return CallNextHookEx(null_mut(), code, wp, lp);
    }
    let start = Instant::now();
    let is_down = matches!(wp as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let (swallow, waited, sent) = on_key(k, is_down);
    // spec 5.8: the first key after an IME toggle is excluded from the p99
    if is_down && !waited {
        let us = start.elapsed().as_micros().min(u32::MAX as u128) as u32;
        if sent {
            SENT.record(us)
        } else {
            PASSED.record(us)
        }
    }
    if swallow {
        1
    } else {
        CallNextHookEx(null_mut(), code, wp, lp)
    }
}

/// Returns (swallow the key, waited for a fresh IME read, called `SendInput`).
unsafe fn on_key(k: &KBDLLHOOKSTRUCT, is_down: bool) -> (bool, bool, bool) {
    let vk = k.vkCode as u16;
    let m = mods();
    let mut st = HOT.get();
    let hot = hotkey_step(&mut st, app::settings().hotkey, vk, is_down, m);
    HOT.set(st);
    match hot {
        Hot::None => {}
        Hot::Toggle => app::toggle(),
        Hot::ToggleSwallow => {
            app::toggle();
            if m.alt {
                let mask = tap(MASK_VK);
                SendInput(mask.len() as u32, mask.as_ptr(), size_of::<INPUT>() as i32);
            }
            return (true, false, m.alt);
        }
        Hot::Swallow => return (true, false, false),
    }
    if !is_down {
        return (false, false, false);
    }
    // Unicode typed straight in by other software is not a Telex key (spec 5.4)
    if vk == VK_PACKET {
        reset_engine();
        return (false, false, false);
    }
    let tid = GetWindowThreadProcessId(GetForegroundWindow(), null_mut());
    let ch = to_char(vk, k.scanCode, m, GetKeyboardLayout(tid));
    let class = classify(vk, ch, m.ctrl, m.alt, m.win);
    // before the Yield check, so the key after turning hiragana off sees the new state
    if matches!(class, KeyClass::MaybeImeToggle | KeyClass::BreakWord) {
        TOGGLED_AT.set(Some(Instant::now()));
        ime_watch::request_recheck();
    }
    if ime_watch::take_word_break() {
        reset_engine();
    }
    if !app::enabled() || ime_watch::password() {
        return (false, false, false);
    }
    let typing = matches!(class, KeyClass::Letter(_) | KeyClass::Boundary(_) | KeyClass::Backspace);
    let waited = typing && wait_for_fresh_state();
    if ime_watch::current() == ImeState::Yield {
        reset_engine();
        return (false, waited, false);
    }
    let out = ENGINE.with_borrow_mut(|e| {
        let Some(e) = e.as_mut() else { return Outcome::Pass };
        match class {
            // AltGr chars: injecting Backspace while Ctrl+Alt are held is not safe
            KeyClass::Letter(_) | KeyClass::Boundary(_) if m.ctrl || m.alt => {
                e.reset();
                Outcome::Pass
            }
            KeyClass::Letter(c) => e.key(c),
            KeyClass::Boundary(c) => e.boundary(c),
            KeyClass::Backspace => e.backspace(),
            KeyClass::BreakWord | KeyClass::MaybeImeToggle => {
                e.reset();
                Outcome::Pass
            }
            KeyClass::Ignore => Outcome::Pass,
        }
    });
    // the engine borrow is released here: SendInput may re-enter this hook
    let Outcome::Replace(edit) = out else { return (false, waited, false) };
    let (edit, vk) = match class {
        KeyClass::Boundary(c) => split_boundary(edit, c),
        _ => (edit, None),
    };
    let mut inputs = build_inputs(&edit);
    inputs.extend(vk.map(tap).into_iter().flatten());
    SendInput(inputs.len() as u32, inputs.as_ptr(), size_of::<INPUT>() as i32);
    (true, waited, true)
}

/// Spec 5.3: the first typing key after a possible toggle may wait up to 30 ms for the
/// watcher's fresh read; past that the cached value is used.
fn wait_for_fresh_state() -> bool {
    let Some(t) = TOGGLED_AT.take() else { return false };
    let limit = Duration::from_millis(FIRST_KEY_WAIT_MS as u64);
    let mut waited = false;
    while ime_watch::pending() && first_key_wait(t.elapsed().as_millis().min(u32::MAX as u128) as u32) {
        waited = true;
        // shortcut: park_timeout may oversleep by a timer tick (~15 ms) at the deadline; the
        // watcher's unpark ends the wait early in the normal case. Upgrade if AT3/AT7 show it.
        std::thread::park_timeout(limit.saturating_sub(t.elapsed()));
    }
    waited
}

/// What the key types on the foreground thread's layout. The hook thread's own key state
/// is not the foreground's, so it is built from the async state.
unsafe fn to_char(vk: u16, scan: u32, m: Mods, hkl: HKL) -> Option<char> {
    let mut ks = [0u8; 256];
    for (on, key) in [(m.shift, VK_SHIFT), (m.ctrl, VK_CONTROL), (m.alt, VK_MENU)] {
        if on {
            ks[key as usize] = 0x80;
        }
    }
    // shortcut: Caps Lock toggle read with GetKeyState on the hook thread; if AT1 shows it
    // stale, track VK_CAPITAL presses in the hook instead
    if GetKeyState(VK_CAPITAL as i32) & 1 != 0 {
        ks[VK_CAPITAL as usize] = 1;
    }
    let mut buf = [0u16; 4];
    let n = ToUnicodeEx(vk as u32, scan, ks.as_ptr(), buf.as_mut_ptr(), buf.len() as i32, TU_NO_STATE_CHANGE, hkl);
    if n == 1 {
        char::from_u32(buf[0] as u32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Hotkey;
    use triokey_engine::Edit;

    #[test]
    fn ctrl_shift_toggles_on_release_only_when_alone() {
        let mut c = HotState::default();
        // Ctrl down, Shift down (Ctrl held), Shift up -> toggle; Ctrl up -> nothing
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_LCONTROL, true, Mods::default()), Hot::None);
        let ctrl = Mods { ctrl: true, ..Mods::default() };
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_LSHIFT, true, ctrl), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_LSHIFT, false, ctrl), Hot::Toggle);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_LCONTROL, false, Mods::default()), Hot::None);
        // Shift first, then Ctrl
        let shift = Mods { shift: true, ..Mods::default() };
        hotkey_step(&mut c, Hotkey::CtrlShift, VK_RSHIFT, true, Mods::default());
        hotkey_step(&mut c, Hotkey::CtrlShift, VK_RCONTROL, true, shift);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_RCONTROL, false, shift), Hot::Toggle);
        // Ctrl+Shift+T is a shortcut, not the toggle
        let both = Mods { ctrl: true, shift: true, ..Mods::default() };
        hotkey_step(&mut c, Hotkey::CtrlShift, VK_LCONTROL, true, Mods::default());
        hotkey_step(&mut c, Hotkey::CtrlShift, VK_LSHIFT, true, ctrl);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, 0x54, true, both), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_LSHIFT, false, ctrl), Hot::None);
        // Shift alone never toggles
        hotkey_step(&mut c, Hotkey::CtrlShift, VK_LSHIFT, true, Mods::default());
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_LSHIFT, false, Mods::default()), Hot::None);
        // other hotkey settings ignore Ctrl+Shift
        let mut c = HotState::default();
        hotkey_step(&mut c, Hotkey::AltZ, VK_LCONTROL, true, Mods::default());
        hotkey_step(&mut c, Hotkey::AltZ, VK_LSHIFT, true, ctrl);
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, VK_LSHIFT, false, ctrl), Hot::None);
    }

    #[test]
    fn alt_z_and_ctrl_space_swallow_keydown() {
        let mut c = HotState::default();
        let alt = Mods { alt: true, ..Mods::default() };
        let ctrl = Mods { ctrl: true, ..Mods::default() };
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, alt), Hot::ToggleSwallow);
        let mut c = HotState::default();
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, Mods::default()), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, Mods { ctrl: true, alt: true, ..Mods::default() }), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, true, ctrl), Hot::ToggleSwallow);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, false, ctrl), Hot::Swallow);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, false, ctrl), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, true, alt), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlShift, VK_SPACE, true, ctrl), Hot::None);
    }

    #[test]
    fn held_hotkey_toggles_once_and_swallows_its_repeats_and_release() {
        let (alt, none) = (Mods { alt: true, ..Mods::default() }, Mods::default());
        let mut c = HotState::default();
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, alt), Hot::ToggleSwallow);
        // auto-repeat
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, alt), Hot::Swallow);
        // Alt released first: its release passes, later Z repeats and the Z release do not
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, VK_LMENU, false, none), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, none), Hot::Swallow);
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, false, none), Hot::Swallow);
        // released: a plain z is typed, its release is not swallowed, the next press toggles
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, none), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, false, none), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, alt), Hot::ToggleSwallow);
        // another key pressed while held: a lost Z release cannot eat a later z
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x41, true, alt), Hot::None);
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, none), Hot::None);

        let ctrl = Mods { ctrl: true, ..Mods::default() };
        let mut c = HotState::default();
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, true, ctrl), Hot::ToggleSwallow);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, true, ctrl), Hot::Swallow);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, false, ctrl), Hot::Swallow);
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, true, ctrl), Hot::ToggleSwallow);
    }

    #[test]
    fn swallowing_presets_leave_shift_shortcuts_alone() {
        let mut c = HotState::default();
        let m = Mods { ctrl: true, shift: true, ..Mods::default() };
        assert_eq!(hotkey_step(&mut c, Hotkey::CtrlSpace, VK_SPACE, true, m), Hot::None);
        let m = Mods { alt: true, shift: true, ..Mods::default() };
        assert_eq!(hotkey_step(&mut c, Hotkey::AltZ, 0x5A, true, m), Hot::None);
    }

    #[test]
    fn enter_and_tab_become_real_keys() {
        let e = |d, s: &str| Edit { delete: d, insert: s.into() };
        assert_eq!(split_boundary(e(3, "thí\n"), '\n'), (e(3, "thí"), Some(VK_RETURN)));
        assert_eq!(split_boundary(e(1, "a\t"), '\t'), (e(1, "a"), Some(VK_TAB)));
        assert_eq!(split_boundary(e(2, "mix "), ' '), (e(2, "mix "), None));
        assert_eq!(split_boundary(e(2, "mix."), '.'), (e(2, "mix."), None));
    }

    #[test]
    fn hook_dead_when_input_outruns_callbacks() {
        assert!(!looks_dead(1000, 1000));
        assert!(!looks_dead(1500, 1000));
        assert!(looks_dead(2500, 1000));
        // callback newer than last input (clock skew) is alive
        assert!(!looks_dead(1000, 1200));
        // tick counter wraps after ~49.7 days
        assert!(!looks_dead(100, u32::MAX - 100));
        assert!(looks_dead(5000, u32::MAX - 100));
    }

    #[test]
    fn ring_keeps_last_samples() {
        let r = Ring::new();
        assert_eq!(r.p99(), (0, 0));
        // older samples are overwritten once the ring is full
        for _ in 0..RING {
            r.record(5000);
        }
        for i in 0..RING as u32 {
            r.record(i % 100 + 1);
        }
        // none of the 5000s are left; 1..=100 over 1024 samples puts the p99 at 99
        assert_eq!(r.p99(), (99, 2 * RING));
        let r = Ring::new();
        r.record(7);
        assert_eq!(r.p99(), (7, 1));
    }

    #[test]
    fn p99_of_samples() {
        assert_eq!(p99(&mut []), 0);
        let mut v: Vec<u32> = (1..=100).rev().collect();
        assert_eq!(p99(&mut v), 99);
        let mut one = [7];
        assert_eq!(p99(&mut one), 7);
    }
}
