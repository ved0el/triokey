use crate::Lang;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const EN_WORDS: &str = include_str!("../data/en_words.txt");
const AMBIGUOUS: &str = include_str!("../data/ambiguous_defaults.txt");

/// Lines with everything from `#` on dropped, trimmed, blanks skipped.
fn entries(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(|l| l.split('#').next().unwrap_or("").trim()).filter(|l| !l.is_empty())
}

fn en_words() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| entries(EN_WORDS).collect())
}

fn tie_defaults() -> &'static HashMap<&'static str, Lang> {
    static MAP: OnceLock<HashMap<&'static str, Lang>> = OnceLock::new();
    MAP.get_or_init(|| {
        entries(AMBIGUOUS)
            .filter_map(|l| match l.split_once('\t')? {
                (w, "en") => Some((w, Lang::En)),
                (w, "vi") => Some((w, Lang::Vi)),
                _ => None,
            })
            .collect()
    })
}

/// English word list (SCOWL), per-word defaults for English/Vietnamese ties,
/// and the user's two force files. All lookups take lowercase keys.
#[derive(Clone, Debug, Default)]
pub struct Lexicon {
    forced: HashMap<String, Lang>,
}

impl Lexicon {
    pub fn builtin() -> Lexicon {
        Lexicon::default()
    }

    /// File contents, one word per line, `#` starts a comment. Words are the typed keys
    /// (`this`, not `thí`), lowercased. A word in both files ends up English.
    pub fn with_user(mut self, force_vi: &str, force_en: &str) -> Lexicon {
        for (text, lang) in [(force_vi, Lang::Vi), (force_en, Lang::En)] {
            for w in entries(text) {
                self.forced.insert(w.to_lowercase(), lang);
            }
        }
        self
    }

    /// In the English list; the force files override it either way.
    pub fn is_english(&self, word_lower: &str) -> bool {
        match self.forced(word_lower) {
            Some(lang) => lang == Lang::En,
            None => en_words().contains(word_lower),
        }
    }

    /// Default reading when keys are both an English word and a Vietnamese syllable.
    /// Keys not in the table are not a tie, so they read as Vietnamese.
    pub fn tie_default(&self, keys_lower: &str) -> Lang {
        tie_defaults().get(keys_lower).copied().unwrap_or(Lang::Vi)
    }

    pub fn forced(&self, keys_lower: &str) -> Option<Lang> {
        self.forced.get(keys_lower).copied()
    }
}
