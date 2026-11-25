//! Typing engine: recomputes the whole word from its raw keys after every key (spec 5.4).

use crate::lexicon::Lexicon;
use crate::syllable::{is_loose_prefix, is_syllable};
use crate::telex::compose;
use crate::token::{is_mixed_case, starts_code};
use crate::{Lang, ToneStyle};

/// Delete `delete` chars before the cursor (one per on-screen char), then insert `insert`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub delete: usize,
    pub insert: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Let the original key through untouched.
    Pass,
    /// Swallow the key and apply this edit instead.
    Replace(Edit),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub tone_style: ToneStyle,
    pub auto_english: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config { tone_style: ToneStyle::Old, auto_english: true }
    }
}

#[derive(Clone, Debug)]
pub struct Engine {
    config: Config,
    lexicon: Lexicon,
    // Current word.
    keys: Vec<char>,
    /// `cancel[i]`: key `i` repeated key `i-1` and undid the mark it made (`verr`, `xooo`).
    cancel: Vec<bool>,
    composed: String,
    shown: String,
    /// Word can no longer be Vietnamese: raw keys until it ends.
    raw: bool,
    // Current cluster (text between whitespace).
    code: bool,
    cluster_empty: bool,
    after_code_sep: bool,
    /// Clear language of the previous committed word on this line (rule 4).
    context: Option<Lang>,
    /// (chars to delete, text to insert) for the backspace right after a commit.
    undo: Option<(usize, String)>,
}

impl Engine {
    pub fn new(config: Config, lexicon: Lexicon) -> Engine {
        Engine {
            config,
            lexicon,
            keys: Vec::new(),
            cancel: Vec::new(),
            composed: String::new(),
            shown: String::new(),
            raw: false,
            code: false,
            cluster_empty: true,
            after_code_sep: false,
            context: None,
            undo: None,
        }
    }

    /// A letter `a-z`/`A-Z`. Anything else just drops the word being typed.
    pub fn key(&mut self, ch: char) -> Outcome {
        self.undo = None;
        if !ch.is_ascii_alphabetic() {
            self.reset();
            return Outcome::Pass;
        }
        if self.after_code_sep && self.config.auto_english {
            self.code = true;
        }
        self.after_code_sep = false;
        self.cluster_empty = false;
        self.push(ch);
        let target = self.typing_target();
        let out = if target == format!("{}{ch}", self.shown) { Outcome::Pass } else { edit(&self.shown, &target) };
        self.shown = target;
        out
    }

    /// Any printable non-letter, including `' '` and `'\n'`: commits the word.
    /// A `Replace` ends with `ch` itself.
    pub fn boundary(&mut self, ch: char) -> Outcome {
        self.undo = None;
        let space = ch.is_whitespace();
        if self.config.auto_english && !space && ((self.cluster_empty && starts_code(ch)) || ch.is_ascii_digit() || ch == '_' || self.after_code_sep) {
            self.code = true;
        }
        let out = if self.keys.is_empty() {
            Outcome::Pass
        } else {
            let (word, alt, class) = self.commit_word();
            if self.config.auto_english && matches!(ch, '/' | '-') && class == Some(Lang::En) {
                self.code = true;
            }
            let out = if word == self.shown { Outcome::Pass } else { edit(&self.shown, &format!("{word}{ch}")) };
            self.undo = alt.map(|a| (word.chars().count() + 1, a));
            self.context = class;
            self.clear_word();
            out
        };
        if space {
            self.code = false;
            self.cluster_empty = true;
            self.after_code_sep = false;
            if matches!(ch, '\n' | '\r') {
                self.context = None;
            }
        } else {
            self.cluster_empty = false;
            self.after_code_sep = matches!(ch, '.' | ':' | '\\');
        }
        out
    }

    /// Right after a commit of a word with two readings (rule 4, or the repeated-key rule
    /// deciding at commit), the first backspace swaps the reading and deletes the boundary
    /// char in one go: `this ⌫` -> `thí`. Otherwise one char is deleted and the raw keys
    /// are re-derived from what is left on screen.
    pub fn backspace(&mut self) -> Outcome {
        if let Some((delete, insert)) = self.undo.take() {
            self.clear_word();
            return Outcome::Replace(Edit { delete, insert });
        }
        if self.shown.is_empty() {
            self.reset();
            return Outcome::Pass;
        }
        self.shown.pop();
        let shown = self.shown.clone();
        let plain = self.raw || self.code;
        self.rebuild(if plain { shown.chars().collect() } else { derive_keys(&shown) });
        if !plain && self.composed != shown {
            self.rebuild(shown.chars().collect());
            self.raw = true;
        }
        Outcome::Pass
    }

    /// The cursor moved elsewhere: forget everything.
    pub fn reset(&mut self) {
        *self = Engine::new(self.config, std::mem::take(&mut self.lexicon));
    }

    fn clear_word(&mut self) {
        self.keys.clear();
        self.cancel.clear();
        self.composed.clear();
        self.shown.clear();
        self.raw = false;
    }

    fn rebuild(&mut self, keys: Vec<char>) {
        let shown = std::mem::take(&mut self.shown);
        let raw = self.raw;
        self.clear_word();
        for k in keys {
            self.push(k);
        }
        self.shown = shown;
        self.raw = raw;
    }

