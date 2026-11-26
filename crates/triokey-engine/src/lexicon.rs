use crate::syllable::{is_syllable, tone_free};
use crate::Lang;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

const FREQ_VI: &str = include_str!("../data/freq_vi.tsv");
const FREQ_EN: &str = include_str!("../data/freq_en.tsv");
const EN_WORDS: &str = include_str!("../data/en_words.txt");
const EN_TECH: &str = include_str!("../data/en_tech.txt");

/// Rate floors (per million): dictionary words are at least rare English; technical words
/// sit inside Vietnamese sentences more than web text shows.
const EN_WORD_EN: f32 = 0.5;
const TECH_EN: f32 = 5.0;
const TECH_VI: f32 = 5.0;
/// A composed non-syllable (`đc`) this common is an abbreviation; typos sit below.
pub(crate) const ABBR_MIN: f32 = 3.0;

/// `word` (composed, used `rate` per million) is an abbreviation: two or more letters with a
/// consonant (`đc`), no syllable. Lone letters and vowel fragments of broken text
/// (`â`, `ươ`, `iệ`) are not.
pub(crate) fn is_abbr(word: &str, rate: f32) -> bool {
    let consonant = |c: char| matches!(c, 'đ' | 'Đ') || (c.is_ascii_alphabetic() && !"aeiouyAEIOUY".contains(c));
    rate >= ABBR_MIN && word.chars().count() > 1 && word.chars().any(consonant) && !is_syllable(word)
}

/// Lines with everything from `#` on dropped, trimmed, blanks skipped.
fn entries(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(|l| l.split('#').next().unwrap_or("").trim()).filter(|l| !l.is_empty())
}

/// `token<TAB>count` lines under a `# total: N` header, as (token, rate per million).
fn per_million(text: &str) -> impl Iterator<Item = (&str, f32)> {
    let total: f32 = text.lines().find_map(|l| l.strip_prefix("# total:")).and_then(|n| n.trim().parse().ok()).expect("frequency table without # total");
    entries(text).filter_map(move |l| {
        let (t, n) = l.split_once('\t')?;
        Some((t, n.trim().parse::<f32>().ok()? * 1e6 / total))
    })
}

struct Tables {
    /// tone-free key -> (Vietnamese, English) rate per million, floors applied.
    rates: HashMap<String, (f32, f32)>,
    /// Every prefix of a common non-syllable with `đ` (`đ` and `đc` of `đc`).
    abbr_prefixes: HashSet<String>,
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut rates: HashMap<String, (f32, f32)> = HashMap::new();
        // `hòa` and `hoà` share a key, so their counts add up.
        for (w, r) in per_million(FREQ_VI) {
            rates.entry(tone_free(w)).or_default().0 += r;
        }
        for (w, r) in per_million(FREQ_EN) {
            rates.entry(tone_free(w)).or_default().1 += r;
        }
        let mut floor = |w: &str, vi: f32, en: f32| {
            let e = rates.entry(tone_free(w)).or_default();
            *e = (e.0.max(vi), e.1.max(en));
        };
        entries(EN_WORDS).for_each(|w| floor(w, 0.0, EN_WORD_EN));
        entries(EN_TECH).for_each(|w| floor(w, TECH_VI, TECH_EN));
        let abbr_prefixes = per_million(FREQ_VI)
            .filter(|&(w, r)| w.contains('đ') && is_abbr(w, r))
            .map(|(w, _)| w)
            .flat_map(|w| w.char_indices().map(|(i, c)| w[..i + c.len_utf8()].to_string()).collect::<Vec<_>>())
            .collect();
        Tables { rates, abbr_prefixes }
    })
}

/// Word rates (web frequencies, SCOWL list, technical list) shared by every
/// engine, plus the user's two force files. Lookups take any case.
#[derive(Clone, Debug, Default)]
pub struct Lexicon {
    forced: HashMap<String, Lang>,
    learned: Learned,
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

    /// (Vietnamese, English) uses per million words; `hòa` and `hoà` are one word.
    /// The first call parses the tables.
    pub fn rates(&self, word: &str) -> (f32, f32) {
        tables().rates.get(&tone_free(word)).copied().unwrap_or_default()
    }

    /// `word` (composed, any case) can still grow into a common abbreviation with `đ`.
    pub fn is_abbr_prefix(&self, word: &str) -> bool {
        tables().abbr_prefixes.contains(&word.to_lowercase())
    }

    pub fn forced(&self, keys_lower: &str) -> Option<Lang> {
        self.forced.get(keys_lower).copied()
    }

    /// Share `learned` with every engine built from this lexicon (and its clones).
    pub fn with_learned(mut self, learned: Learned) -> Lexicon {
        self.learned = learned;
        self
    }

    pub fn learned(&self) -> &Learned {
        &self.learned
    }
}

/// Most entries kept, and the longest keys and text (in chars) of one entry.
pub const LEARNED_MAX: usize = 5000;
const LEARNED_LEN: usize = 32;
const LEARNED_HEADER: &str = "# TrioKey nhớ các từ bạn đã đổi bằng Backspace rồi giữ nguyên: phím gõ<TAB>chữ hiện ra. Chỉ lưu trên máy này.
# Xoá một dòng để TrioKey quên từ đó (sửa khi TrioKey đã Thoát, nếu không nó sẽ ghi đè).
";

