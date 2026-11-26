//! The current input source and whether TrioKey types under it (docs/ARCHITECTURE.md, 12. macOS app).

/// Value of `kTISTypeKeyboardLayout`: a plain keyboard layout, no input method or input mode.
pub const TYPE_KEYBOARD_LAYOUT: &str = "TISTypeKeyboardLayout";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Type,
    /// Another input method may be composing: step aside.
    Yield,
}

/// What TIS reports; `None` is a property that could not be read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Source {
    pub id: Option<String>,
    pub kind: Option<String>,
    pub mode: Option<String>,
    pub ascii: Option<bool>,
}

impl Source {
    /// Deliberately conservative for the first build: anything but an ASCII-capable plain layout, including a source
    /// that could not be read, yields.
    pub fn decision(&self) -> Decision {
        if self.kind.as_deref() == Some(TYPE_KEYBOARD_LAYOUT) && self.ascii == Some(true) {
            Decision::Type
        } else {
            Decision::Yield
        }
    }
}

/// Distinct sources seen since launch, for the diagnostics.
pub const SEEN_MAX: usize = 32;

pub fn remember(seen: &mut Vec<Source>, s: &Source) {
    if seen.len() < SEEN_MAX && !seen.contains(s) {
        seen.push(s.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(kind: Option<&str>, ascii: Option<bool>) -> Source {
        Source { id: Some("x".into()), kind: kind.map(Into::into), mode: None, ascii }
    }

    #[test]
    fn only_ascii_layout_types() {
        assert_eq!(src(Some(TYPE_KEYBOARD_LAYOUT), Some(true)).decision(), Decision::Type);
        assert_eq!(src(Some(TYPE_KEYBOARD_LAYOUT), Some(false)).decision(), Decision::Yield);
        assert_eq!(src(Some(TYPE_KEYBOARD_LAYOUT), None).decision(), Decision::Yield);
        assert_eq!(src(Some("TISTypeKeyboardInputMode"), Some(true)).decision(), Decision::Yield);
        assert_eq!(src(Some("TISTypeKeyboardInputMethodWithoutModes"), Some(true)).decision(), Decision::Yield);
        assert_eq!(src(None, Some(true)).decision(), Decision::Yield);
        assert_eq!(Source::default().decision(), Decision::Yield);
    }

    #[test]
    fn remember_dedups_and_caps() {
        let mut seen = Vec::new();
        remember(&mut seen, &src(None, None));
        remember(&mut seen, &src(None, None));
        assert_eq!(seen.len(), 1);
        for i in 0..40 {
            remember(&mut seen, &Source { id: Some(i.to_string()), ..Source::default() });
        }
        assert_eq!(seen.len(), SEEN_MAX);
    }
}
