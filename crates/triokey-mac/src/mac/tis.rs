//! The current input source and Secure Input, from Carbon (HIToolbox). No maintained crate covers TIS and Apple has no
//! current reference for it: these declarations follow TextInputSources.h. TIS must be called on the main thread only.

use std::ffi::c_void;
use std::ptr::NonNull;

use objc2_core_foundation::{CFBoolean, CFRetained, CFString, CFType};
use triokey_mac::source::Source;

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    /// +1 reference; NULL with some CJK input methods.
    fn TISCopyCurrentKeyboardInputSource() -> *mut c_void;
    /// Not retained: valid while the source is.
    fn TISGetInputSourceProperty(source: *mut c_void, key: *const CFString) -> *mut c_void;
    static kTISPropertyInputSourceID: *const CFString;
    static kTISPropertyInputSourceType: *const CFString;
    static kTISPropertyInputModeID: *const CFString;
    static kTISPropertyInputSourceIsASCIICapable: *const CFString;
    pub static kTISNotifySelectedKeyboardInputSourceChanged: *const CFString;
    /// `Boolean`: while any process has it on, no event tap receives keyboard events (TN2150).
    fn IsSecureEventInputEnabled() -> u8;
}

pub fn current() -> Source {
    // SAFETY: called on the main thread (every caller holds a MainThreadMarker); the copied source is released on drop and
    // every property is read, and copied out, while it is alive.
    unsafe {
        let Some(src) = NonNull::new(TISCopyCurrentKeyboardInputSource().cast::<CFType>()) else { return Source::default() };
        let src = CFRetained::from_raw(src);
        let raw = CFRetained::as_ptr(&src).as_ptr().cast::<c_void>();
        let prop = |key: *const CFString| -> Option<&CFType> {
            if key.is_null() {
                return None;
            }
            TISGetInputSourceProperty(raw, key).cast::<CFType>().as_ref()
        };
        let text = |key| prop(key).and_then(|v| v.downcast_ref::<CFString>()).map(|s| s.to_string());
        Source {
            id: text(kTISPropertyInputSourceID),
            kind: text(kTISPropertyInputSourceType),
            mode: text(kTISPropertyInputModeID),
            ascii: prop(kTISPropertyInputSourceIsASCIICapable).and_then(|v| v.downcast_ref::<CFBoolean>()).map(|b| b.value()),
        }
    }
}

pub fn secure_input() -> bool {
    // SAFETY: no arguments, reads a flag
    unsafe { IsSecureEventInputEnabled() != 0 }
}