/// Words the person swapped with Backspace and kept: typed keys -> on-screen text, both
/// lowercase. One map shared by every clone (the hook's engine, the saver thread); the lock
/// is held only for a lookup, an insert or a render.
#[derive(Clone, Debug, Default)]
pub struct Learned(Arc<Mutex<(BTreeMap<String, String>, u64)>>);

fn learnable(keys: &str, text: &str) -> bool {
    let fits = |s: &str| (1..=LEARNED_LEN).contains(&s.chars().count());
    fits(keys) && fits(text) && keys.chars().all(|c| c.is_ascii_alphabetic()) && text.chars().all(char::is_alphabetic)
}

impl Learned {
    /// `keys<TAB>text` lines; `#` lines, anything else malformed and entries past
    /// `LEARNED_MAX` are skipped (the file is user-writable and read by an elevated copy).
    pub fn parse(text: &str) -> Learned {
        let mut map = BTreeMap::new();
        for line in text.strip_prefix('\u{FEFF}').unwrap_or(text).lines().filter(|l| !l.trim_start().starts_with('#')) {
            let mut t = line.split_whitespace();
            if let (Some(k), Some(v), None) = (t.next(), t.next(), t.next()) {
                let (k, v) = (k.to_lowercase(), v.to_lowercase());
                if learnable(&k, &v) && map.len() < LEARNED_MAX {
                    map.insert(k, v);
                }
            }
        }
        Learned(Arc::new(Mutex::new((map, 0))))
    }

    /// The file contents: header, then entries sorted by keys.
    pub fn render(&self) -> String {
        let g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        g.0.iter().fold(LEARNED_HEADER.to_string(), |s, (k, v)| s + k + "\t" + v + "\n")
    }

    /// Moves on every change, for a saver on another thread.
    pub fn changes(&self) -> u64 {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).1
    }

    pub fn get(&self, keys_lower: &str) -> Option<String> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).0.get(keys_lower).cloned()
    }

    /// Remember `text` for `keys` (lowercased), or forget them with None. Entries the file
    /// format would refuse are not kept, and a full map takes no new keys.
    pub fn set(&self, keys: &str, text: Option<&str>) {
        let (keys, text) = (keys.to_lowercase(), text.map(str::to_lowercase));
        let mut g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let changed = match text {
            Some(t) if learnable(&keys, &t) && (g.0.len() < LEARNED_MAX || g.0.contains_key(&keys)) => g.0.insert(keys, t.clone()) != Some(t),
            Some(_) => false,
            None => g.0.remove(&keys).is_some(),
        };
        g.1 += changed as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_million_reads_the_total() {
        let t: Vec<_> = per_million("# total: 2000000\nthe\t10\n# c\nbad line\nof\t1\n").collect();
        assert_eq!(t, [("the", 5.0), ("of", 0.5)]);
    }

    #[test]
    fn floors_and_tone_position() {
        let lx = Lexicon::builtin();
        assert_eq!(lx.rates("hòa"), lx.rates("HOÀ"));
        assert!(lx.rates("hòa").0 > 470.0); // hòa 425 + hoà 52 per million
        assert!(lx.rates("hòa").0 > 100.0 && lx.rates("this").1 > 1000.0);
        let (vi, en) = lx.rates("diff"); // en_tech
        assert!(vi >= TECH_VI && en >= TECH_EN);
        assert!(lx.rates("đc").0 >= ABBR_MIN);
        assert!(lx.rates("aardvark").1 >= EN_WORD_EN);
        assert_eq!(lx.rates("tooi"), (0.0, 0.0));
        assert!(lx.is_abbr_prefix("Đ") && lx.is_abbr_prefix("đc") && !lx.is_abbr_prefix("dc"));
    }

    #[test]
    fn learned_file_round_trips_and_caps() {
        let l = Learned::parse("# c\nThis\tthí\nsongs\tsóng\n\nbad line here\nx1\ty\nkeys\tte-xt\n");
        assert_eq!(l.get("this").as_deref(), Some("thí"));
        assert_eq!(l.render(), format!("{LEARNED_HEADER}songs\tsóng\nthis\tthí\n"));
        assert_eq!(Learned::parse(&l.render()).render(), l.render());
        let long = "a".repeat(33);
        assert!(Learned::parse(&format!("{long}\tb\nb\t{long}\n")).get("b").is_none());
        // a BOM before the first entry (header deleted, saved by Notepad) is not part of the key
        assert_eq!(Learned::parse("\u{FEFF}this\tthí\r\nfor\tfỏ\r\n").get("this").as_deref(), Some("thí"));
        // distinct letter keys: i in base 26
        let key = |mut i: usize| {
            let mut s = String::from("k");
            loop {
                s.push((b'a' + (i % 26) as u8) as char);
                i /= 26;
                if i == 0 {
                    break s;
                }
            }
        };
        let many = Learned::parse(&(0..LEARNED_MAX + 10).map(|i| format!("{}\tv\n", key(i))).collect::<String>());
        assert_eq!(many.render().lines().count(), 2 + LEARNED_MAX);
        many.set("zzzzzz", Some("v"));
        assert!(many.get("zzzzzz").is_none() && many.changes() == 0);
        // changes move only when the map changes
        l.set("this", Some("THÍ"));
        assert_eq!(l.changes(), 0);
        l.set("this", None);
        l.set("for", Some("fỏ"));
        l.set("bad", Some("a b"));
        assert_eq!((l.changes(), l.get("this"), l.get("bad")), (2, None, None));
    }
}
