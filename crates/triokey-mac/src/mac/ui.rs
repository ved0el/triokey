//! Menu bar item, its menu, the permission guide window, the timer and the input source notification. One Objective-C
//! class receives every action, the settings window's too (`settings.rs`); the state it acts on lives in `app.rs`.

use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::null_mut;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, AnyThread, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAlert, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication, NSBitmapImageRep, NSButton, NSColor,
    NSControlStateValueOff, NSControlStateValueOn, NSDeviceRGBColorSpace, NSFont, NSImage, NSLayoutAttribute, NSMenu, NSMenuDelegate, NSMenuItem, NSPasteboard,
    NSPasteboardTypeString, NSStackView, NSStatusBar, NSStatusBarButton, NSStatusItem, NSTextField, NSUserInterfaceLayoutOrientation,
    NSVariableStatusItemLength, NSView, NSWindow, NSWindowStyleMask, NSWorkspace,
};
use objc2_foundation::{
    ns_string, NSArray, NSDistributedNotificationCenter, NSEdgeInsets, NSNotification, NSNotificationSuspensionBehavior, NSObject, NSObjectProtocol, NSPoint,
    NSRect, NSSize, NSString, NSTimer, NSURL,
};
use triokey_mac::text::{self, State, Text};
use triokey_shared::glyph::Glyph;

use super::{app, settings, tis};

struct Ui {
    target: Retained<Target>,
    item: Retained<NSStatusItem>,
    toggle: Retained<NSMenuItem>,
    /// Every menu item with a fixed text, retitled when the language changes; Quit last.
    titled: Vec<(Retained<NSMenuItem>, Text)>,
    /// "Có bản … — mở trang tải về", hidden while no newer version waits.
    waiting: Retained<NSMenuItem>,
    guide: Retained<NSWindow>,
    /// The guide's fixed texts, retitled when the language changes.
    guide_texts: Vec<(Retained<NSTextField>, Text)>,
    guide_button: Retained<NSButton>,
    guide_status: Retained<NSTextField>,
    /// What the menu bar image shows (glyph, dark bar); `None` while there is no image. Rebuilt only on a change: this
    /// runs inside the tap callback when the hotkey toggles.
    icon: Cell<Option<(Glyph, bool)>>,
}

/// SF Symbols of the menu items (macOS 11): Settings, guide, new version, Restart, Quit.
const SYMBOLS: [&str; 5] = ["gearshape", "lock.shield", "arrow.down.circle", "arrow.clockwise", "power"];

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

        #[unsafe(method(showSettings:))]
        fn show_settings(&self, _sender: Option<&AnyObject>) {
            guarded(|| settings::show(self.mtm()));
        }

        #[unsafe(method(settingChanged:))]
        fn setting_changed(&self, _sender: Option<&AnyObject>) {
            guarded(settings::changed);
        }

        #[unsafe(method(toneOld:))]
        fn tone_old(&self, _sender: Option<&AnyObject>) {
            guarded(|| settings::tone(false));
        }

        #[unsafe(method(toneNew:))]
        fn tone_new(&self, _sender: Option<&AnyObject>) {
            guarded(|| settings::tone(true));
        }

        #[unsafe(method(loginChanged:))]
        fn login_changed(&self, _sender: Option<&AnyObject>) {
            guarded(settings::login_changed);
        }

        #[unsafe(method(openFolder:))]
        fn open_folder(&self, _sender: Option<&AnyObject>) {
            guarded(|| {
                if let Some(dir) = app::config_dir().and_then(|d| d.to_str().map(str::to_string)) {
                    NSWorkspace::sharedWorkspace().openURL(&NSURL::fileURLWithPath(&NSString::from_str(&dir)));
                }
            });
        }

        #[unsafe(method(copyDebug:))]
        fn copy_debug(&self, _sender: Option<&AnyObject>) {
            guarded(|| copy_debug(self.mtm()));
        }

        #[unsafe(method(checkNow:))]
        fn check_now(&self, _sender: Option<&AnyObject>) {
            guarded(app::check_now);
        }

        #[unsafe(method(openDownload:))]
        fn open_download(&self, _sender: Option<&AnyObject>) {
            guarded(|| open_url(triokey_shared::RELEASES));
        }

        #[unsafe(method(openRepository:))]
        fn open_repository(&self, _sender: Option<&AnyObject>) {
            guarded(|| open_url(triokey_shared::REPOSITORY));
        }

        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: Option<&AnyObject>) {
            guarded(|| open_url(text::SETTINGS_URL));
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&AnyObject>) {
            guarded(|| app::quit(self.mtm()));
        }

        #[unsafe(method(restart:))]
        fn restart(&self, _sender: Option<&AnyObject>) {
            guarded(|| app::restart(self.mtm()));
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

fn ns(s: &str) -> Retained<NSString> {
    NSString::from_str(s)
}

fn item(mtm: MainThreadMarker, title: &str, action: objc2::runtime::Sel, target: &Target) -> Retained<NSMenuItem> {
    // SAFETY: every action is a method of Target, which is the item's target
    let i = unsafe { NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &ns(title), Some(action), ns_string!("")) };
    // SAFETY: Target outlives the menu: both are kept in UI until the process ends
    unsafe { i.setTarget(Some(target)) };
    i
}

