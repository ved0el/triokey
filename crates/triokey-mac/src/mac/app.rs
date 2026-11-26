//! State shared by the tap, the menu and the settings window, start-up, the files and the 2-second timer. All of it lives on
//! the main thread; the only other thread is the version check's, which hands its answer back through a channel
//! (docs/ARCHITECTURE.md, 12.2 Everything on the main thread).

use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSRunningApplication};
use objc2_application_services::{kAXTrustedCheckOptionPrompt, AXIsProcessTrusted, AXIsProcessTrustedWithOptions};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};
use objc2_foundation::{NSBundle, NSProcessInfo, NSString};
use triokey_engine::lexicon::{Learned, Lexicon};
use triokey_engine::{Config, Engine, ToneStyle};
use triokey_mac::diag::{Counters, Diagnostics};
use triokey_mac::hotkey::HotState;
use triokey_mac::source::{remember, Decision, Source};
use triokey_mac::store::{self, Saver};
use triokey_mac::text::{self, State};
use triokey_mac::version::{self, Answer};
use triokey_mac::{login, BUNDLE_ID, VERSION};
use triokey_shared::config::{parse, render, Settings};

use super::{settings, tap, tis, ui};

/// Where TrioKey keeps its files: the data folder and the login item.
pub struct Files {
    pub dir: PathBuf,
    pub agent: PathBuf,
}

#[derive(Default)]
pub struct Update {
    /// The reserved line of the settings window; empty before the first check.
    pub status: String,
    pub waiting: Option<String>,
    /// The check in flight.
    rx: Option<Receiver<Answer>>,
    last_try: Option<Instant>,
}

pub struct App {
    pub engine: Engine,
    lexicon: Lexicon,
    pub on: bool,
    pub source: Source,
    pub seen: Vec<Source>,
    pub secure: bool,
    pub failed: bool,
    pub counters: Counters,
    pub last_error: Option<String>,
    pub settings: Settings,
    pub hot: HotState,
    pub files: Option<Files>,
    /// config.txt exists but could not be read: it is never written, and the settings window says so once.
    pub config_readable: bool,
    pub told_unreadable: bool,
    saver: Option<Saver>,
    /// force_vi.txt and force_en.txt entries, as read at start.
    force: (usize, usize),
    pub update: Update,
    started: Instant,
}

impl App {
    /// The input source decision, ignored when "Nhường khi bộ gõ Nhật/Trung/Hàn đang bật" is off.
    pub fn decision(&self) -> Decision {
        if self.settings.yield_cjk {
            self.source.decision()
        } else {
            Decision::Type
        }
    }
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

fn engine_config(s: &Settings) -> Config {
    Config { tone_style: if s.tone_new { ToneStyle::New } else { ToneStyle::Old }, auto_english: s.auto_english }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).filter(|h| h.is_absolute())
}

