//! The session event tap: sees every key-down and mouse-down, feeds the engine and re-types the word
//! (docs/ARCHITECTURE.md, 12. macOS app). It runs on the main run loop, so it shares the main thread with AppKit.
//!
//! This code swallows keys and was written without a Mac to run it on, so it fails open: a panic or an internal error
//! disables the tap and lets the event through unchanged.

use std::cell::RefCell;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null_mut, NonNull};

use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop, CFRunLoopSource};
use objc2_core_graphics::{CGEvent, CGEventField, CGEventFlags, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType};
use triokey_mac::keymap::{classify, vk, KeyClass};
use triokey_mac::plan::{chunks, plan, Plan};
use triokey_mac::source::Decision;

use super::app;

/// `kCGEventSourceUserData` on every event TrioKey posts, so another tap could tell them apart. The value must never change.
pub const TRIOKEY_MARK: i64 = 0x45564A31;

struct Tap {
    port: CFRetained<CFMachPort>,
    source: CFRetained<CFRunLoopSource>,
}

thread_local! {
    static TAP: RefCell<Option<Tap>> = const { RefCell::new(None) };
}

fn bit(t: CGEventType) -> u64 {
    1 << t.0
}

/// Creates the tap and adds it to the main run loop. `false`: macOS refused (no Accessibility permission yet). The only
/// trustworthy permission check: `AXIsProcessTrusted` and the preflight calls go stale while the process runs.
pub fn create() -> bool {
    if exists() {
        return true;
    }
    let mask = bit(CGEventType::KeyDown) | bit(CGEventType::LeftMouseDown) | bit(CGEventType::RightMouseDown) | bit(CGEventType::OtherMouseDown);
    // SAFETY: the callback matches CGEventTapCallBack and uses no user info
    let port = unsafe {
        CGEvent::tap_create(
            CGEventTapLocation::SessionEventTap,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            mask,
            Some(callback),
            null_mut(),
        )
    };
    let Some(port) = port else { return false };
    let (Some(source), Some(main)) = (CFMachPort::new_run_loop_source(None, Some(&port), 0), CFRunLoop::main()) else {
        port.invalidate();
        return false;
    };
    // SAFETY: a constant CoreFoundation exports
    main.add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
    CGEvent::tap_enable(&port, true);
    TAP.set(Some(Tap { port, source }));
    true
}

pub fn exists() -> bool {
    TAP.with_borrow(|t| t.is_some())
}

/// (created, enabled) for the diagnostics.
pub fn status() -> (bool, bool) {
    TAP.with_borrow(|t| t.as_ref().map_or((false, false), |t| (true, CGEvent::tap_is_enabled(&t.port))))
}

/// Stops the tap for good (an error, or quitting). Keeps the record so the diagnostics show it was created.
pub fn disable() {
    let _ = TAP.try_with(|t| {
        if let Some(t) = t.try_borrow().ok().as_deref().and_then(Option::as_ref) {
            CGEvent::tap_enable(&t.port, false);
            t.source.invalidate();
        }
    });
}

unsafe extern "C-unwind" fn callback(proxy: CGEventTapProxy, ty: CGEventType, event: NonNull<CGEvent>, _info: *mut c_void) -> *mut CGEvent {
    let pass = event.as_ptr();
    // SAFETY: CoreGraphics hands a valid event for the duration of the callback
    let ev = unsafe { event.as_ref() };
    match catch_unwind(AssertUnwindSafe(|| on_event(proxy, ty, ev))) {
        Ok(Ok(true)) => null_mut(),
        Ok(Ok(false)) => pass,
        Ok(Err(e)) => {
            app::fail(e, false);
            pass
        }
        Err(_) => {
            app::panicked("in the event tap");
            pass
        }
    }
}

