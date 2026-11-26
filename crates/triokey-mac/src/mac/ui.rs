//! Menu bar item, its menu, the permission guide window, the timer and the input source notification. One Objective-C
//! class receives every action; the state it acts on lives in `app.rs`.

use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAlert, NSApplication, NSButton, NSColor, NSControlStateValueOff, NSControlStateValueOn, NSFont, NSLayoutAttribute, NSMenu, NSMenuDelegate, NSMenuItem,
    NSPasteboard, NSPasteboardTypeString, NSStackView, NSStatusBar, NSStatusItem, NSTextField, NSUserInterfaceLayoutOrientation, NSVariableStatusItemLength,
    NSView, NSWindow, NSWindowStyleMask, NSWorkspace,
};
use objc2_foundation::{
    ns_string, NSArray, NSDistributedNotificationCenter, NSEdgeInsets, NSNotification, NSNotificationSuspensionBehavior, NSObject, NSObjectProtocol, NSPoint,
    NSRect, NSSize, NSString, NSTimer, NSURL,
};
use triokey_mac::text::{self, State};

use super::{app, tis};

struct Ui {
    target: Retained<Target>,
    item: Retained<NSStatusItem>,
    toggle: Retained<NSMenuItem>,
    guide: Retained<NSWindow>,
    guide_status: Retained<NSTextField>,
}

thread_local! {
    static UI: RefCell<Option<Ui>> = const { RefCell::new(None) };
}

/// A panic must not unwind into AppKit: it is handled as in the tap.
fn guarded(f: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(f)).is_err() {
        app::panicked("in a menu action");
    }
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements; Target has no Drop
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "TrioKeyTarget"]
    struct Target;

    impl Target {
        #[unsafe(method(toggle:))]
        fn toggle(&self, _sender: Option<&AnyObject>) {
            guarded(app::toggle);
        }

        #[unsafe(method(showGuide:))]
        fn show_guide(&self, _sender: Option<&AnyObject>) {
            guarded(|| show_guide(self.mtm()));
        }

        #[unsafe(method(copyDiagnostics:))]
        fn copy_diagnostics(&self, _sender: Option<&AnyObject>) {
            guarded(|| copy_diagnostics(self.mtm()));
        }

        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: Option<&AnyObject>) {
            guarded(|| {
                if let Some(url) = NSURL::URLWithString(&NSString::from_str(text::SETTINGS_URL)) {
                    NSWorkspace::sharedWorkspace().openURL(&url);
                }
            });
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&AnyObject>) {
            guarded(|| app::quit(self.mtm()));
        }

        #[unsafe(method(tick:))]
        fn tick(&self, _timer: Option<&AnyObject>) {
            guarded(app::tick);
        }

        #[unsafe(method(sourceChanged:))]
        fn source_changed(&self, _note: Option<&NSNotification>) {
            // TIS off the main thread crashes; the 2-second timer covers a notification delivered anywhere else
            if MainThreadMarker::new().is_some() {
                guarded(app::source_changed);
            }
        }
    }

    unsafe impl NSObjectProtocol for Target {}

    unsafe impl NSMenuDelegate for Target {
        #[unsafe(method(menuWillOpen:))]
        fn menu_will_open(&self, _menu: &NSMenu) {
            guarded(app::tick);
        }
    }
);

impl Target {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: NSObject's init
        unsafe { msg_send![super(this), init] }
    }
}

fn item(mtm: MainThreadMarker, title: &str, action: Option<objc2::runtime::Sel>, target: &Target) -> Retained<NSMenuItem> {
    // SAFETY: every action is a method of Target, which is the item's target
    let i = unsafe { NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &NSString::from_str(title), action, ns_string!("")) };
    if action.is_some() {
        // SAFETY: Target outlives the menu: both are kept in UI until the process ends
        unsafe { i.setTarget(Some(target)) };
    }
    i
}

fn label(mtm: MainThreadMarker, s: &str) -> Retained<NSTextField> {
    let l = NSTextField::wrappingLabelWithString(&NSString::from_str(s), mtm);
    l.setPreferredMaxLayoutWidth(440.0);
    l.setSelectable(true);
    l
}

