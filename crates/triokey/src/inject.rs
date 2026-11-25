//! Turn an engine `Edit` into `SendInput` records. Building only; sending is Task 7.

use triokey_engine::Edit;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_BACK};

/// `dwExtraInfo` on every injected key so the hook skips its own input. Spells "EVJ1", the working name; kept so the value never changes.
pub const TRIOKEY_MARK: usize = 0x45564A31;

fn key(vk: u16, scan: u16, flags: u32) -> INPUT {
    INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: vk, wScan: scan, dwFlags: flags, time: 0, dwExtraInfo: TRIOKEY_MARK } } }
}

/// `delete` x Backspace (down, up), then each UTF-16 unit of `insert` as a Unicode key (down, up).
pub fn build_inputs(edit: &Edit) -> Vec<INPUT> {
    let back = (0..edit.delete).flat_map(|_| [key(VK_BACK, 0, 0), key(VK_BACK, 0, KEYEVENTF_KEYUP)]);
    let text = edit.insert.encode_utf16().flat_map(|u| [key(0, u, KEYEVENTF_UNICODE), key(0, u, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP)]);
    back.chain(text).collect()
}

/// One real key press (down, up), e.g. Enter after a replaced word.
pub fn tap(vk: u16) -> [INPUT; 2] {
    [key(vk, 0, 0), key(vk, 0, KEYEVENTF_KEYUP)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ki(i: &INPUT) -> KEYBDINPUT {
        assert_eq!(i.r#type, INPUT_KEYBOARD);
        unsafe { i.Anonymous.ki }
    }

    #[test]
    fn delete_two_insert_one() {
        let v = build_inputs(&Edit { delete: 2, insert: "ế".into() });
        assert_eq!(v.len(), 6);
        assert!(v.iter().all(|i| ki(i).dwExtraInfo == TRIOKEY_MARK));
        for (n, i) in v[..4].iter().enumerate() {
            let k = ki(i);
            assert_eq!(k.wVk, VK_BACK);
            assert_eq!(k.dwFlags, if n % 2 == 0 { 0 } else { KEYEVENTF_KEYUP });
        }
        let (d, u) = (ki(&v[4]), ki(&v[5]));
        assert_eq!((d.wVk, d.wScan, d.dwFlags), (0, 0x1EBF, KEYEVENTF_UNICODE));
        assert_eq!((u.wScan, u.dwFlags), (0x1EBF, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
    }

    #[test]
    fn surrogate_pair_is_two_units() {
        let v = build_inputs(&Edit { delete: 0, insert: "😀".into() });
        assert_eq!(v.len(), 4);
        assert_eq!(ki(&v[0]).wScan, 0xD83D);
        assert_eq!(ki(&v[2]).wScan, 0xDE00);
    }
}
