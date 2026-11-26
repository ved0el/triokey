//! The settings window (docs/ARCHITECTURE.md, 12.13 Settings, files and the window): the structure and texts of the
//! Windows window, built once with standard controls and system colours only, so it follows light and dark by itself.
//! Fixed size; only developer mode changes its height. Every action goes through `ui::Target`. A change of the interface
//! language builds it again in the other language, as wide as its longest row needs.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::{sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAlert, NSBox, NSBoxType, NSButton, NSColor, NSControlStateValueOff, NSControlStateValueOn, NSFont, NSLayoutAttribute, NSPopUpButton, NSStackView,
    NSStackViewGravity, NSTextField, NSUserInterfaceLayoutOrientation, NSView, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{NSArray, NSEdgeInsets, NSPoint, NSRect, NSSize, NSString};
use triokey_mac::text::{self, Text, HOTKEYS, LANGUAGES};
use triokey_mac::VERSION;
use triokey_shared::config::Settings;

use super::{app, ui};

struct Win {
    window: Retained<NSWindow>,
    stack: Retained<NSStackView>,
    enabled: Retained<NSButton>,
    hotkey: Retained<NSPopUpButton>,
    language: Retained<NSPopUpButton>,
    tone_old: Retained<NSButton>,
    tone_new: Retained<NSButton>,
    auto_en: Retained<NSButton>,
    yield_cjk: Retained<NSButton>,
    login: Retained<NSButton>,
    login_note: Retained<NSTextField>,
    dev: Retained<NSButton>,
    dev_row: Retained<NSStackView>,
    auto_check: Retained<NSButton>,
    status: Retained<NSTextField>,
    link: Retained<NSButton>,
    /// Width of every row: `WIDTH`, or more when a text in this language needs it.
    width: f64,
}

thread_local! {
    /// Borrowed shared, never across a modal alert or a window call that can run the loop. Replaced only by `rebuild`,
    /// from a settings action with no borrow held and nothing between take and set that runs the loop.
    static WIN: RefCell<Option<Win>> = const { RefCell::new(None) };
    /// The window before the last language change, kept until the next one: the pop-up that asked for the change is
    /// still in its action when the new window replaces it.
    static RETIRED: RefCell<Option<Win>> = const { RefCell::new(None) };
}

/// Least width of every row; the window is the rows' width plus the margins.
const WIDTH: f64 = 440.0;
/// Height of the reserved answer line of the version check, whatever it shows.
const STATUS_HEIGHT: f64 = 22.0;

fn ns(s: &str) -> Retained<NSString> {
    NSString::from_str(s)
}

fn on(b: bool) -> isize {
    if b {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    }
}

fn ticked(b: &NSButton) -> bool {
    b.state() == NSControlStateValueOn
}

fn check(mtm: MainThreadMarker, title: Text, target: &AnyObject, action: Sel) -> Retained<NSButton> {
    // SAFETY: every action is a method of ui::Target, which lives until the process ends
    unsafe { NSButton::checkboxWithTitle_target_action(&ns(title.get()), Some(target), Some(action), mtm) }
}

/// Each radio button has its own action, which sets both: the pair does not rely on AppKit's automatic grouping.
fn radio(mtm: MainThreadMarker, title: Text, target: &AnyObject, action: Sel) -> Retained<NSButton> {
    // SAFETY: as in `check`
    unsafe { NSButton::radioButtonWithTitle_target_action(&ns(title.get()), Some(target), Some(action), mtm) }
}

fn button(mtm: MainThreadMarker, title: Text, target: &AnyObject, action: Sel) -> Retained<NSButton> {
    // SAFETY: as in `check`
    unsafe { NSButton::buttonWithTitle_target_action(&ns(title.get()), Some(target), Some(action), mtm) }
}

/// A pop-up button (not a pull-down) of `items` that reports to `settingChanged:`.
fn popup(mtm: MainThreadMarker, items: &[&str], target: &AnyObject) -> Retained<NSPopUpButton> {
    let p = NSPopUpButton::initWithFrame_pullsDown(NSPopUpButton::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)), false);
    for name in items {
        p.addItemWithTitle(&ns(name));
    }
    // SAFETY: settingChanged: is a method of ui::Target
    unsafe {
        p.setTarget(Some(target));
        p.setAction(Some(sel!(settingChanged:)));
    }
    p
}

