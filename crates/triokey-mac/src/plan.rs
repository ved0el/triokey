//! What to post for an engine `Outcome` (the macOS counterpart of crates/triokey/src/inject.rs and `split_boundary`).

use triokey_engine::Outcome;

use crate::keymap::{vk, KeyClass};

/// Developers report that a longer Unicode string in one key event is cut at 20 UTF-16 units.
pub const CHUNK: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub backspaces: usize,
    pub text: String,
    /// A real key after the text: Return or Tab typed as a character does not act as the key in most apps.
    pub key: Option<u16>,
}

/// `None`: let the original event through.
pub fn plan(out: Outcome, class: KeyClass) -> Option<Plan> {
    let Outcome::Replace(edit) = out else { return None };
    let mut p = Plan { backspaces: edit.delete, text: edit.insert, key: None };
    let (c, key) = match class {
        KeyClass::Boundary('\n') => ('\n', vk::RETURN),
        KeyClass::Boundary('\t') => ('\t', vk::TAB),
        _ => return Some(p),
    };
    if p.text.ends_with(c) {
        p.text.pop();
        p.key = Some(key);
    }
    Some(p)
}

/// `text` as UTF-16 pieces of at most `CHUNK` units; a surrogate pair is never split.
pub fn chunks(text: &str) -> Vec<Vec<u16>> {
    let mut out: Vec<Vec<u16>> = Vec::new();
    let mut buf = [0u16; 2];
    for c in text.chars() {
        let u = c.encode_utf16(&mut buf);
        match out.last_mut() {
            Some(last) if last.len() + u.len() <= CHUNK => last.extend_from_slice(u),
            _ => out.push(u.to_vec()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use triokey_engine::Edit;

    fn rep(delete: usize, insert: &str) -> Outcome {
        Outcome::Replace(Edit { delete, insert: insert.into() })
    }

    #[test]
    fn pass_posts_nothing() {
        assert_eq!(plan(Outcome::Pass, KeyClass::Letter('a')), None);
    }

    #[test]
    fn letter_and_space() {
        assert_eq!(plan(rep(2, "ế"), KeyClass::Letter('s')), Some(Plan { backspaces: 2, text: "ế".into(), key: None }));
        assert_eq!(plan(rep(3, "việt "), KeyClass::Boundary(' ')), Some(Plan { backspaces: 3, text: "việt ".into(), key: None }));
    }

    #[test]
    fn enter_and_tab_become_real_keys() {
        assert_eq!(plan(rep(4, "việt\n"), KeyClass::Boundary('\n')), Some(Plan { backspaces: 4, text: "việt".into(), key: Some(vk::RETURN) }));
        assert_eq!(plan(rep(1, "a\t"), KeyClass::Boundary('\t')), Some(Plan { backspaces: 1, text: "a".into(), key: Some(vk::TAB) }));
        // an edit that does not end with the boundary keeps its text and sends no key
        assert_eq!(plan(rep(1, "a"), KeyClass::Boundary('\n')), Some(Plan { backspaces: 1, text: "a".into(), key: None }));
    }

    #[test]
    fn chunks_cut_at_20_units() {
        assert!(chunks("").is_empty());
        let s = "a".repeat(45);
        let c = chunks(&s);
        assert_eq!(c.iter().map(Vec::len).collect::<Vec<_>>(), [20, 20, 5]);
        assert_eq!(String::from_utf16(&c.concat()).unwrap(), s);
    }

    #[test]
    fn chunks_keep_surrogate_pairs_whole() {
        // 19 units then an astral character: the pair would straddle the cut, so it opens the next piece
        let s = format!("{}😀b", "a".repeat(19));
        let c = chunks(&s);
        assert_eq!(c.iter().map(Vec::len).collect::<Vec<_>>(), [19, 3]);
        assert_eq!(c[1][..2], [0xD83D, 0xDE00]);
        assert!(c.iter().all(|p| p.len() <= CHUNK && String::from_utf16(p).is_ok()));
    }
}
