//! Typing engine: recomputes the whole word from its raw keys after every key
//! (docs/ARCHITECTURE.md, 5. Engine).

use crate::lexicon::{is_abbr, Lexicon};
use crate::syllable::{is_loose_prefix, is_syllable, marks};
use crate::telex::compose;
use crate::token::{is_mixed_case, starts_code};
use crate::{Lang, ToneStyle};

// Scoring (see `commit_word`); rates are uses per million words. Tuned on the dev split of
// tests/realtext.rs.
/// Line mode at the start of a line, its decay per word, and the most one word moves it.
const PRIOR: f32 = 1.0;
const DECAY: f32 = 0.6;
const CAP: f32 = 3.0;
/// Smoothing of the per-word log-odds.
const EPS: f32 = 0.5;
/// Rate of a valid syllable missing from the table: one mark, two or more marks.
const SYL_FLOOR: f32 = 0.1;
const MARKED_FLOOR: f32 = 0.2;
/// Rate of raw keys that are no known word.
const RAW_FLOOR: f32 = 0.05;
/// An ALL CAPS word composes only when the Vietnamese word is this common.
const CAPS_MIN: f32 = 50.0;
/// The losing reading is the backspace alternative only when the winner is at most
/// this many times likelier. A near-certain word leaves Backspace alone.
const ARM_RATIO: f32 = 100.0;
/// Raw keys this common in English are always the backspace alternative to a Vietnamese win.
const ARM_EN: f32 = 10.0;

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