/// A borderless button in the system link colour.
fn link(mtm: MainThreadMarker, title: Text, target: &AnyObject, action: Sel) -> Retained<NSButton> {
    let b = button(mtm, title, target, action);
    b.setBordered(false);
    b.setContentTintColor(Some(&NSColor::linkColor()));
    b
}

fn label(mtm: MainThreadMarker, s: &str) -> Retained<NSTextField> {
    NSTextField::labelWithString(&ns(s), mtm)
}

fn secondary(mtm: MainThreadMarker, s: &str) -> Retained<NSTextField> {
    let l = label(mtm, s);
    l.setTextColor(Some(&NSColor::secondaryLabelColor()));
    l
}

fn heading(mtm: MainThreadMarker, s: Text) -> Retained<NSTextField> {
    let l = label(mtm, s.get());
    l.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
    l
}

/// One horizontal row: `lead` from the left, `trail` against the right edge once `build` gives it the full width.
fn row(mtm: MainThreadMarker, lead: &[&NSView], trail: &[&NSView]) -> Retained<NSStackView> {
    let r = NSStackView::new(mtm);
    r.setOrientation(NSUserInterfaceLayoutOrientation::Horizontal);
    r.setSpacing(8.0);
    for v in lead {
        r.addView_inGravity(v, NSStackViewGravity::Leading);
    }
    for v in trail {
        r.addView_inGravity(v, NSStackViewGravity::Trailing);
    }
    r
}