/// `Ok(true)`: the event was replaced by posted ones and must be swallowed.
fn on_event(proxy: CGEventTapProxy, ty: CGEventType, ev: &CGEvent) -> Result<bool, String> {
    if ty == CGEventType::TapDisabledByTimeout || ty == CGEventType::TapDisabledByUserInput {
        TAP.with_borrow(|t| t.iter().for_each(|t| CGEvent::tap_enable(&t.port, true)));
        app::with(|a| if ty == CGEventType::TapDisabledByTimeout { a.counters.reenabled_timeout += 1 } else { a.counters.reenabled_user += 1 });
        return Ok(false);
    }
    if ty == CGEventType::LeftMouseDown || ty == CGEventType::RightMouseDown || ty == CGEventType::OtherMouseDown {
        // a click may move the caret
        app::with(|a| a.engine.reset());
        return Ok(false);
    }
    if ty != CGEventType::KeyDown {
        return Ok(false);
    }
    let ev = Some(ev);
    let keycode = CGEvent::integer_value_field(ev, CGEventField::KeyboardEventKeycode) as u16;
    let autorepeat = CGEvent::integer_value_field(ev, CGEventField::KeyboardEventAutorepeat) != 0;
    let flags = CGEvent::flags(ev).0;
    let mut buf = [0u16; 8];
    let mut len = 0;
    // SAFETY: both pointers are valid for the given capacity
    unsafe { CGEvent::keyboard_get_unicode_string(ev, buf.len() as _, &mut len, buf.as_mut_ptr()) };
    let text = String::from_utf16_lossy(&buf[..(len as usize).min(buf.len())]);
    let todo = app::with(|a| {
        a.counters.keys += 1;
        if !a.on || a.source.decision() == Decision::Yield {
            a.engine.reset();
            return None;
        }
        let class = classify(keycode, &text, flags, autorepeat);
        let e = &mut a.engine;
        let out = match class {
            KeyClass::Letter(c) => e.key(c),
            KeyClass::Boundary(c) => e.boundary(c),
            KeyClass::Backspace => e.backspace(),
            KeyClass::BreakWord => {
                e.reset();
                return None;
            }
            KeyClass::Ignore => return None,
        };
        plan(out, class)
    });
    let Some(p) = todo else { return Ok(false) };
    // every event is built before any is posted: a failure half way must not leave half a word on screen
    let events = build(&p, CGEventFlags(flags) & (CGEventFlags::MaskShift | CGEventFlags::MaskAlternate))?;
    for e in &events {
        // SAFETY: `proxy` is the one this callback received; posted here, the events reach only later taps, never this one
        unsafe { CGEvent::tap_post_event(proxy, Some(e)) };
    }
    app::with(|a| {
        let c = &mut a.counters;
        c.replaced += 1;
        c.backspaces += p.backspaces as u64;
        c.chars += p.text.chars().count() as u64;
    });
    Ok(true)
}

fn key(code: u16, down: bool, flags: CGEventFlags) -> Result<CFRetained<CGEvent>, String> {
    let e = CGEvent::new_keyboard_event(None, code, down).ok_or("CGEventCreateKeyboardEvent returned NULL")?;
    CGEvent::set_flags(Some(&e), flags);
    CGEvent::set_integer_value_field(Some(&e), CGEventField::EventSourceUserData, TRIOKEY_MARK);
    Ok(e)
}

/// Backspaces, the text in chunks (key code 0 carrying the Unicode string), then the real Return or Tab. The person may
/// still hold Shift or Option: backspaces and text carry no flags, while the real key keeps the original event's Shift
/// and Option (`flags`; Shift+Return is a new line in chat apps where Return sends).
fn build(p: &Plan, flags: CGEventFlags) -> Result<Vec<CFRetained<CGEvent>>, String> {
    let none = CGEventFlags::empty();
    let mut v = Vec::new();
    for _ in 0..p.backspaces {
        v.extend([key(vk::DELETE, true, none)?, key(vk::DELETE, false, none)?]);
    }
    for c in chunks(&p.text) {
        for down in [true, false] {
            let e = key(0, down, none)?;
            // SAFETY: `c` holds `c.len()` UTF-16 units
            unsafe { CGEvent::keyboard_set_unicode_string(Some(&e), c.len() as _, c.as_ptr()) };
            v.push(e);
        }
    }
    if let Some(k) = p.key {
        v.extend([key(k, true, flags)?, key(k, false, flags)?]);
    }
    Ok(v)
}