/// Status item, menu and guide window. Builds, shows nothing (the self-test calls this too).
pub fn build(mtm: MainThreadMarker) -> Result<(), String> {
    let target = Target::new(mtm);
    let item_ = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
    let menu = NSMenu::new(mtm);
    let toggle = item(mtm, text::MENU_TOGGLE, Some(sel!(toggle:)), &target);
    menu.addItem(&toggle);
    menu.addItem(&item(mtm, text::MENU_GUIDE, Some(sel!(showGuide:)), &target));
    menu.addItem(&item(mtm, text::MENU_DIAG, Some(sel!(copyDiagnostics:)), &target));
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    // no action: the menu shows it disabled
    menu.addItem(&item(mtm, &text::menu_version(), None, &target));
    menu.addItem(&item(mtm, text::MENU_QUIT, Some(sel!(quit:)), &target));
    menu.setDelegate(Some(ProtocolObject::from_ref(&*target)));
    item_.setMenu(Some(&menu));
    item_.button(mtm).ok_or("the status item has no button")?;

    let why = label(mtm, text::GUIDE_WHY);
    let steps = label(mtm, text::GUIDE_STEPS);
    // SAFETY: openSettings: is a method of Target
    let button = unsafe { NSButton::buttonWithTitle_target_action(&NSString::from_str(text::GUIDE_BUTTON), Some(&target), Some(sel!(openSettings:)), mtm) };
    let guide_status = label(mtm, text::guide_status(false));
    guide_status.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
    let relaunch = label(mtm, text::GUIDE_RELAUNCH);
    let warning = label(mtm, text::GUIDE_WARNING);
    // a system colour: readable in light and dark
    warning.setTextColor(Some(&NSColor::systemRedColor()));
    let views: [&NSView; 6] = [&why, &steps, &button, &guide_status, &relaunch, &warning];
    let stack = NSStackView::stackViewWithViews(&NSArray::from_slice(&views), mtm);
    stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
    stack.setAlignment(NSLayoutAttribute::Leading);
    stack.setSpacing(14.0);
    stack.setEdgeInsets(NSEdgeInsets { top: 20.0, left: 20.0, bottom: 20.0, right: 20.0 });
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(480.0, 420.0));
    // SAFETY: a plain titled window; released only through its Retained (setReleasedWhenClosed(false) below)
    let guide = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            rect,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            objc2_app_kit::NSBackingStoreType::Buffered,
            false,
        )
    };
    // SAFETY: closing must only hide it; the Retained keeps ownership
    unsafe { guide.setReleasedWhenClosed(false) };
    guide.setTitle(&NSString::from_str(text::GUIDE_TITLE));
    guide.setContentView(Some(&stack));
    guide.setContentSize(stack.fittingSize());
    guide.center();
    UI.set(Some(Ui { target, item: item_, toggle, guide, guide_status }));
    Ok(())
}

/// The 2-second timer and the input source notification. Registered in the self-test too, where neither fires: it never runs the loop.
pub fn observe() {
    UI.with_borrow(|u| {
        let Some(u) = u else { return };
        // SAFETY: tick: is a method of Target; the run loop retains the timer
        let _ = unsafe { NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(2.0, &u.target, sel!(tick:), None, true) };
        // SAFETY: a constant Carbon exports; CFString and NSString are toll-free bridged
        let name = unsafe { tis::kTISNotifySelectedKeyboardInputSourceChanged.cast::<NSString>().as_ref() };
        if let Some(name) = name {
            // an accessory app is never active: without DeliverImmediately the notification would wait for activation
            // SAFETY: sourceChanged: is a method of Target, which stays alive until the process ends
            unsafe {
                NSDistributedNotificationCenter::defaultCenter().addObserver_selector_name_object_suspensionBehavior(
                    &u.target,
                    sel!(sourceChanged:),
                    Some(name),
                    None,
                    NSNotificationSuspensionBehavior::DeliverImmediately,
                )
            };
        }
    });
}

pub fn show_state(s: State, on: bool, has_permission: bool) {
    UI.with_borrow(|u| {
        let Some(u) = u else { return };
        let mtm = u.target.mtm();
        if let Some(b) = u.item.button(mtm) {
            b.setTitle(&NSString::from_str(text::title(s)));
            b.setToolTip(Some(&NSString::from_str(text::tooltip(s))));
        }
        u.toggle.setState(if on { NSControlStateValueOn } else { NSControlStateValueOff });
        u.guide_status.setStringValue(&NSString::from_str(text::guide_status(has_permission)));
    });
}

fn activate(mtm: MainThreadMarker) {
    // `activate` exists only from macOS 14; this one still works on 11 to 26
    #[allow(deprecated)]
    NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
}

pub fn show_guide(mtm: MainThreadMarker) {
    let w = UI.with_borrow(|u| u.as_ref().map(|u| u.guide.clone()));
    if let Some(w) = w {
        activate(mtm);
        w.makeKeyAndOrderFront(None);
        // from macOS 14 activation is only a request the system may refuse; this brings the window forward anyway
        w.orderFrontRegardless();
    }
}

/// Works without permission and without a tap.
fn copy_diagnostics(mtm: MainThreadMarker) {
    let d = app::diagnostics().render();
    let pb = NSPasteboard::generalPasteboard();
    pb.clearContents();
    // SAFETY: a constant AppKit exports
    pb.setString_forType(&NSString::from_str(&d), unsafe { NSPasteboardTypeString });
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(text::DIAG_COPIED));
    alert.setInformativeText(&NSString::from_str(text::DIAG_COPIED_DETAIL));
    activate(mtm);
    // no borrow of the app state is held here: the tap may run while the alert spins the run loop
    alert.runModal();
}