/// Builds the window, shows nothing (the self-test calls this too).
pub fn build(mtm: MainThreadMarker, target: &AnyObject) {
    let changed = sel!(settingChanged:);
    let enabled = check(mtm, text::SET_ENABLED, target, changed);
    let hotkey = popup(mtm, &HOTKEYS.map(|(_, name)| name.get()), target);
    let hotkey_label = label(mtm, text::SET_HOTKEY.get());
    let top = row(mtm, &[&enabled], &[&hotkey_label, &hotkey]);

    let tone_label = label(mtm, text::SET_TONE.get());
    let (tone_old, tone_new) = (radio(mtm, text::SET_TONE_OLD, target, sel!(toneOld:)), radio(mtm, text::SET_TONE_NEW, target, sel!(toneNew:)));
    let tone = row(mtm, &[&tone_label, &tone_old, &tone_new], &[]);
    let auto_en = check(mtm, text::SET_AUTO_EN, target, changed);
    let yield_cjk = check(mtm, text::SET_YIELD, target, changed);

    let login = check(mtm, text::SET_LOGIN, target, sel!(loginChanged:));
    let login_note = secondary(mtm, text::SET_LOGIN_ONLY_APPS.get());
    login_note.setFont(Some(&NSFont::systemFontOfSize(NSFont::smallSystemFontSize())));
    let login_row = row(mtm, &[&login, &login_note], &[]);
    let language_label = label(mtm, text::SET_LANGUAGE.get());
    let language = popup(mtm, &LANGUAGES, target);
    let language_row = row(mtm, &[&language_label, &language], &[]);
    let dev = check(mtm, text::SET_DEV, target, changed);
    let folder = button(mtm, text::SET_FOLDER, target, sel!(openFolder:));
    let debug = button(mtm, text::SET_DEBUG, target, sel!(copyDebug:));
    let dev_row = row(mtm, &[&folder, &debug], &[]);
    // under the check box's label, as on Windows
    dev_row.setEdgeInsets(NSEdgeInsets { top: 0.0, left: 18.0, bottom: 0.0, right: 0.0 });
    // one arranged view whether the buttons show or not, so the gap after it stays the same
    let dev_box = NSStackView::stackViewWithViews(&NSArray::from_slice(&[&*dev as &NSView, &dev_row]), mtm);
    dev_box.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
    dev_box.setAlignment(NSLayoutAttribute::Leading);
    dev_box.setSpacing(8.0);
    dev_box.setDetachesHiddenViews(true);

    let auto_check = check(mtm, text::SET_AUTO_CHECK, target, changed);
    let check_now = button(mtm, text::SET_CHECK, target, sel!(checkNow:));
    let updates = row(mtm, &[&auto_check], &[&check_now]);
    let status = label(mtm, "");
    let download = link(mtm, text::SET_LINK, target, sel!(openDownload:));
    let status_row = row(mtm, &[&status, &download], &[]);
    // the link keeps its place while hidden, and the row its height whatever the answer: the window never changes size
    status_row.setDetachesHiddenViews(false);
    status_row.heightAnchor().constraintEqualToConstant(STATUS_HEIGHT).setActive(true);

    let line = NSBox::new(mtm);
    line.setBoxType(NSBoxType::Separator);
    let credit = secondary(mtm, &triokey_shared::footer(VERSION));
    let github = link(mtm, text::SET_GITHUB, target, sel!(openRepository:));
    let footer = row(mtm, &[&credit], &[&github]);

    let (h_typing, h_system, h_updates) = (heading(mtm, text::SET_TYPING), heading(mtm, text::SET_SYSTEM), heading(mtm, text::SET_UPDATES));
    let views: [&NSView; 14] =
        [&top, &h_typing, &tone, &auto_en, &yield_cjk, &h_system, &login_row, &language_row, &dev_box, &h_updates, &updates, &status_row, &line, &footer];
    // the rows' width: WIDTH, or the widest view's natural width when a text needs more (English), so the window grows
    let width = views.iter().map(|v| v.fittingSize().width.ceil()).fold(WIDTH, f64::max);
    for v in [&*top as &NSView, &tone, &login_row, &language_row, &updates, &status_row, &line, &footer] {
        v.widthAnchor().constraintEqualToConstant(width).setActive(true);
    }
    let stack = NSStackView::stackViewWithViews(&NSArray::from_slice(&views), mtm);
    stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
    stack.setAlignment(NSLayoutAttribute::Leading);
    stack.setSpacing(8.0);
    for v in [&*h_typing, &*h_system, &*h_updates] {
        stack.setCustomSpacing_afterView(6.0, v);
    }
    for v in [&*yield_cjk as &NSView, &dev_box, &status_row] {
        stack.setCustomSpacing_afterView(16.0, v);
    }
    stack.setEdgeInsets(NSEdgeInsets { top: 20.0, left: 20.0, bottom: 16.0, right: 20.0 });
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width + 40.0, 400.0));
    // SAFETY: a plain titled window that cannot be resized; released only through its Retained
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            rect,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            objc2_app_kit::NSBackingStoreType::Buffered,
            false,
        )
    };
    // SAFETY: closing must only hide it; the Retained keeps ownership
    unsafe { window.setReleasedWhenClosed(false) };
    window.setTitle(&ns(text::SETTINGS_TITLE.get()));
    window.setContentView(Some(&stack));
    WIN.set(Some(Win {
        window,
        stack,
        enabled,
        hotkey,
        language,
        tone_old,
        tone_new,
        auto_en,
        yield_cjk,
        login,
        login_note,
        dev,
        dev_row,
        auto_check,
        status,
        link: download,
        width,
    }));
    sync_all();
    WIN.with_borrow(|w| w.iter().for_each(|w| w.window.center()));
}

/// The interface language changed: the window is built again in it, where the old one was and shown if it was.
pub fn rebuild(mtm: MainThreadMarker, target: &AnyObject) {
    let Some(old) = WIN.take() else { return };
    build(mtm, target);
    let f = old.window.frame();
    WIN.with_borrow(|w| {
        if let Some(w) = w {
            w.window.setFrameTopLeftPoint(NSPoint::new(f.origin.x, f.origin.y + f.size.height));
            if old.window.isVisible() {
                w.window.makeKeyAndOrderFront(None);
            }
        }
    });
    old.window.orderOut(None);
    RETIRED.set(Some(old));
}

/// The window's content height fits its rows; the top edge stays where it was.
fn fit(w: &Win) {
    let old = w.window.frame();
    w.window.setContentSize(w.stack.fittingSize());
    let new = w.window.frame();
    w.window.setFrameOrigin(NSPoint::new(new.origin.x, old.origin.y + old.size.height - new.size.height));
}

