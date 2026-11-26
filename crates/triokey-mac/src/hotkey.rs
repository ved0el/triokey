//! The hotkey that switches TrioKey on and off (docs/ARCHITECTURE.md, 12.13 Settings, files and the window). The macOS
//! counterpart of `hotkey_step` in crates/triokey/src/hook.rs, fed from the tap: modifier-changed events, key-downs and
//! mouse-downs.

use triokey_shared::config::Hotkey;

use crate::keymap::{FLAG_COMMAND, FLAG_CONTROL, FLAG_OPTION, FLAG_SHIFT};

/// The four modifiers a hotkey looks at. Real flags also carry Caps Lock, Fn, the non-coalesced bit and the
/// left/right device bits (IOLLEvent.h), which never decide a hotkey.
const MODS: u64 = FLAG_SHIFT | FLAG_CONTROL | FLAG_OPTION | FLAG_COMMAND;
/// `kVK_ANSI_Z` (Events.h): the key at the Z position of an ANSI keyboard.
pub const VK_Z: u16 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hot {
    None,
    /// Toggle, let the event through.
    Toggle,
    /// Toggle and swallow the key.
    ToggleSwallow,
    /// Swallow only: the auto-repeat of a swallowed hotkey press.
    Swallow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HotState {
    /// Exactly Control and Shift are held.
    full: bool,
    /// Another modifier, a key or a click came while a modifier was held; cleared once all four are up.
    spoiled: bool,
}

impl HotState {
    /// A modifier-changed event; `flags` are the event's flags after the change. The event always passes.
    /// Control+Shift fires when one of the two is released after both were held with no other key or modifier.
    pub fn flags_changed(&mut self, hk: Hotkey, flags: u64) -> Hot {
        let m = flags & MODS;
        const BOTH: u64 = FLAG_CONTROL | FLAG_SHIFT;
        if m & !BOTH != 0 {
            self.spoiled = true;
        }
        let fire = self.full && !self.spoiled && m != BOTH;
        self.full = m == BOTH;
        if m == 0 {
            self.spoiled = false;
        }
        if fire && hk == Hotkey::CtrlShift {
            Hot::Toggle
        } else {
            Hot::None
        }
    }

    /// A key-down; Option+Z (exactly Option among the four modifiers) toggles and is swallowed.
    pub fn key_down(&mut self, hk: Hotkey, keycode: u16, flags: u64, autorepeat: bool) -> Hot {
        self.mouse_down(flags);
        match hk {
            Hotkey::AltZ if keycode == VK_Z && flags & MODS == FLAG_OPTION => {
                if autorepeat {
                    Hot::Swallow
                } else {
                    Hot::ToggleSwallow
                }
            }
            _ => Hot::None,
        }
    }

    /// A mouse-down, or any key-down: `flags` are the event's.
    pub fn mouse_down(&mut self, flags: u64) {
        self.full = false;
        self.spoiled |= flags & MODS != 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Hotkey::*;

    // what real events carry besides the four modifiers: non-coalesced, left Control, left Shift, left Option
    const NC: u64 = 0x100;
    const LCTL: u64 = 0x1;
    const LSHIFT: u64 = 0x2;
    const LALT: u64 = 0x20;
    const CAPS: u64 = 1 << 16;

    #[test]
    fn control_shift_fires_on_release_with_nothing_between() {
        let mut s = HotState::default();
        assert_eq!(s.flags_changed(CtrlShift, NC | FLAG_CONTROL | LCTL), Hot::None);
        assert_eq!(s.flags_changed(CtrlShift, NC | FLAG_CONTROL | LCTL | FLAG_SHIFT | LSHIFT), Hot::None);
        assert_eq!(s.flags_changed(CtrlShift, NC | FLAG_CONTROL | LCTL), Hot::Toggle);
        assert_eq!(s.flags_changed(CtrlShift, NC), Hot::None);
        // Shift first, released together; Caps Lock on does not matter
        s.flags_changed(CtrlShift, CAPS | FLAG_SHIFT | LSHIFT);
        s.flags_changed(CtrlShift, CAPS | FLAG_SHIFT | LSHIFT | FLAG_CONTROL | LCTL);
        assert_eq!(s.flags_changed(CtrlShift, CAPS | NC), Hot::Toggle);
    }

    #[test]
    fn control_shift_is_cancelled_by_a_key_a_click_or_another_modifier() {
        let mut s = HotState::default();
        let both = FLAG_CONTROL | FLAG_SHIFT | LCTL | LSHIFT;
        s.flags_changed(CtrlShift, both);
        assert_eq!(s.key_down(CtrlShift, 17, both, false), Hot::None);
        assert_eq!(s.flags_changed(CtrlShift, FLAG_CONTROL), Hot::None);
        s.flags_changed(CtrlShift, 0);
        s.flags_changed(CtrlShift, both);
        s.mouse_down(both);
        assert_eq!(s.flags_changed(CtrlShift, FLAG_SHIFT), Hot::None);
        s.flags_changed(CtrlShift, 0);
        s.flags_changed(CtrlShift, both);
        s.flags_changed(CtrlShift, both | FLAG_OPTION | LALT);
        assert_eq!(s.flags_changed(CtrlShift, both), Hot::None);
        assert_eq!(s.flags_changed(CtrlShift, 0), Hot::None, "Option came in between");
        // a key typed with no modifier held spoils nothing
        s.key_down(CtrlShift, 0, NC, false);
        s.flags_changed(CtrlShift, both);
        assert_eq!(s.flags_changed(CtrlShift, FLAG_SHIFT), Hot::Toggle);
        // Shift alone, Control alone, Command+Shift
        for f in [FLAG_SHIFT, FLAG_CONTROL, FLAG_COMMAND | FLAG_SHIFT] {
            s.flags_changed(CtrlShift, f);
            assert_eq!(s.flags_changed(CtrlShift, 0), Hot::None);
        }
    }

    #[test]
    fn other_settings_ignore_control_shift() {
        for hk in [AltZ, Off] {
            let mut s = HotState::default();
            s.flags_changed(hk, FLAG_CONTROL | FLAG_SHIFT);
            assert_eq!(s.flags_changed(hk, 0), Hot::None);
        }
    }

    #[test]
    fn option_z_toggles_and_swallows_once() {
        let mut s = HotState::default();
        assert_eq!(s.key_down(AltZ, VK_Z, NC | FLAG_OPTION | LALT, false), Hot::ToggleSwallow);
        assert_eq!(s.key_down(AltZ, VK_Z, NC | FLAG_OPTION | LALT | CAPS, true), Hot::Swallow);
        assert_eq!(s.key_down(AltZ, VK_Z, 0, false), Hot::None);
        assert_eq!(s.key_down(AltZ, VK_Z, FLAG_OPTION | FLAG_SHIFT, false), Hot::None);
        assert_eq!(s.key_down(AltZ, VK_Z, FLAG_OPTION | FLAG_COMMAND, false), Hot::None);
        assert_eq!(s.key_down(AltZ, 7, FLAG_OPTION, false), Hot::None);
        for hk in [CtrlShift, Off] {
            assert_eq!(s.key_down(hk, VK_Z, FLAG_OPTION, false), Hot::None);
        }
    }
}
