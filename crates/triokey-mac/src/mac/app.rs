//! State shared by the tap and the menu, start-up and the 2-second timer. All of it lives on the main thread.

use std::cell::RefCell;
use std::io::Write;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSRunningApplication};
use objc2_application_services::{kAXTrustedCheckOptionPrompt, AXIsProcessTrusted, AXIsProcessTrustedWithOptions};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};
use objc2_foundation::{NSBundle, NSProcessInfo, NSString};
use triokey_engine::lexicon::Lexicon;
use triokey_engine::{Config, Engine};
use triokey_mac::diag::{Counters, Diagnostics};
use triokey_mac::source::{remember, Source};
use triokey_mac::text::{self, State};
use triokey_mac::{BUNDLE_ID, VERSION};

use super::{tap, tis, ui};

pub struct App {
    pub engine: Engine,
    pub on: bool,
    pub source: Source,
    pub seen: Vec<Source>,
    pub secure: bool,
    pub failed: bool,
    pub counters: Counters,
    pub last_error: Option<String>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    /// Where the last panic happened, set by the panic hook.
    static PANIC_AT: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn with<R>(f: impl FnOnce(&mut App) -> R) -> R {
    APP.with_borrow_mut(|a| f(a.as_mut().expect("app state is set before the tap and the menu exist")))
}

/// Keeps only the place of a panic: its message can quote the text being handled, and typed text is never kept or printed.
/// The self-test sees no typed text and prints the whole message (objc2's type checks name the selector there).
fn panic_hook(self_test: bool) {
    std::panic::set_hook(Box::new(move |info| {
        let at = info.location().map_or_else(|| "?".into(), |l| format!("{}:{}", l.file(), l.line()));
        // not eprintln!: it panics when stderr fails, and a panic inside the hook aborts the process
        let _ = if self_test { writeln!(std::io::stderr(), "TrioKey: {info}") } else { writeln!(std::io::stderr(), "TrioKey: panic at {at}") };
        let _ = PANIC_AT.try_with(|p| p.try_borrow_mut().map(|mut p| *p = Some(at)));
    }));
}

/// A panic was caught (`catch_unwind`) in `context`.
pub fn panicked(context: &str) {
    let at = PANIC_AT.try_with(|p| p.try_borrow_mut().ok().and_then(|mut p| p.take())).ok().flatten().unwrap_or_else(|| "?".into());
    fail(format!("panic {context} at {at}"), true);
}

pub fn run(self_test: bool) -> Result<(), String> {
    panic_hook(self_test);
    let mtm = MainThreadMarker::new().ok_or("not started on the main thread")?;
    let app = NSApplication::sharedApplication(mtm);
    if another_instance() {
        return if self_test { Err("another TrioKey is running".into()) } else { Ok(()) };
    }
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    // build every table before the tap exists: a throwaway word for the syllable tables, one rate lookup for the
    // frequency tables, which take about 70 ms to parse (as crates/triokey/src/hook.rs does)
    let lexicon = Lexicon::builtin();
    let mut warm = Engine::new(Config::default(), lexicon.clone());
    let _ = (warm.key('a'), warm.boundary(' '), lexicon.rates("a"));
    let source = tis::current();
    let mut seen = Vec::new();
    remember(&mut seen, &source);
    let engine = Engine::new(Config::default(), lexicon);
    APP.set(Some(App { engine, on: true, source, seen, secure: tis::secure_input(), failed: false, counters: Counters::default(), last_error: None }));
    ui::build(mtm)?;
    // the timer and the notification are registered in the self-test too; it never runs the loop, so they never fire
    ui::observe();
    refresh();
    if self_test {
        // CI: everything is built, nothing shown, no tap, nothing that can prompt
        print!("{}", diagnostics().render());
        return Ok(());
    }
    if !tap::create() {
        // once, so macOS lists TrioKey under Accessibility; the answer is ignored (it goes stale), the tap decides
        let key: &CFString = unsafe { kAXTrustedCheckOptionPrompt };
        let opts = CFDictionary::<CFString, CFBoolean>::from_slices(&[key], &[CFBoolean::new(true)]);
        // SAFETY: the dictionary holds the documented key and a CFBoolean
        let _ = unsafe { AXIsProcessTrustedWithOptions(Some(opts.as_opaque())) };
        ui::show_guide(mtm);
    }
    refresh();
    app.run();
    Ok(())
}

/// A second copy with the same bundle identifier exits at once.
fn another_instance() -> bool {
    let me = std::process::id() as i32;
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(BUNDLE_ID)).iter().any(|a| a.processIdentifier() != me)
}

pub fn state() -> State {
    let tap = tap::exists();
    with(|a| text::state(a.failed, tap, a.secure, a.on, a.source.decision()))
}

/// Re-reads the input source; on a change the word is dropped. Main thread only (TIS).
pub fn source_changed() {
    let s = tis::current();
    with(|a| {
        if s != a.source {
            a.engine.reset();
            remember(&mut a.seen, &s);
            a.source = s;
        }
    });
    refresh();
}

/// Every 2 seconds and when the menu opens: Secure Input, the tap while there is no permission, the input source (a
/// backstop in case the change notification does not arrive).
pub fn tick() {
    let secure = tis::secure_input();
    let failed = with(|a| {
        a.secure = secure;
        a.failed
    });
    if !failed && !tap::exists() {
        tap::create();
    }
    source_changed();
}

pub fn toggle() {
    with(|a| {
        a.on = !a.on;
        a.engine.reset();
    });
    refresh();
}

pub fn refresh() {
    let s = state();
    let on = with(|a| a.on);
    ui::show_state(s, on, tap::exists() && s != State::Error);
}

/// A panic or an internal error, in the tap or in a menu action: stop catching keys for good and keep what happened for
/// the diagnostics. Must not panic itself.
pub fn fail(error: String, panic: bool) {
    tap::disable();
    let _ = APP.try_with(|a| {
        if let Ok(mut a) = a.try_borrow_mut() {
            if let Some(a) = a.as_mut() {
                a.failed = true;
                a.counters.panics += panic as u64;
                a.last_error = Some(error);
            }
        }
    });
    let _ = std::panic::catch_unwind(refresh);
}

pub fn quit(mtm: MainThreadMarker) {
    // a tap left behind by a dying process must not swallow keys
    tap::disable();
    NSApplication::sharedApplication(mtm).terminate(None);
}

pub fn diagnostics() -> Diagnostics {
    let (tap_created, tap_enabled) = tap::status();
    let st = state();
    // system calls first, outside the borrow of the app state
    let os = NSProcessInfo::processInfo().operatingSystemVersionString().to_string();
    let bundle = NSBundle::mainBundle().bundlePath().to_string();
    // SAFETY: no arguments; never prompts
    let ax_trusted = unsafe { AXIsProcessTrusted() };
    with(|a| Diagnostics {
        version: VERSION.into(),
        os,
        arch: std::env::consts::ARCH.into(),
        bundle,
        home: std::env::var("HOME").ok(),
        tap_created,
        tap_enabled,
        ax_trusted,
        secure: a.secure,
        state: st,
        current: a.source.clone(),
        seen: a.seen.clone(),
        counters: a.counters,
        last_error: a.last_error.clone(),
    })
}
