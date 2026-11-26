//! The settings window (docs/ARCHITECTURE.md, 12.13 Settings, files and the window): the structure and texts of the
//! Windows window, built once with standard controls and system colours only, so it follows light and dark by itself.
//! Fixed size; only developer mode changes its height. Every action goes through `ui::Target`.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::{sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAlert, NSBox, NSBoxType, NSButton, NSColor, NSControlStateValueOff, NSControlStateValueOn, NSFont, NSLayoutAttribute, NSPopUpButton, NSStackView,
    NSStackViewGravity, NSTextField, NSUserInterfaceLayoutOrientation, NSView, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{NSArray, NSEdgeInsets, NSPoint, NSRect, NSSize, NSString};
use triokey_mac::text::{self, HOTKEYS};
use triokey_mac::VERSION;
use triokey_shared::config::Settings;

use super::{app, ui};

struct Win {
    window: Retained<NSWindow>,
    stack: Retained<NSStackView>,
    enabled: Retained<NSButton>,
    hotkey: Retained<NSPopUpButton>,
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
}

thread_local! {
    /// Only ever borrowed shared, and never across a modal alert or a window call that can run the loop.
    static WIN: RefCell<Option<Win>> = const { RefCell::new(None) };
}

/// Width of every row; the window is this plus the margins.
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

fn check(mtm: MainThreadMarker, title: &str, target: &AnyObject, action: Sel) -> Retained<NSButton> {
    // SAFETY: every action is a method of ui::Target, which lives until the process ends
    unsafe { NSButton::checkboxWithTitle_target_action(&ns(title), Some(target), Some(action), mtm) }
}

/// Each radio button has its own action, which sets both: the pair does not rely on AppKit's automatic grouping.
fn radio(mtm: MainThreadMarker, title: &str, target: &AnyObject, action: Sel) -> Retained<NSButton> {
    // SAFETY: as in `check`
    unsafe { NSButton::radioButtonWithTitle_target_action(&ns(title), Some(target), Some(action), mtm) }
}

fn button(mtm: MainThreadMarker, title: &str, target: &AnyObject, action: Sel) -> Retained<NSButton> {
    // SAFETY: as in `check`
    unsafe { NSButton::buttonWithTitle_target_action(&ns(title), Some(target), Some(action), mtm) }
}

/// A borderless button in the system link colour.
fn link(mtm: MainThreadMarker, title: &str, target: &AnyObject, action: Sel) -> Retained<NSButton> {
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

fn heading(mtm: MainThreadMarker, s: &str) -> Retained<NSTextField> {
    let l = label(mtm, s);
    l.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
    l
}

/// One horizontal row of the full width: `lead` from the left, `trail` against the right edge.
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
    r.widthAnchor().constraintEqualToConstant(WIDTH).setActive(true);
    r
}

/// Builds the window, shows nothing (the self-test calls this too).
pub fn build(mtm: MainThreadMarker, target: &AnyObject) {
    let changed = sel!(settingChanged:);
    let enabled = check(mtm, text::SET_ENABLED, target, changed);
    // a pop-up button, not a pull-down
    let hotkey = NSPopUpButton::initWithFrame_pullsDown(NSPopUpButton::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)), false);
    for (_, name) in HOTKEYS {
        hotkey.addItemWithTitle(&ns(name));
    }
    // SAFETY: settingChanged: is a method of ui::Target
    unsafe {
        hotkey.setTarget(Some(target));
        hotkey.setAction(Some(changed));
    }
    let hotkey_label = label(mtm, text::SET_HOTKEY);
    let top = row(mtm, &[&enabled], &[&hotkey_label, &hotkey]);

    let tone_label = label(mtm, text::SET_TONE);
    let (tone_old, tone_new) = (radio(mtm, text::SET_TONE_OLD, target, sel!(toneOld:)), radio(mtm, text::SET_TONE_NEW, target, sel!(toneNew:)));
    let tone = row(mtm, &[&tone_label, &tone_old, &tone_new], &[]);
    let auto_en = check(mtm, text::SET_AUTO_EN, target, changed);
    let yield_cjk = check(mtm, text::SET_YIELD, target, changed);

    let login = check(mtm, text::SET_LOGIN, target, sel!(loginChanged:));
    let login_note = secondary(mtm, text::SET_LOGIN_ONLY_APPS);
    login_note.setFont(Some(&NSFont::systemFontOfSize(NSFont::smallSystemFontSize())));
    let login_row = row(mtm, &[&login, &login_note], &[]);
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
    line.widthAnchor().constraintEqualToConstant(WIDTH).setActive(true);
    let credit = secondary(mtm, &triokey_shared::credit(VERSION));
    let github = link(mtm, text::SET_GITHUB, target, sel!(openRepository:));
    let footer = row(mtm, &[&credit], &[&github]);

    let (h_typing, h_system, h_updates) = (heading(mtm, text::SET_TYPING), heading(mtm, text::SET_SYSTEM), heading(mtm, text::SET_UPDATES));
    let views: [&NSView; 13] =
        [&top, &h_typing, &tone, &auto_en, &yield_cjk, &h_system, &login_row, &dev_box, &h_updates, &updates, &status_row, &line, &footer];
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
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(WIDTH + 40.0, 400.0));
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
    window.setTitle(&ns(text::SETTINGS_TITLE));
    window.setContentView(Some(&stack));
    WIN.set(Some(Win {
        window,
        stack,
        enabled,
        hotkey,
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
    }));
    sync_all();
    WIN.with_borrow(|w| w.iter().for_each(|w| w.window.center()));
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
        w.tone_old.setState(on(!s.tone_new));
        w.tone_new.setState(on(s.tone_new));
        w.auto_en.setState(on(s.auto_english));
        w.yield_cjk.setState(on(s.yield_cjk));
        w.login.setState(on(login));
        w.login.setEnabled(usable);
        w.login_note.setHidden(usable);
        w.login.setToolTip((!usable).then(|| ns(text::SET_LOGIN_ONLY_APPS)).as_deref());
        w.dev.setState(on(s.dev_mode));
        w.auto_check.setState(on(s.auto_update));
        w.dev_row.setHidden(!s.dev_mode);
        fit(w);
    });
    sync_live();
}

/// What changes without the window: the switch (menu, hotkey) and the version check. Cheap; runs on every refresh.
pub fn sync_live() {
    let (enabled, status, waiting) = app::with(|a| (a.on, a.update.status.clone(), a.update.waiting.clone()));
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
    app::change(|s| {
        // a stored ctrl_space shows as "Không dùng" and stays as it is until another hotkey is picked
        if text::hotkey_index(s.hotkey) != v.hotkey {
            s.hotkey = HOTKEYS[v.hotkey].0;
        }
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
        alert.setMessageText(&ns(text::CONFIG_UNREADABLE));
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
        let newer = text::update_newer("9.9.9");
        for (status, waiting) in [("", false), (text::UPDATE_CHECKING, false), (text::UPDATE_LATEST, false), (&*newer, true), (text::UPDATE_FAILED, true)] {
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