pub fn run(self_test: bool) -> Result<(), String> {
    panic_hook(self_test);
    let mtm = MainThreadMarker::new().ok_or("not started on the main thread")?;
    let app = NSApplication::sharedApplication(mtm);
    if another_instance() {
        return if self_test { Err("another TrioKey is running".into()) } else { Ok(()) };
    }
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    // the self-test never touches the real ~/Library: its files go to a temporary folder, removed at the end
    let root = if self_test { Some(std::env::temp_dir().join(format!("triokey-self-test-{}", std::process::id()))) } else { home() };
    let files = root.as_deref().map(store::paths).map(|(dir, agent)| Files { dir, agent });
    let (settings, readable) = files.as_ref().map_or((Settings::default(), true), |f| store::load_config(&f.dir));
    let (learned, saver) = files.as_ref().map_or((Learned::default(), None), |f| store::load_learned(&f.dir));
    let (vi, en) = files.as_ref().map_or_else(Default::default, |f| store::load_force(&f.dir));
    // build every table before the tap exists: a throwaway word for the syllable tables, one rate lookup for the
    // frequency tables, which take about 70 ms to parse (as crates/triokey/src/hook.rs does). The throwaway engine has
    // its own empty remembered words, so the warm-up word is never remembered.
    let mut warm = Engine::new(Config::default(), Lexicon::builtin());
    let _ = (warm.key('a'), warm.boundary(' '), Lexicon::builtin().rates("a"));
    let lexicon = Lexicon::builtin().with_user(&vi, &en).with_learned(learned);
    let source = tis::current();
    let mut seen = Vec::new();
    remember(&mut seen, &source);
    APP.set(Some(App {
        engine: Engine::new(engine_config(&settings), lexicon.clone()),
        lexicon,
        on: settings.enabled,
        source,
        seen,
        secure: tis::secure_input(),
        failed: false,
        counters: Counters::default(),
        last_error: None,
        settings,
        hot: HotState::default(),
        files,
        config_readable: readable,
        told_unreadable: false,
        saver,
        force: (store::entries(&vi), store::entries(&en)),
        update: Update::default(),
        started: Instant::now(),
    }));
    ui::build(mtm)?;
    // the timer and the notification are registered in the self-test too; it never runs the loop, so they never fire
    ui::observe();
    refresh();
    if self_test {
        // CI: everything is built, nothing shown, no tap, nothing that can prompt, no network, no pasteboard
        let result = self_test_checks();
        print!("{}", diagnostics().render());
        if let Some(r) = root {
            let _ = std::fs::remove_dir_all(r);
        }
        return result;
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

/// What the self-test checks beyond building: the settings window in both developer modes and every status (one height
/// each), the config format, the login item text and the version rule. Nothing here touches the network.
fn self_test_checks() -> Result<(), String> {
    settings::self_test()?;
    let s = Settings { tone_new: true, auto_english: false, dev_mode: true, last_update_check: 1_800_000_000, ..Settings::default() };
    if parse(&render(&s)) != s {
        return Err("config.txt does not round-trip".into());
    }
    let sample = "/Applications/TrioKey.app/Contents/MacOS/triokey-mac";
    let plist = login::plist(sample).ok_or("no login item for the sample path")?;
    if login::program(&plist).as_deref() != Some(sample) || plist.contains("KeepAlive") {
        return Err("the login item does not read back".into());
    }
    let tag = format!("{}/releases/tag/v", triokey_shared::REPOSITORY);
    let newer = version::answer(&format!("{tag}99.0.0"), VERSION) == Answer::Newer("99.0.0".into());
    let same = version::answer(&format!("{tag}{VERSION}"), VERSION) == Answer::Latest;
    if !newer || !same || version::answer("", VERSION) != Answer::Failed {
        return Err("the version rule is wrong".into());
    }
    println!("TrioKey self-test: settings window, config, login item and version rule OK");
    Ok(())
}

/// A second copy with the same bundle identifier exits at once.
fn another_instance() -> bool {
    let me = std::process::id() as i32;
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(BUNDLE_ID)).iter().any(|a| a.processIdentifier() != me)
}

pub fn state() -> State {
    let tap = tap::exists();
    with(|a| text::state(a.failed, tap, a.secure, a.on, a.decision()))
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

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Every 2 seconds and when the menu opens: Secure Input, the tap while there is no permission, learned.txt, the version
/// check, the input source (a backstop in case the change notification does not arrive).
pub fn tick() {
    let secure = tis::secure_input();
    let (failed, due) = with(|a| {
        a.secure = secure;
        if let Some(s) = a.saver.as_mut() {
            s.tick(false);
        }
        let u = &a.update;
        let since_try = u.last_try.map(|t| t.elapsed().as_secs());
        let due = u.rx.is_none() && version::due(a.settings.auto_update, a.started.elapsed().as_secs(), since_try, unix_now(), a.settings.last_update_check);
        (a.failed, due)
    });
    if !failed && !tap::exists() {
        tap::create();
    }
    update_answer();
    if due {
        check_now();
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
    settings::sync_live();
}

/// The settings window changed `f`: store, write config.txt (unless it was unreadable) and apply at once.
pub fn change(f: impl FnOnce(&mut Settings)) {
    let write = with(|a| {
        let old = a.settings;
        f(&mut a.settings);
        let s = a.settings;
        if (s.tone_new, s.auto_english) != (old.tone_new, old.auto_english) {
            a.engine = Engine::new(engine_config(&s), a.lexicon.clone());
        }
        if s.yield_cjk != old.yield_cjk {
            a.engine.reset();
        }
        a.files.as_ref().filter(|_| a.config_readable).map(|f| (f.dir.clone(), render(&s)))
    });
    // a failed write is ignored: the change still applies to this run
    if let Some((dir, text)) = write {
        let _ = store::write(&dir, store::CONFIG, &text);
    }
    refresh();
}

/// "Bật gõ tiếng Việt": switches TrioKey now and is the state at the next start. The menu and the hotkey switch only
/// this run, as on Windows.
pub fn set_enabled(on: bool) {
    with(|a| {
        a.on = on;
        a.engine.reset();
    });
    change(|s| s.enabled = on);
}

/// The running program, for the login item.
pub fn exe() -> Option<String> {
    let p = std::env::current_exe().ok()?;
    std::fs::canonicalize(&p).unwrap_or(p).to_str().map(str::to_string)
}

/// The login item's file, when there is one that can be read.
fn login_text() -> Option<String> {
    let agent = with(|a| a.files.as_ref().map(|f| f.agent.clone()))?;
    std::fs::read_to_string(agent).ok()
}

/// (ticked, usable) for "Khởi động cùng máy": ticked when the file exists and starts this program.
pub fn login() -> (bool, bool) {
    let Some(exe) = exe() else { return (false, false) };
    let usable = login::usable(&exe) && with(|a| a.files.is_some());
    (usable && login_text().and_then(|t| login::program(&t)).as_deref() == Some(exe.as_str()), usable)
}

/// Ticking writes the login item, unticking deletes it. Errors are ignored: the check box then shows the file as it is.
pub fn set_login(on: bool) {
    let (Some(exe), Some(agent)) = (exe(), with(|a| a.files.as_ref().map(|f| f.agent.clone()))) else { return };
    if !login::usable(&exe) {
        return;
    }
    if on {
        if let (Some(text), Some(dir)) = (login::plist(&exe), agent.parent()) {
            let _ = store::write(dir, login::FILE, &text);
        }
    } else {
        let _ = std::fs::remove_file(&agent);
    }
    change(|s| s.run_at_startup = on);
}

/// The data folder, created if needed, for "Mở thư mục cấu hình…".
pub fn config_dir() -> Option<PathBuf> {
    let dir = with(|a| a.files.as_ref().map(|f| f.dir.clone()))?;
    std::fs::create_dir_all(&dir).ok().map(|()| dir)
}

/// The version check on its own thread: /usr/bin/curl asks where the fixed releases address redirects to.
fn ask_github() -> Answer {
    let out = Command::new(version::CURL).args(version::curl_args()).arg(version::ADDRESS).stdin(Stdio::null()).stderr(Stdio::null()).output();
    match out {
        Ok(o) if o.status.success() => version::answer(&String::from_utf8_lossy(&o.stdout), VERSION),
        _ => Answer::Failed,
    }
}

/// "Kiểm tra bản mới", and the daily check. At most one runs.
pub fn check_now() {
    with(|a| {
        let u = &mut a.update;
        if u.rx.is_some() {
            return;
        }
        u.last_try = Some(Instant::now());
        let (tx, rx) = mpsc::channel();
        let spawned = std::thread::Builder::new().name("triokey-version-check".into()).spawn(move || {
            let _ = tx.send(ask_github());
        });
        if spawned.is_ok() {
            u.rx = Some(rx);
            u.status = text::UPDATE_CHECKING.into();
        } else {
            u.status = text::UPDATE_FAILED.into();
        }
    });
    refresh();
}

/// The longest a check may stay unanswered (curl's own limit is shorter).
const CHECK_LIMIT: Duration = Duration::from_secs(60);

/// The 2-second timer: takes the check's answer when there is one.
fn update_answer() {
    let write = with(|a| {
        let u = &mut a.update;
        let answer = match u.rx.as_ref()?.try_recv() {
            Ok(answer) => answer,
            Err(TryRecvError::Empty) if !u.last_try.is_some_and(|t| t.elapsed() > CHECK_LIMIT) => return None,
            // the thread ended without an answer, or took too long
            Err(_) => Answer::Failed,
        };
        u.rx = None;
        u.status = match &answer {
            Answer::Newer(v) => text::update_newer(v),
            Answer::Latest => text::UPDATE_LATEST.into(),
            Answer::Failed => text::UPDATE_FAILED.into(),
        };
        match answer {
            Answer::Newer(v) => u.waiting = Some(v),
            Answer::Latest => u.waiting = None,
            // keeps a version found earlier
            Answer::Failed => return Some(None),
        }
        a.settings.last_update_check = unix_now();
        Some(a.files.as_ref().filter(|_| a.config_readable).map(|f| (f.dir.clone(), render(&a.settings))))
    });
    let Some(write) = write else { return };
    if let Some((dir, text)) = write {
        let _ = store::write(&dir, store::CONFIG, &text);
    }
    refresh();
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
    let _ = std::panic::catch_unwind(|| {
        with(|a| {
            if let Some(s) = a.saver.as_mut() {
                s.tick(true);
            }
        })
    });
    NSApplication::sharedApplication(mtm).terminate(None);
}

pub fn diagnostics() -> Diagnostics {
    let (tap_created, tap_enabled) = tap::status();
    let st = state();
    // system calls and files first, outside the borrow of the app state
    let os = NSProcessInfo::processInfo().operatingSystemVersionString().to_string();
    let bundle = NSBundle::mainBundle().bundlePath().to_string();
    // SAFETY: no arguments; never prompts
    let ax_trusted = unsafe { AXIsProcessTrusted() };
    let login = login::state(login_text().as_deref(), &exe().unwrap_or_default()).to_string();
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
        config: render(&a.settings),
        config_dir: a.files.as_ref().map(|f| f.dir.display().to_string()),
        config_readable: a.config_readable,
        counts: (store::learned_count(a.lexicon.learned()), a.force.0, a.force.1),
        login,
        update: a.update.status.clone(),
        waiting: a.update.waiting.clone(),
    })
}