    fn push(&mut self, ch: char) {
        let prev = std::mem::take(&mut self.composed);
        let repeated = self.keys.last().is_some_and(|l| l.eq_ignore_ascii_case(&ch));
        self.keys.push(ch);
        self.composed = compose(&self.keys, self.config.tone_style);
        let cancelled =
            repeated && self.composed.chars().count() == prev.chars().count() + 1 && self.composed.ends_with(ch) && self.composed != format!("{prev}{ch}");
        self.cancel.push(cancelled);
    }

    fn double(&self) -> String {
        self.keys.iter().collect()
    }

    /// Keys without the second press of each cancelling repeat (`verr` -> `ver`).
    fn single(&self) -> String {
        self.keys.iter().zip(&self.cancel).filter(|(_, c)| !**c).map(|(k, _)| k).collect()
    }

    fn is_en(&self, w: &str) -> bool {
        self.lexicon.is_english(&w.to_lowercase())
    }

    /// Raw-key form while typing: a cancelled repeat shows once unless the full keys are English.
    fn raw_form(&self) -> String {
        let double = self.double();
        if self.cancel.contains(&true) && !self.is_en(&double) {
            self.single()
        } else {
            double
        }
    }

    fn typing_target(&mut self) -> String {
        if !self.config.auto_english {
            return self.composed.clone();
        }
        if is_mixed_case(&self.composed) {
            self.code = true;
        }
        if self.code {
            return self.double();
        }
        if !self.raw && !is_loose_prefix(&self.composed) {
            self.raw = true;
        }
        if self.raw {
            self.raw_form()
        } else {
            self.composed.clone()
        }
    }

    /// (final word, other reading for undo, clear language of the word).
    fn commit_word(&self) -> (String, Option<String>, Option<Lang>) {
        if !self.config.auto_english {
            return (self.composed.clone(), None, None);
        }
        if self.code {
            return (self.double(), None, None);
        }
        let c = self.composed.clone();
        let double = self.double();
        // A word that went raw stays raw to its end (spec 5.4), even if its keys compose.
        if !self.raw && is_syllable(&c) {
            let class = (!c.is_ascii()).then_some(Lang::Vi);
            if !self.is_en(&double) {
                return (c, None, class); // rule 3
            }
            // Rule 4: both a syllable and an English word.
            let lower = double.to_lowercase();
            let lang = self.lexicon.forced(&lower).or(self.context).unwrap_or_else(|| self.lexicon.tie_default(&lower));
            let raw = self.raw_form();
            let (word, alt) = match lang {
                Lang::Vi => (c, raw),
                Lang::En => (raw, c),
            };
            let class = (!word.is_ascii()).then_some(Lang::Vi);
            let alt = (alt != word).then_some(alt);
            return (word, alt, class);
        }
        // Rule 2, with the repeated-key rule deciding by the English list both ways.
        let (word, alt) = if self.cancel.contains(&true) {
            let single = self.single();
            match (self.is_en(&single), self.is_en(&double)) {
                (false, true) => (double, Some(single)),
                (true, false) => (single, Some(double)),
                _ => (self.raw_form(), None),
            }
        } else {
            (double, None)
        };
        (word, alt, Some(Lang::En))
    }
}

fn edit(shown: &str, target: &str) -> Outcome {
    let common = shown.chars().zip(target.chars()).take_while(|(a, b)| a == b).count();
    Outcome::Replace(Edit { delete: shown.chars().count() - common, insert: target.chars().skip(common).collect() })
}

/// Base vowel then its sắc, huyền, hỏi, ngã, nặng forms.
const TONED: &[&str] = &["aáàảãạ", "ăắằẳẵặ", "âấầẩẫậ", "eéèẻẽẹ", "êếềểễệ", "iíìỉĩị", "oóòỏõọ", "ôốồổỗộ", "ơớờởỡợ", "uúùủũụ", "ưứừửữự", "yýỳỷỹỵ"];
const TONE_KEYS: [char; 5] = ['s', 'f', 'r', 'x', 'j'];

/// Telex keys that compose back to `word` (tone key last): `tiế` -> `tiees`.
pub fn derive_keys(word: &str) -> Vec<char> {
    let case = |k: char, upper: bool| if upper { k.to_ascii_uppercase() } else { k };
    let mut keys = Vec::new();
    let mut tone = None;
    for c in word.chars() {
        let upper = c.is_uppercase();
        let lower = c.to_lowercase().next().unwrap_or(c);
        let (base, t) = TONED.iter().find_map(|row| row.chars().position(|x| x == lower).map(|i| (row.chars().next().unwrap(), i))).unwrap_or((lower, 0));
        if t > 0 {
            tone = Some(case(TONE_KEYS[t - 1], upper));
        }
        let (k1, k2) = match base {
            'â' => ('a', Some('a')),
            'ă' => ('a', Some('w')),
            'ê' => ('e', Some('e')),
            'ô' => ('o', Some('o')),
            'ơ' => ('o', Some('w')),
            'ư' => ('u', Some('w')),
            'đ' => ('d', Some('d')),
            b => (b, None),
        };
        keys.push(case(k1, upper));
        keys.extend(k2.map(|k| case(k, upper)));
    }
    keys.extend(tone);
    keys
}