/// An SF Symbol, `None` when this macOS lacks it (then the item has no image).
fn symbol(name: &str) -> Option<Retained<NSImage>> {
    NSImage::imageWithSystemSymbolName_accessibilityDescription(&ns(name), None)
}

/// The menu bar image: 18×18 pt with bitmaps of 18 and 36 px, in colour (not a template). `dark`: a dark menu bar, which
/// takes the light shades. `None` when AppKit refuses a bitmap; the caller then shows the letter.
fn glyph_image(g: Glyph, dark: bool) -> Option<Retained<NSImage>> {
    let pt = NSSize::new(18.0, 18.0);
    let image = NSImage::initWithSize(NSImage::alloc(), pt);
    for n in [18usize, 36] {
        let px = text::icon(g, n as u32, !dark);
        // SAFETY: null planes: the rep allocates its own buffer for this layout (8-bit RGBA, interleaved); the colour space
        // name is a constant AppKit exports
        let rep = unsafe {
            NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
                NSBitmapImageRep::alloc(),
                null_mut(),
                n as isize,
                n as isize,
                8,
                4,
                true,
                false,
                NSDeviceRGBColorSpace,
                (n * 4) as isize,
                32,
            )
        }?;
        let (data, row) = (rep.bitmapData(), usize::try_from(rep.bytesPerRow()).unwrap_or(0));
        if data.is_null() || row < n * 4 {
            return None;
        }
        for (y, line) in px.chunks_exact(n * 4).enumerate() {
            // SAFETY: the rep's buffer holds `n` rows of `row` >= n * 4 bytes
            unsafe { std::ptr::copy_nonoverlapping(line.as_ptr(), data.add(y * row), line.len()) };
        }
        rep.setSize(pt);
        image.addRepresentation(&rep);
    }
    image.setTemplate(false);
    Some(image)
}

/// Whether the menu bar behind `b` is dark: the best match among light and dark, so a vibrant variant counts too.
fn dark_bar(b: &NSStatusBarButton) -> bool {
    // SAFETY: constants AppKit exports
    let (aqua, dark) = unsafe { (NSAppearanceNameAqua, NSAppearanceNameDarkAqua) };
    b.effectiveAppearance().bestMatchFromAppearancesWithNames(&NSArray::from_slice(&[aqua, dark])).is_some_and(|n| *n == *dark)
}

fn label(mtm: MainThreadMarker, s: Text) -> Retained<NSTextField> {
    let l = NSTextField::wrappingLabelWithString(&ns(s.get()), mtm);
    l.setPreferredMaxLayoutWidth(440.0);
    l.setSelectable(true);
    l
}

/// Status item, menu and guide window. Builds, shows nothing (the self-test calls this too).
pub fn build(mtm: MainThreadMarker) -> Result<(), String> {
    let target = Target::new(mtm);
    let item_ = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
    let menu = NSMenu::new(mtm);
    let titled: Vec<(Retained<NSMenuItem>, Text)> = [
        (text::MENU_TOGGLE, sel!(toggle:), None),
        (text::MENU_SETTINGS, sel!(showSettings:), Some(SYMBOLS[0])),
        (text::MENU_GUIDE, sel!(showGuide:), Some(SYMBOLS[1])),
        (text::MENU_RESTART, sel!(restart:), Some(SYMBOLS[3])),
        (text::MENU_QUIT, sel!(quit:), Some(SYMBOLS[4])),
    ]
    .into_iter()
    .map(|(t, action, image)| {
        let i = item(mtm, t.get(), action, &target);
        // the toggle keeps its check mark and has no image
        i.setImage(image.and_then(symbol).as_deref());
        (i, t)
    })
    .collect();
    let toggle = titled[0].0.clone();
    let waiting = item(mtm, &text::menu_waiting("", false), sel!(openDownload:), &target);
    waiting.setImage(symbol(SYMBOLS[2]).as_deref());
    waiting.setHidden(true);
    for (i, _) in &titled[..3] {
        menu.addItem(i);
    }
    menu.addItem(&waiting);
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    for (i, _) in &titled[3..] {
        menu.addItem(i);
    }
    menu.setDelegate(Some(ProtocolObject::from_ref(&*target)));
    item_.setMenu(Some(&menu));
    item_.button(mtm).ok_or("the status item has no button")?;

    let guide_texts: Vec<(Retained<NSTextField>, Text)> =
        [text::GUIDE_WHY, text::GUIDE_STEPS, text::GUIDE_RELAUNCH, text::GUIDE_WARNING].into_iter().map(|t| (label(mtm, t), t)).collect();
    let [why, steps, relaunch, warning] = [0, 1, 2, 3].map(|i| &guide_texts[i].0);
    // SAFETY: openSettings: is a method of Target
    let button = unsafe { NSButton::buttonWithTitle_target_action(&ns(text::GUIDE_BUTTON.get()), Some(&target), Some(sel!(openSettings:)), mtm) };
    let guide_status = label(mtm, text::guide_status(false));
    guide_status.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
    // a system colour: readable in light and dark
    warning.setTextColor(Some(&NSColor::systemRedColor()));
    let views: [&NSView; 6] = [why, steps, &button, &guide_status, relaunch, warning];
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
    guide.setTitle(&ns(text::GUIDE_TITLE.get()));
    guide.setContentView(Some(&stack));
    guide.setContentSize(stack.fittingSize());
    guide.center();
    settings::build(mtm, &target);
    UI.set(Some(Ui { target, item: item_, toggle, titled, waiting, guide, guide_texts, guide_button: button, guide_status, icon: Cell::new(None) }));
    Ok(())
}

