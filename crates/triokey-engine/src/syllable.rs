//! Vietnamese syllable set from phonotactic rules (initial x rhyme x tone), not a word list.

use std::collections::HashMap;
use std::sync::OnceLock;

const INITIALS: &[&str] =
    &["", "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x"];

// Toneless rhymes (medial + nucleus + final). `oăp` left out on purpose: no real word (`goặp`).
const RHYMES: &[&str] = &[
    "a", "ac", "ach", "ai", "am", "an", "ang", "anh", "ao", "ap", "at", "au", "ay", "ăc", "ăm", "ăn", "ăng", "ăp", "ăt", "âc", "âm", "ân", "âng", "âp", "ât",
    "âu", "ây", "e", "ec", "em", "en", "eng", "eo", "ep", "et", "ê", "êch", "êm", "ên", "ênh", "êp", "êt", "êu", "i", "ia", "ich", "im", "in", "inh", "ip",
    "it", "iu", "iêc", "iêm", "iên", "iêng", "iêp", "iêt", "iêu", "o", "oc", "oi", "om", "on", "ong", "op", "ot", "ooc", "oong", "oa", "oac", "oach", "oai",
    "oam", "oan", "oang", "oanh", "oao", "oap", "oat", "oay", "oăc", "oăm", "oăn", "oăng", "oăt", "oe", "oen", "oeo", "oet", "ô", "ôc", "ôi", "ôm", "ôn",
    "ông", "ôp", "ôt", "ơ", "ơi", "ơm", "ơn", "ơp", "ơt", "u", "ua", "uc", "ui", "um", "un", "ung", "up", "ut", "uân", "uâng", "uât", "uây", "uê", "uêch",
    "uênh", "uơ", "uôc", "uôi", "uôm", "uôn", "uông", "uôt", "uy", "uya", "uych", "uynh", "uyp", "uyt", "uyu", "uyên", "uyêt", "ư", "ưa", "ưc", "ưi", "ưm",
    "ưn", "ưng", "ưt", "ưu", "ươc", "ươi", "ươm", "ươn", "ương", "ươp", "ươt", "ươu", "y", "yêm", "yên", "yêt", "yêu", "ych", "ynh", "yp", "yt",
];

// Base vowel followed by its sắc, huyền, hỏi, ngã, nặng forms (tone 1..=5; 0 = none).
const TONED: &[&str] = &["aáàảãạ", "ăắằẳẵặ", "âấầẩẫậ", "eéèẻẽẹ", "êếềểễệ", "iíìỉĩị", "oóòỏõọ", "ôốồổỗộ", "ơớờởỡợ", "uúùủũụ", "ưứừửữự", "yýỳỷỹỵ"];

/// Splits off the tone: (toneless lowercase, tone 0..=5); None if two tone marks.
fn strip_tone(word: &str) -> Option<(String, u8)> {
    let mut tone = 0;
    let mut out = String::with_capacity(word.len());
    for c in word.chars().flat_map(char::to_lowercase) {
        let hit = TONED.iter().find_map(|row| row.chars().position(|x| x == c).map(|i| (row.chars().next().unwrap(), i as u8)));
        match hit {
            Some((base, t)) if t > 0 => {
                if tone != 0 {
                    return None;
                }
                tone = t;
                out.push(base);
            }
            _ => out.push(c),
        }
    }
    Some((out, tone))
}

/// Spelling rules for c/k/q/g/gh/ng/ngh.
fn spelling_ok(initial: &str, rhyme: &str) -> bool {
    let first = rhyme.chars().next().unwrap();
    let front = matches!(first, 'i' | 'e' | 'ê' | 'y');
    // `y` + final (quýt, quỳnh) exists only after the `qu` glide; elsewhere it is `uy…`.
    if first == 'y' && rhyme.len() > 1 && !rhyme.starts_with("yê") {
        return initial == "qu";
    }
    match initial {
        // `ka` is in the plan's must-accept list (loanword spelling); only bare `ka`, so kan/kao stay out.
        "k" => front || rhyme == "a",
        "gh" | "ngh" => matches!(first, 'i' | 'e' | 'ê'),
        "c" | "ng" => !front,
        "g" => !matches!(first, 'e' | 'ê' | 'y'), // `g` + `i…` is the `gi` spelling
        "gi" => !matches!(first, 'i' | 'y'),
        "qu" => !matches!(first, 'o' | 'u' | 'ư'),
        _ => true,
    }
}

const ALL_TONES: u8 = 0b11_1111;
const STOP_TONES: u8 = 1 << 1 | 1 << 5; // sắc, nặng

/// toneless string -> (tones allowed as a prefix, tones allowed as a full syllable).
fn table() -> &'static HashMap<String, (u8, u8)> {
    static T: OnceLock<HashMap<String, (u8, u8)>> = OnceLock::new();
    T.get_or_init(|| {
        let mut m: HashMap<String, (u8, u8)> = HashMap::new();
        for ini in INITIALS {
            for rh in RHYMES.iter().filter(|rh| spelling_ok(ini, rh)) {
                let full = format!("{ini}{rh}");
                let stop = ["c", "ch", "p", "t"].iter().any(|f| full.ends_with(f));
                let tones = if stop { STOP_TONES } else { ALL_TONES };
                m.entry(full.clone()).or_default().1 |= tones;
                for (i, _) in full.char_indices().chain([(full.len(), ' ')]) {
                    // Prefixes may lack the tone for now, so bit 0 is always allowed.
                    m.entry(full[..i].to_string()).or_default().0 |= tones | 1;
                }
            }
        }
        m
    })
}

fn check(word: &str, pick: fn(&(u8, u8)) -> u8) -> bool {
    let Some((base, tone)) = strip_tone(word) else { return false };
    table().get(&base).is_some_and(|e| pick(e) & (1 << tone) != 0)
}

/// Whole word (with diacritics, any case) is a valid Vietnamese syllable.
pub fn is_syllable(word: &str) -> bool {
    check(word, |e| e.1)
}

/// Word can still grow into a valid syllable (used while typing).
pub fn is_prefix(word: &str) -> bool {
    check(word, |e| e.0)
}

fn fold(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'â' | 'ă' => 'a',
            'ê' => 'e',
            'ô' | 'ơ' => 'o',
            'ư' => 'u',
            'đ' => 'd',
            c => c,
        })
        .collect()
}

/// Like `is_prefix`, but vowel marks and tone may still be added by later Telex keys
/// (`nguoi` can still become `người`).
pub(crate) fn is_loose_prefix(word: &str) -> bool {
    static F: OnceLock<std::collections::HashSet<String>> = OnceLock::new();
    let set = F.get_or_init(|| table().keys().map(|k| fold(k)).collect());
    strip_tone(word).is_some_and(|(base, _)| set.contains(&fold(&base)))
}