/// Every control from the settings, the login item and the live state.
fn sync_all() {
    let (login, usable) = app::login();
    let s = app::with(|a| a.settings);
    WIN.with_borrow(|w| {
        let Some(w) = w else { return };
        w.hotkey.selectItemAtIndex(text::hotkey_index(s.hotkey) as isize);
        w.language.selectItemAtIndex(s.english as isize);
        w.tone_old.setState(on(!s.tone_new));
        w.tone_new.setState(on(s.tone_new));
        w.auto_en.setState(on(s.auto_english));
        w.yield_cjk.setState(on(s.yield_cjk));
        w.login.setState(on(login));
        w.login.setEnabled(usable);
        w.login_note.setHidden(usable);
        w.login.setToolTip((!usable).then(|| ns(text::SET_LOGIN_ONLY_APPS.get())).as_deref());
        w.dev.setState(on(s.dev_mode));
        w.auto_check.setState(on(s.auto_update));
        w.dev_row.setHidden(!s.dev_mode);
        fit(w);
    });
    sync_live();
}

/// What changes without the window: the switch (menu, hotkey) and the version check. Cheap; runs on every refresh.
pub fn sync_live() {
    let (enabled, status, waiting) = app::with(|a| (a.on, a.update.status(triokey_shared::english()), a.update.waiting.clone()));
    WIN.with_borrow(|w| {
        let Some(w) = w else { return };
        w.enabled.setState(on(enabled));
        w.status.setStringValue(&ns(&status));
        w.link.setHidden(waiting.is_none());
    });
    ui::show_waiting(waiting.as_deref());
}

/// Any check box, radio button or the pop-up except the login item.
pub fn changed() {
    struct V {
        enabled: bool,
        hotkey: usize,
        english: bool,
        tone_new: bool,
        auto_en: bool,
        yield_cjk: bool,
        dev: bool,
        auto_check: bool,
    }
    let v = WIN.with_borrow(|w| {
        w.as_ref().map(|w| V {
            enabled: ticked(&w.enabled),
            hotkey: usize::try_from(w.hotkey.indexOfSelectedItem()).unwrap_or(0).min(HOTKEYS.len() - 1),
            english: w.language.indexOfSelectedItem() == 1,
            tone_new: ticked(&w.tone_new),
            auto_en: ticked(&w.auto_en),
            yield_cjk: ticked(&w.yield_cjk),
            dev: ticked(&w.dev),
            auto_check: ticked(&w.auto_check),
        })
    });
    let Some(v) = v else { return };
    if v.enabled != app::with(|a| a.on) {
        app::set_enabled(v.enabled);
    }
    let dev_was = app::with(|a| a.settings.dev_mode);
    // a new language rebuilds this window (`app::change`, `rebuild`)
    app::change(|s| {
        s.hotkey = HOTKEYS[v.hotkey].0;
        s.english = v.english;
        (s.tone_new, s.auto_english, s.yield_cjk, s.dev_mode, s.auto_update) = (v.tone_new, v.auto_en, v.yield_cjk, v.dev, v.auto_check);
    });
    if v.dev != dev_was {
        WIN.with_borrow(|w| {
            if let Some(w) = w {
                w.dev_row.setHidden(!v.dev);
                fit(w);
            }
        });
    }
}

/// A tone radio button: `new` is `hoà, thuý`.
pub fn tone(new: bool) {
    WIN.with_borrow(|w| {
        if let Some(w) = w {
            w.tone_old.setState(on(!new));
            w.tone_new.setState(on(new));
        }
    });
    changed();
}

pub fn login_changed() {
    let Some(want) = WIN.with_borrow(|w| w.as_ref().map(|w| ticked(&w.login))) else { return };
    app::set_login(want);
    sync_all();
}