/// The interface language changed: every menu title, the guide window (refitted) and the settings window (rebuilt). The
/// tooltip and the guide's status line follow at the next `show_state`.
pub fn relabel(mtm: MainThreadMarker) {
    let target = UI.with_borrow(|u| {
        let u = u.as_ref()?;
        for (i, t) in &u.titled {
            i.setTitle(&ns(t.get()));
        }
        for (l, t) in &u.guide_texts {
            l.setStringValue(&ns(t.get()));
        }
        u.guide_button.setTitle(&ns(text::GUIDE_BUTTON.get()));
        u.guide.setTitle(&ns(text::GUIDE_TITLE.get()));
        if let Some(v) = u.guide.contentView() {
            u.guide.setContentSize(v.fittingSize());
        }
        Some(u.target.clone())
    });
    if let Some(target) = target {
        settings::rebuild(mtm, &target);
    }
}

/// The self-test: the menu bar image for both glyphs in both shades, what the item shows, and every menu symbol.
pub fn self_test() -> Result<(), String> {
    for g in [Glyph::V, Glyph::E] {
        for dark in [false, true] {
            let reps = glyph_image(g, dark).map(|i| i.representations().count());
            if reps != Some(2) {
                return Err(format!("the menu bar image for {g:?} (dark {dark}) has {reps:?} bitmaps"));
            }
        }
    }
    if let Some(missing) = SYMBOLS.iter().find(|s| symbol(s).is_none()) {
        return Err(format!("no SF Symbol {missing}"));
    }
    let shown = UI.with_borrow(|u| u.as_ref().and_then(|u| u.item.button(u.target.mtm())).map(|b| (b.image().is_some(), b.title().to_string())));
    if shown != Some((true, String::new())) {
        return Err(format!("the menu bar item shows {shown:?}, not its image"));
    }
    println!("TrioKey self-test: menu bar images and menu symbols OK");
    Ok(())
}

/// The Quit item's title, for the self-test's language switch.
pub fn quit_title() -> Option<String> {
    UI.with_borrow(|u| u.as_ref().and_then(|u| u.titled.last()).map(|(i, _)| i.title().to_string()))
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
            // the shade is re-read on every refresh (the 2-second timer too), so a switch between light and dark is seen
            let want = (text::glyph(s), dark_bar(&b));
            if u.icon.get() != Some(want) {
                let image = glyph_image(want.0, want.1);
                b.setImage(image.as_deref());
                // without an image the letter stays: an empty item could not be found to quit
                b.setTitle(&ns(if image.is_some() { "" } else { text::title(s) }));
                u.icon.set(image.is_some().then_some(want));
            }
            let tip = ns(text::tooltip(s).get());
            b.setToolTip(Some(&tip));
            if let Some(i) = b.image() {
                i.setAccessibilityDescription(Some(&tip));
            }
        }
        u.toggle.setState(if on { NSControlStateValueOn } else { NSControlStateValueOff });
        u.guide_status.setStringValue(&ns(text::guide_status(has_permission).get()));
    });
}

pub fn show_waiting(version: Option<&str>) {
    UI.with_borrow(|u| {
        let Some(u) = u else { return };
        u.waiting.setHidden(version.is_none());
        if let Some(v) = version {
            u.waiting.setTitle(&ns(&text::menu_waiting(v, triokey_shared::english())));
        }
    });
}

/// Opens a fixed address (the releases page, the repository, the Accessibility settings) in the default handler.
fn open_url(url: &str) {
    if let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) {
        NSWorkspace::sharedWorkspace().openURL(&url);
    }
}

pub fn activate(mtm: MainThreadMarker) {
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

/// "Chép thông tin debug": works without permission and without a tap.
fn copy_debug(mtm: MainThreadMarker) {
    let d = app::diagnostics().render();
    let pb = NSPasteboard::generalPasteboard();
    pb.clearContents();
    // SAFETY: a constant AppKit exports
    pb.setString_forType(&NSString::from_str(&d), unsafe { NSPasteboardTypeString });
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&ns(text::DIAG_COPIED.get()));
    alert.setInformativeText(&ns(text::DIAG_COPIED_DETAIL.get()));
    activate(mtm);
    // no borrow of the app state is held here: the tap may run while the alert spins the run loop
    alert.runModal();
}
