//! Classify a macOS key-down for the engine (same classes as crates/triokey/src/keymap.rs).

/// `kCGEventFlagMaskShift`.
pub const FLAG_SHIFT: u64 = 1 << 17;
/// `kCGEventFlagMaskControl` and `kCGEventFlagMaskCommand` (CGEventTypes.h).
pub const FLAG_CONTROL: u64 = 1 << 18;
pub const FLAG_COMMAND: u64 = 1 << 20;
/// `kCGEventFlagMaskAlternate` (Option).
pub const FLAG_OPTION: u64 = 1 << 19;

/// macOS virtual key codes (Events.h, `kVK_*`); the same on every layout.
pub mod vk {
    pub const RETURN: u16 = 36;
    pub const TAB: u16 = 48;
    pub const DELETE: u16 = 51;
    pub const ESCAPE: u16 = 53;
    pub const KEYPAD_ENTER: u16 = 76;
    pub const HOME: u16 = 115;
    pub const PAGE_UP: u16 = 116;
    pub const FORWARD_DELETE: u16 = 117;
    pub const END: u16 = 119;
    pub const PAGE_DOWN: u16 = 121;
    pub const LEFT: u16 = 123;
    pub const RIGHT: u16 = 124;
    pub const DOWN: u16 = 125;
    pub const UP: u16 = 126;
    /// The key code the posted text rides on: one no keyboard has (Events.h defines no kVK_* 0x46). Never 0 (`A`): while
    /// the real A is still down from fast typing (`ma` then `w`), apps dropped text posted on A's code, so `mă` showed `m`.
    pub const TEXT: u16 = 0x46;
    /// F1..F20, Help: type nothing a word can hold.
    pub const FUNCTION: [u16; 21] = [122, 120, 99, 118, 96, 97, 98, 100, 101, 109, 103, 111, 105, 107, 113, 106, 64, 79, 80, 90, 114];
    /// Command, Shift, Caps Lock, Option, Control (left and right), Fn: normally flags-changed events, never typing.
    pub const MODIFIERS: [u16; 10] = [54, 55, 56, 57, 58, 59, 60, 61, 62, 63];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyClass {
    Letter(char),
    Boundary(char),
    Backspace,
    /// Cursor moved, a shortcut ran or the key typed something outside Telex: drop the word, let the key through.
    BreakWord,
    Ignore,
}

/// `text` is what the event types (`CGEventKeyboardGetUnicodeString`), `flags` its `CGEventFlags`.
pub fn classify(keycode: u16, text: &str, flags: u64, autorepeat: bool) -> KeyClass {
    use KeyClass::*;
    if vk::MODIFIERS.contains(&keycode) {
        return Ignore;
    }
    if flags & (FLAG_COMMAND | FLAG_CONTROL) != 0 {
        return BreakWord;
    }
    match keycode {
        // Option+Delete deletes a whole word: the engine cannot follow it
        vk::DELETE if flags & FLAG_OPTION != 0 => return BreakWord,
        vk::DELETE => return Backspace,
        // a held key repeating is not Telex typing; replacing words at repeat rate is where a first build goes wrong
        _ if autorepeat => return BreakWord,
        vk::RETURN | vk::KEYPAD_ENTER => return Boundary('\n'),
        vk::TAB => return Boundary('\t'),
        vk::ESCAPE | vk::LEFT | vk::RIGHT | vk::UP | vk::DOWN | vk::HOME | vk::END | vk::PAGE_UP | vk::PAGE_DOWN | vk::FORWARD_DELETE => return BreakWord,
        k if vk::FUNCTION.contains(&k) => return BreakWord,
        _ => {}
    }
    let mut it = text.chars();
    match (it.next(), it.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => Letter(c),
        (Some(c), None) if c.is_ascii_graphic() || c == ' ' => Boundary(c),
        // empty (dead key), several characters, an Option letter such as `å`, a control character
        _ => BreakWord,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyClass::*;

    #[test]
    fn letters_and_boundaries() {
        assert_eq!(classify(0, "a", 0, false), Letter('a'));
        assert_eq!(classify(0, "A", 1 << 17, false), Letter('A'));
        assert_eq!(classify(49, " ", 0, false), Boundary(' '));
        assert_eq!(classify(18, "1", 0, false), Boundary('1'));
        assert_eq!(classify(47, ".", 0, false), Boundary('.'));
        assert_eq!(classify(19, "@", 1 << 17, false), Boundary('@'));
    }

    #[test]
    fn named_keys() {
        assert_eq!(classify(vk::DELETE, "\u{8}", 0, false), Backspace);
        assert_eq!(classify(vk::RETURN, "\r", 0, false), Boundary('\n'));
        assert_eq!(classify(vk::KEYPAD_ENTER, "\u{3}", 0, false), Boundary('\n'));
        assert_eq!(classify(vk::TAB, "\t", 0, false), Boundary('\t'));
        for k in [vk::ESCAPE, vk::LEFT, vk::RIGHT, vk::UP, vk::DOWN, vk::HOME, vk::END, vk::PAGE_UP, vk::PAGE_DOWN, vk::FORWARD_DELETE, 122, 111] {
            assert_eq!(classify(k, "\u{f702}", 0, false), BreakWord, "key {k}");
        }
        for k in vk::MODIFIERS {
            assert_eq!(classify(k, "", FLAG_COMMAND, false), Ignore, "key {k}");
        }
    }

    #[test]
    fn command_or_control_breaks() {
        assert_eq!(classify(8, "c", FLAG_COMMAND, false), BreakWord);
        assert_eq!(classify(8, "\u{3}", FLAG_CONTROL, false), BreakWord);
        assert_eq!(classify(vk::DELETE, "\u{8}", FLAG_COMMAND, false), BreakWord);
        assert_eq!(classify(vk::RETURN, "\r", FLAG_CONTROL, false), BreakWord);
        assert_eq!(classify(vk::DELETE, "\u{8}", FLAG_OPTION, false), BreakWord);
        // Shift or Option with Return stays a boundary: the real Return is posted with the original flags
        assert_eq!(classify(vk::RETURN, "\r", FLAG_OPTION | 1 << 17, false), Boundary('\n'));
    }

    #[test]
    fn text_outside_telex_breaks() {
        assert_eq!(classify(0, "å", 1 << 19, false), BreakWord);
        assert_eq!(classify(49, "\u{a0}", 1 << 19, false), BreakWord);
        assert_eq!(classify(32, "", 0, false), BreakWord);
        assert_eq!(classify(0, "ab", 0, false), BreakWord);
        assert_eq!(classify(0, "ú", 0, false), BreakWord);
    }

    #[test]
    fn autorepeat_only_backspace_counts() {
        assert_eq!(classify(0, "a", 0, true), BreakWord);
        assert_eq!(classify(49, " ", 0, true), BreakWord);
        assert_eq!(classify(vk::DELETE, "\u{8}", 0, true), Backspace);
    }
}
