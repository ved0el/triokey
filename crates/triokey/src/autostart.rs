//! Start with Windows in normal mode: `HKCU\...\Run` value `TrioKey` (spec 5.7). Never touches
//! `HKCU\Keyboard Layout` or the language list (AT6).

use std::path::Path;
use std::ptr::null_mut;

use windows_sys::Win32::System::Registry::*;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "TrioKey";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

/// The Run value data: the exe path in quotes, so a path with spaces is not split.
pub fn run_value(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

/// `value` (the Run data) starts `exe`. Windows paths: compared without case.
pub fn points_at(value: &str, exe: &Path) -> bool {
    value.to_lowercase() == run_value(exe).to_lowercase()
}

/// The Run value exists and starts this exe. A value left behind by a moved or renamed copy
/// counts as off, so ticking the box again repairs it.
pub fn is_on() -> bool {
    let Ok(exe) = std::env::current_exe() else { return false };
    // longer data fails with ERROR_MORE_DATA: off
    let mut buf = [0u16; 1024];
    let mut bytes = (buf.len() * 2) as u32;
    let r = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), wide(VALUE).as_ptr(), RRF_RT_REG_SZ, null_mut(), buf.as_mut_ptr().cast(), &mut bytes)
    };
    // `bytes` counts the terminating NUL
    r == 0 && points_at(&String::from_utf16_lossy(&buf[..(bytes as usize / 2).saturating_sub(1)]), &exe)
}

/// `on`: point the value at `exe`. Off: delete it (already absent counts as success).
pub fn set(on: bool, exe: &Path) -> bool {
    unsafe {
        if on {
            let data = wide(&run_value(exe));
            let bytes = (data.len() * 2) as u32;
            RegSetKeyValueW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), wide(VALUE).as_ptr(), REG_SZ, data.as_ptr().cast(), bytes) == 0
        } else {
            let r = RegDeleteKeyValueW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), wide(VALUE).as_ptr());
            r == 0 || r == windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND
        }
    }
}

/// A DWORD under HKCU, `None` when missing.
pub fn hkcu_dword(key: &str, value: &str) -> Option<u32> {
    let (mut v, mut size) = (0u32, 4u32);
    let r = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, wide(key).as_ptr(), wide(value).as_ptr(), RRF_RT_REG_DWORD, null_mut(), (&mut v as *mut u32).cast(), &mut size)
    };
    (r == 0).then_some(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_value_is_quoted() {
        assert_eq!(run_value(Path::new(r"C:\Program Files\TrioKey\triokey.exe")), r#""C:\Program Files\TrioKey\triokey.exe""#);
        assert_eq!(run_value(Path::new(r"D:\tools\triokey.exe")), r#""D:\tools\triokey.exe""#);
    }

    #[test]
    fn run_value_of_another_copy_is_off() {
        let exe = Path::new(r"D:\tools\triokey.exe");
        assert!(points_at(r#""d:\Tools\TrioKey.exe""#, exe));
        assert!(!points_at(r#""D:\old\triokey.exe""#, exe));
        assert!(!points_at(r"D:\tools\triokey.exe", exe));
        assert!(!points_at("", exe));
    }
}
