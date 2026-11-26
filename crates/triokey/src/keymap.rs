//! Classify a low-level key event for the engine.

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyClass {
    Letter(char),
    Boundary(char),
    Backspace,
    /// Cursor moved or a shortcut ran: drop the word. The caller should also re-read the IME.
    BreakWord,
    /// Key that types nothing (F-keys, Kanji, IME keys...): may have toggled the IME.
    MaybeImeToggle,
    Ignore,
}

/// `ch` is what the key types on the current layout (from `ToUnicode`), if anything.
pub fn classify(vk: u16, ch: Option<char>, ctrl: bool, alt: bool, win: bool) -> KeyClass {
    use KeyClass::*;
    match vk {
        VK_SHIFT | VK_CONTROL | VK_MENU | VK_LSHIFT | VK_RSHIFT | VK_LCONTROL | VK_RCONTROL | VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN => return Ignore,
        _ => {}
    }
    // Ctrl+Alt with a character is AltGr on many layouts: it types, it is not a shortcut.
    let altgr_char = ctrl && alt && !win && ch.is_some_and(|c| !c.is_control());
    if (ctrl || alt || win) && !altgr_char {
        return BreakWord;
    }
    match vk {
        VK_BACK => return Backspace,
        VK_RETURN => return Boundary('\n'),
        VK_TAB => return Boundary('\t'),
        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_DELETE | VK_INSERT | VK_ESCAPE => return BreakWord,
        _ => {}
    }
    match ch {
        Some(c) if c.is_ascii_alphabetic() => Letter(c),
        // a letter from another layout (e.g. `ú`) is outside Telex: drop the word
        Some(c) if c.is_alphabetic() || c.is_control() => BreakWord,
        Some(c) => Boundary(c),
        None => MaybeImeToggle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyClass::*;

    #[test]
    fn classify_cases() {
        assert_eq!(classify(0x41, Some('a'), false, false, false), Letter('a'));
        assert_eq!(classify(0x41, Some('A'), false, false, false), Letter('A'));
        assert_eq!(classify(VK_SPACE, Some(' '), false, false, false), Boundary(' '));
        assert_eq!(classify(0x31, Some('1'), false, false, false), Boundary('1'));
        assert_eq!(classify(VK_OEM_PERIOD, Some('.'), false, false, false), Boundary('.'));
        assert_eq!(classify(VK_RETURN, Some('\r'), false, false, false), Boundary('\n'));
        assert_eq!(classify(VK_TAB, Some('\t'), false, false, false), Boundary('\t'));
        assert_eq!(classify(VK_BACK, Some('\u{8}'), false, false, false), Backspace);
        assert_eq!(classify(0x43, Some('c'), true, false, false), BreakWord);
        assert_eq!(classify(VK_BACK, None, true, false, false), BreakWord);
        assert_eq!(classify(VK_TAB, Some('\t'), false, true, false), BreakWord);
        assert_eq!(classify(0x45, Some('e'), false, false, true), BreakWord);
        for vk in [VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN, VK_HOME, VK_END, VK_DELETE, VK_ESCAPE] {
            assert_eq!(classify(vk, None, false, false, false), BreakWord, "vk {vk:#x}");
        }
        // directly typed non-ASCII letter resets the word
        assert_eq!(classify(0xBA, Some('ú'), false, false, false), BreakWord);
        // AltGr (Ctrl+Alt) that types a character is a character
        assert_eq!(classify(0x51, Some('@'), true, true, false), Boundary('@'));
        assert_eq!(classify(VK_F5, None, false, false, false), MaybeImeToggle);
        assert_eq!(classify(VK_KANJI, None, false, false, false), MaybeImeToggle);
        assert_eq!(classify(VK_OEM_3, None, false, true, false), BreakWord);
        for vk in [VK_SHIFT, VK_LSHIFT, VK_RCONTROL, VK_LMENU, VK_LWIN, VK_RWIN] {
            assert_eq!(classify(vk, None, false, false, false), Ignore, "vk {vk:#x}");
        }
        assert_eq!(classify(VK_LSHIFT, None, true, false, false), Ignore);
    }
}