pub fn show(mtm: MainThreadMarker) {
    sync_all();
    let Some(w) = WIN.with_borrow(|w| w.as_ref().map(|w| w.window.clone())) else { return };
    ui::activate(mtm);
    w.makeKeyAndOrderFront(None);
    w.orderFrontRegardless();
    let tell = app::with(|a| !a.config_readable && !std::mem::replace(&mut a.told_unreadable, true));
    if tell {
        let alert = NSAlert::new(mtm);
        alert.setMessageText(&ns(text::CONFIG_UNREADABLE.get()));
        // no borrow is held here: the tap may run while the alert spins the run loop
        alert.runModal();
    }
}

/// The self-test: developer mode off and on through the real path, and every answer of the version check in the
/// reserved line: one height per developer mode, the taller one with it on.
pub fn self_test() -> Result<(), String> {
    let mut heights = Vec::new();
    for dev in [false, true] {
        app::with(|a| a.settings.dev_mode = dev);
        sync_all();
        let newer = text::update_newer("9.9.9", false);
        let [checking, latest, failed] = [text::UPDATE_CHECKING, text::UPDATE_LATEST, text::UPDATE_FAILED].map(Text::get);
        for (status, waiting) in [("", false), (checking, false), (latest, false), (&*newer, true), (failed, true)] {
            let h = WIN.with_borrow(|w| {
                let w = w.as_ref()?;
                w.status.setStringValue(&ns(status));
                w.link.setHidden(!waiting);
                Some(w.stack.fittingSize().height)
            });
            heights.push((dev, h.ok_or("the settings window was not built")?));
        }
    }
    app::with(|a| a.settings.dev_mode = false);
    sync_all();
    // the handlers, as a click runs them (files go to the self-test's folder; no alert, network or pasteboard): the
    // controls must read back what sync_all wrote, and the tone buttons must switch
    let before = app::with(|a| a.settings);
    changed();
    tone(!before.tone_new);
    let switched = app::with(|a| a.settings.tone_new) != before.tone_new;
    tone(before.tone_new);
    if !switched {
        return Err("the tone buttons do not switch".into());
    }
    login_changed();
    // run from /Applications, the login item is usable and unticked in the empty folder: run_at_startup follows it
    let after = Settings { run_at_startup: before.run_at_startup, ..app::with(|a| a.settings) };
    if after != before {
        return Err(format!("the settings window does not read back: {before:?} became {after:?}"));
    }
    let was = app::with(|a| a.update.waiting.replace("9.9.9".into()));
    sync_live();
    app::with(|a| a.update.waiting = was);
    sync_live();
    language()?;
    let one = |dev: bool| {
        let h: Vec<f64> = heights.iter().filter(|(d, _)| *d == dev).map(|(_, h)| *h).collect();
        h.iter().all(|x| x == &h[0]).then_some(h[0])
    };
    match (one(false), one(true)) {
        (Some(off), Some(on)) if on > off => {
            println!("TrioKey self-test: settings window height {off} (developer mode {on})");
            Ok(())
        }
        _ => Err(format!("the settings window changes height: {heights:?}")),
    }
}

/// The self-test's language switch, to English and back through the real path (`changed`, `app::change`, `rebuild`):
/// the menu and this window follow, and nothing sticks out of the rows (the window is as wide as they are).
fn language() -> Result<(), String> {
    for en in [true, false] {
        WIN.with_borrow(|w| w.iter().for_each(|w| w.language.selectItemAtIndex(en as isize)));
        changed();
        let got = WIN.with_borrow(|w| w.as_ref().map(|w| (w.window.title().to_string(), w.stack.fittingSize().width, w.width)));
        let Some((title, fit, width)) = got else { return Err("the settings window was not rebuilt".into()) };
        let quit = ui::quit_title();
        let set = app::with(|a| a.settings.english);
        if triokey_shared::english() != en || set != en || title != text::SETTINGS_TITLE.pick(en) || quit.as_deref() != Some(text::MENU_QUIT.pick(en)) {
            return Err(format!("the language switch to english={en} shows {title:?} and {quit:?} (setting {set})"));
        }
        if (fit - (width + 40.0)).abs() > 0.5 {
            return Err(format!("the settings window (english={en}) is {fit} wide for rows of {width}"));
        }
        println!("TrioKey self-test: language english={en}, settings window rows {width}");
    }
    Ok(())
}