/// The backspace swap right after a commit: delete `delete` chars, insert `insert`, set the
/// line mode to `mode`. Kept (a space, Tab or Enter comes next), `keys` remember `insert`, or
/// are forgotten when `forget` (the swap undid a remembered word back to a fresh line's pick).
#[derive(Clone, Debug)]
struct Undo {
    delete: usize,
    insert: String,
    mode: f32,
    keys: String,
    forget: bool,
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
    /// Shown as raw keys until it ends: it can no longer become Vietnamese.
    raw: bool,
    /// Keys were rebuilt from the screen after a backspace: commit them as they are.
    literal: bool,
    // Current cluster (text between whitespace).
    code: bool,
    cluster_empty: bool,
    after_code_sep: bool,
    /// Line mode: log-odds that this line is Vietnamese (maybe with English terms), not English.
    mode: f32,
    /// Previous word on this line: None at its start, else whether it was ALL CAPS Vietnamese.
    prev: Option<bool>,
    undo: Option<Undo>,
    /// The swap the last backspace made; remembered if the next event is a space, Tab or Enter.
    swapped: Option<Undo>,
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
            literal: false,
            code: false,
            cluster_empty: true,
            after_code_sep: false,
            mode: PRIOR,
            prev: None,
            undo: None,
            swapped: None,
        }
    }

    /// A letter `a-z`/`A-Z`. Anything else just drops the word being typed.
    pub fn key(&mut self, ch: char) -> Outcome {
        self.undo = None;
        if !ch.is_ascii_alphabetic() {
            self.reset();
            return Outcome::Pass;
        }
        // glued to the swapped word: the person only wanted the space gone (`test ⌫s`)
        self.swapped = None;
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
        // punctuation glued to the swapped word (`test ⌫.`) keeps nothing either
        if space {
            self.keep_swap();
        } else {
            self.swapped = None;
        }
        if self.config.auto_english && !space && ((self.cluster_empty && starts_code(ch)) || ch.is_ascii_digit() || ch == '_' || self.after_code_sep) {
            self.code = true;
        }
        let out = if self.keys.is_empty() {
            Outcome::Pass
        } else {
            let (word, alt, forget) = self.commit_word();
            let out = if word == self.shown { Outcome::Pass } else { edit(&self.shown, &format!("{word}{ch}")) };
            let before = self.mode;
            let keys = self.double().to_lowercase();
            self.undo = alt.map(|a| Undo { delete: word.chars().count() + 1, mode: self.next_mode(before, &a), insert: a, keys, forget });
            self.mode = self.next_mode(before, &word);
            self.prev = Some(!word.chars().any(char::is_lowercase) && self.lean(&word) > 0.0);
            // English, not a syllable: `src/main`, `foo-bar` turn to code; `bật/tắt`, `ca-nô` do not.
            if self.config.auto_english && matches!(ch, '/' | '-') && word.is_ascii() && !is_syllable(&word) {
                self.code = true;
            }
            self.clear_word();
            out
        };
        if space {
            self.code = false;
            self.cluster_empty = true;
            self.after_code_sep = false;
            if matches!(ch, '\n' | '\r') {
                self.mode = PRIOR;
                self.prev = None;
            }
        } else {
            self.cluster_empty = false;
            self.after_code_sep = matches!(ch, '.' | ':' | '\\');
        }
        out
    }

    /// Right after a commit of a word with two close readings, the first backspace swaps
    /// the reading and deletes the boundary char in one go: `this ⌫` -> `thí`. Otherwise one
    /// char is deleted and the raw keys are re-derived from what is left on screen.
    pub fn backspace(&mut self) -> Outcome {
        self.swapped = None;
        if let Some(u) = self.undo.take() {
            self.clear_word();
            self.mode = u.mode;
            let out = Outcome::Replace(Edit { delete: u.delete, insert: u.insert.clone() });
            self.swapped = Some(u);
            return out;
        }
        if self.shown.is_empty() {
            // into the previous word: still the same line, so its mode stays
            let (mode, prev) = (self.mode, self.prev);
            self.reset();
            (self.mode, self.prev) = (mode, prev);
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
        self.literal = self.raw;
        Outcome::Pass
    }

    /// The cursor moved elsewhere: forget everything.
    pub fn reset(&mut self) {
        *self = Engine::new(self.config, std::mem::take(&mut self.lexicon));
    }

    /// A space, Tab or Enter right after a backspace swap: remember it (or forget the entry it
    /// undid). Only here, never in `backspace`, so a probe that clones the engine and swaps
    /// learns nothing. Forced words are not stored: typing could never clear such an entry.
    fn keep_swap(&mut self) {
        if let Some(u) = self.swapped.take().filter(|u| self.lexicon.forced(&u.keys).is_none()) {
            self.lexicon.learned().set(&u.keys, (!u.forget).then_some(u.insert.as_str()));
        }
    }

    fn clear_word(&mut self) {
        self.keys.clear();
        self.cancel.clear();
        self.composed.clear();
        self.shown.clear();
        self.raw = false;
        self.literal = false;
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

    /// Probability that this line is Vietnamese.
    fn p_vi(&self) -> f32 {
        1.0 / (1.0 + (-self.mode).exp())
    }

    /// How often the raw keys `w` are used as a word on a line that is Vietnamese with probability `p`.
    fn raw_rate(&self, w: &str, p: f32) -> f32 {
        let (vi, en) = self.lexicon.rates(w);
        p * vi + (1.0 - p) * en
    }

    /// In any table.
    fn known(&self, w: &str) -> bool {
        let (vi, en) = self.lexicon.rates(w);
        vi > 0.0 || en > 0.0
    }

    /// Log-odds evidence that the committed `word` is Vietnamese.
    fn lean(&self, word: &str) -> f32 {
        let (vi, en) = self.lexicon.rates(word);
        ((vi + EPS) / (en + EPS)).ln().clamp(-CAP, CAP)
    }

    /// Line mode after committing `word`: code words and single letters say nothing.
    fn next_mode(&self, mode: f32, word: &str) -> f32 {
        if self.code || word.chars().count() < 2 {
            mode
        } else {
            DECAY * mode + self.lean(word)
        }
    }

    /// The raw reading when a repeat cancelled a mark, and the other one for undo.
    /// Only one known: that one. Both: as typed (`off`; plain `of` already gives `of`), the
    /// other armed if it is close. Neither: the single form only if it `fits` as Vietnamese
    /// (the repeat escaped a mark: `ass` -> `as`), else as typed (`passwd`, `Terraform`).
    fn raw_pick(&self, fits: fn(&str) -> bool) -> (String, Option<String>) {
        let double = self.double();
        if !self.cancel.contains(&true) || self.lexicon.forced(&double.to_lowercase()) == Some(Lang::En) {
            return (double, None);
        }
        let single = self.single();
        match (self.known(&double), self.known(&single)) {
            (true, true) => {
                let p = self.p_vi();
                pick((double.clone(), self.raw_rate(&double, p)), (single.clone(), self.raw_rate(&single, p)))
            }
            (true, false) => (double, None),
            (false, true) => (single, None),
            (false, false) => {
                let keys: Vec<char> = single.chars().collect();
                (if fits(&compose(&keys, self.config.tone_style)) { single } else { double }, None)
            }
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
        if !self.raw && !is_loose_prefix(&self.composed) && !self.lexicon.is_abbr_prefix(&self.composed) {
            self.raw = true;
        }
        if self.raw {
            self.raw_pick(is_loose_prefix).0
        } else {
            self.composed.clone()
        }
    }

    /// (final word, other reading for the backspace swap, swapping and keeping it forgets the
    /// remembered entry): the force files, then a remembered entry, then `scored`. An entry
    /// applies only when it names one of the readings in play, written with the keys' case,
    /// and never replaces a Vietnamese pick once the line leans more Vietnamese than a fresh
    /// one: there it is only the backspace swap, so mid-line no Vietnamese word is lost to it.
    fn commit_word(&mut self) -> (String, Option<String>, bool) {
        let (word, alt) = self.scored();
        let keys = self.double().to_lowercase();
        if !self.config.auto_english || self.code || self.literal || self.lexicon.forced(&keys).is_some() {
            return (word, alt, false);
        }
        let Some(text) = self.lexicon.learned().get(&keys) else { return (word, alt, false) };
        let readings = [self.double(), self.single(), self.composed.clone()];
        let Some(learned) = readings.into_iter().find(|r| r.to_lowercase() == text && *r != word) else { return (word, alt, false) };
        if word == self.composed && self.mode > PRIOR {
            return (word, Some(learned), false);
        }
        let (mode, prev) = (self.mode, self.prev);
        (self.mode, self.prev) = (PRIOR, None);
        let fresh = self.scored().0;
        (self.mode, self.prev) = (mode, prev);
        let forget = fresh == word;
        (learned, Some(word), forget)
    }

    /// (final word, other reading for the backspace swap): the raw reading `r` (`raw_pick`)
    /// against the composed `v`, each rated by how often it is used on this line.
    fn scored(&self) -> (String, Option<String>) {
        if !self.config.auto_english {
            return (self.composed.clone(), None);
        }
        if self.code {
            return (self.double(), None);
        }
        let (r, r_alt) = self.raw_pick(is_syllable);
        let v = &self.composed;
        // Keys rebuilt after a backspace stay as they are when they are a word (`boss⌫` -> `bos`).
        if (self.literal && self.known(&r)) || *v == r {
            return (r, r_alt);
        }
        if v.is_ascii() {
            // `z` removed the tone (`toansz` -> `toan`): the syllable, unless the keys are a word
            return if is_syllable(v) && !self.known(&r) { (v.clone(), None) } else { (r, r_alt) };
        }
        // a syllable, or an abbreviation (`đc`) common enough not to be a typo
        let vi_v = self.lexicon.rates(v).0;
        let rate_v = if is_syllable(v) {
            vi_v.max(if marks(v) >= 2 { MARKED_FLOOR } else { SYL_FLOOR })
        } else if is_abbr(v, vi_v) {
            vi_v
        } else {
            return (r, r_alt);
        };
        let p = self.p_vi();
        let rate_r = match self.raw_rate(&r, p) {
            0.0 => RAW_FLOOR,
            x => x,
        };
        let score_v = p * rate_v;
        // ALL CAPS: `OS`, `USA` stay; `TÔI` composes, and a word right after an ALL CAPS Vietnamese one is scored normally.
        // A one-letter word is typed `As`, not `AS`, so two capitals are an acronym wherever they stand.
        let caps = self.keys.iter().all(char::is_ascii_uppercase) && self.prev != Some(true);
        let (vi, raw) = ((v.clone(), score_v), (r, rate_r));
        match self.lexicon.forced(&self.double().to_lowercase()) {
            Some(Lang::Vi) => pick(vi, (raw.0, f32::MAX)), // forced: Backspace still offers the other
            Some(Lang::En) => pick(raw, (vi.0, f32::MAX)),
            None if caps && (self.known(&raw.0) || vi_v < CAPS_MIN) => (raw.0, Some(vi.0)),
            None if vi.1 > raw.1 => {
                // a real English word stays one Backspace away however Vietnamese the line is
                let en = self.lexicon.rates(&raw.0).1 >= ARM_EN;
                let (word, alt) = pick(vi, raw.clone());
                (word, alt.or(en.then_some(raw.0)))
            }
            None => {
                let (word, alt) = pick(raw, vi);
                (word, alt.or(r_alt))
            }
        }
    }
}

/// `win` against `lose` (word, rate): the winner, and the loser if it is close enough to arm
/// the backspace swap (`ARM_RATIO`).
fn pick(win: (String, f32), lose: (String, f32)) -> (String, Option<String>) {
    let armed = lose.1 * ARM_RATIO >= win.1;
    (win.0, armed.then_some(lose.0))
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
