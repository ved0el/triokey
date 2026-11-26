//! Pure IME-state rules (docs/ARCHITECTURE.md, 3.2 Rule per language). The thread that reads
//! the IME lives in `ime_watch.rs`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeState {
    /// Let every key go straight to the IME.
    Yield,
    /// TrioKey types Telex.
    Type,
}

const LANG_JAPANESE: u16 = 0x11;
const LANG_CHINESE: u16 = 0x04;
const LANG_KOREAN: u16 = 0x12;
const IME_CMODE_NATIVE: u32 = 1;
/// The first key after a possible IME toggle may wait this long for a fresh read.
pub const FIRST_KEY_WAIT_MS: u32 = 30;

/// `open`/`conv` are `None` when the read failed or timed out; then the window's last known
/// state wins, and with none, yield.
pub fn decide(lang_primary: u16, open: Option<bool>, conv: Option<u32>, last_known: Option<ImeState>) -> ImeState {
    let fallback = last_known.unwrap_or(ImeState::Yield);
    let yield_if = |b: bool| if b { ImeState::Yield } else { ImeState::Type };
    match (lang_primary, open) {
        (LANG_JAPANESE | LANG_CHINESE | LANG_KOREAN, None) => fallback,
        (LANG_JAPANESE, Some(o)) => yield_if(o),
        (_, Some(false)) => ImeState::Type,
        // Chinese/Korean open: conversion mode is needed too; a missing one counts as a failed read
        (LANG_CHINESE | LANG_KOREAN, Some(true)) => match conv {
            Some(c) => yield_if(c & IME_CMODE_NATIVE != 0),
            None => fallback,
        },
        _ => ImeState::Type,
    }
}

/// Layouts whose IME state matters; every other layout always types.
pub fn is_cjk(lang_primary: u16) -> bool {
    matches!(lang_primary, LANG_JAPANESE | LANG_CHINESE | LANG_KOREAN)
}

/// For the tray tooltip "đang nhường IME …" / "yielding to the … IME". Unknown (nothing read yet): the generic name.
pub fn cjk_name(lang_primary: u16, english: bool) -> &'static str {
    let pick = |vi, en| triokey_shared::pick(english, vi, en);
    match lang_primary {
        LANG_JAPANESE => pick("Nhật", "Japanese"),
        LANG_CHINESE => pick("Trung", "Chinese"),
        LANG_KOREAN => pick("Hàn", "Korean"),
        _ => "CJK",
    }
}

/// True while the first key may still wait for a fresh IME read.
pub fn first_key_wait(elapsed_ms: u32) -> bool {
    elapsed_ms < FIRST_KEY_WAIT_MS
}

#[cfg(test)]
mod tests {
    use super::*;
    use ImeState::*;

    const JA: u16 = 0x11;
    const ZH: u16 = 0x04;
    const KO: u16 = 0x12;
    const EN: u16 = 0x09;

    #[test]
    fn decide_cases() {
        // non-CJK: never read, always type
        assert_eq!(decide(EN, Some(true), Some(1), Some(Yield)), Type);
        assert_eq!(decide(EN, None, None, None), Type);
        // Japanese: open status alone decides
        assert_eq!(decide(JA, Some(true), Some(0), None), Yield);
        assert_eq!(decide(JA, Some(false), Some(1), Some(Yield)), Type);
        // Chinese / Korean: open AND native conversion
        assert_eq!(decide(ZH, Some(true), Some(1), None), Yield);
        assert_eq!(decide(ZH, Some(true), Some(0), None), Type);
        assert_eq!(decide(KO, Some(true), Some(0x9), None), Yield);
        assert_eq!(decide(KO, Some(false), Some(1), None), Type);
        // read failed: last known value, else yield
        assert_eq!(decide(JA, None, None, Some(Type)), Type);
        assert_eq!(decide(JA, None, None, None), Yield);
        assert_eq!(decide(KO, None, Some(1), Some(Type)), Type);
        assert_eq!(decide(ZH, Some(true), None, None), Yield);
        assert_eq!(decide(ZH, Some(true), None, Some(Type)), Type);
    }

    #[test]
    fn first_key_wait_limit() {
        assert!(first_key_wait(0));
        assert!(first_key_wait(29));
        assert!(!first_key_wait(30));
        assert!(!first_key_wait(u32::MAX));
    }
}

#[cfg(test)]
mod cjk_tests {
    use super::*;

    #[test]
    fn cjk_langs() {
        assert!(is_cjk(0x11) && is_cjk(0x04) && is_cjk(0x12));
        assert!(!is_cjk(0x09) && !is_cjk(0x2A));
        assert_eq!([0x11, 0x04, 0x12, 0, 0x09].map(|l| cjk_name(l, false)), ["Nhật", "Trung", "Hàn", "CJK", "CJK"]);
        assert_eq!([0x11, 0x04, 0x12, 0].map(|l| cjk_name(l, true)), ["Japanese", "Chinese", "Korean", "CJK"]);
    }
}
