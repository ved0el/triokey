//! Regenerates `data/en_words.txt` and `data/ambiguous_defaults.txt` from SCOWL.
//! See docs/decisions/0002-lexicon.md and 0005-tie-defaults.md. Not part of the normal test run:
//!   TRIOKEY_SCOWL=<path to scowl-en.txt> TRIOKEY_VIELOTUS_LEXICON=<VieLotus lexicon dir> \
//!     mise x -- cargo test -p triokey-engine --test gen_lexicon -- --ignored

use std::collections::HashSet;

use triokey_engine::syllable::is_syllable;
use triokey_engine::telex::compose;
use triokey_engine::{derive_keys, ToneStyle};

/// Hand review 2026-11-25 of all 138 level-0 rows: flipped to `vi` only where the Vietnamese
/// word is everyday, typed this exact standard way, and the English word is rare/odd
/// (`cums`, `hes`) or far less frequent than the Vietnamese one (`sex` = `sẽ`).
const VI_OVERRIDES: &[&str] = &["cums", "hes", "sex"];

/// VieLotus lexicon files whose words are added to the English list (see 0005).
const VIELOTUS_EN: &[&str] = &["04_unconditional_english_words.txt", "05_context_english_collision_words.txt"];

/// Level given to words that come from VieLotus, not SCOWL: never the level-0 rule.
const EXTRA_LEVEL: u8 = 9;

fn set(text: &str) -> HashSet<&str> {
    text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect()
}

/// Keys a Vietnamese typist would use for `vi`: its standard Telex spelling (`derive_keys`,
/// also with `w` for `uw` and `uow` for `ươ`), with the tone key, if any, typed anywhere in the
/// word. `songs`, `hast` qualify; `cars` (two tone keys), `photos` (`o..o` split by a
/// consonant) and `theme` do not.
fn natural(keys: &str, vi: &str) -> bool {
    let mut d: String = derive_keys(vi).into_iter().collect();
    let tone = d.ends_with(['s', 'f', 'r', 'x', 'j']);
    let tone = tone.then(|| d.pop().unwrap());
    let bases = [d.clone(), d.replace("uwow", "uow"), d.replace("uw", "w")];
    let fits = |k: &str| bases.iter().any(|b| b == k);
    match tone {
        None => fits(keys),
        Some(t) => keys.char_indices().filter(|&(_, c)| c == t).any(|(i, _)| fits(&format!("{}{}", &keys[..i], &keys[i + 1..]))),
    }
}

/// Coda-less reading that is a lone vowel (`ox` = `õ`) or carries a vowel mark (`cows` = `cớ`,
/// `bees` = `bế`): basic Telex drills, and the English side is a short word.
fn open_marked(vi: &str) -> bool {
    let d: String = derive_keys(vi).into_iter().filter(|c| !"sfrxj".contains(*c)).collect();
    let open = d.ends_with(['a', 'e', 'i', 'o', 'u', 'y', 'w']);
    let mark = ["aa", "ee", "oo", "aw", "ow", "uw"].iter().any(|m| d.contains(m));
    open && (mark || vi.chars().count() == 1)
}

#[test]
fn natural_keys() {
    for v in ["õ", "cớ", "bế", "lớ", "bề"] {
        assert!(open_marked(v), "{v}");
    }
    for v in ["tóe", "bốt", "hẻo", "má"] {
        assert!(!open_marked(v), "{v}");
    }
    for (k, v) in
        [("songs", "sóng"), ("sosng", "sóng"), ("tieesng", "tiếng"), ("hoaf", "hòa"), ("thuys", "thúy"), ("nhuw", "như"), ("nguowif", "người"), ("teen", "tên")]
    {
        assert!(natural(k, v), "{k}");
    }
    for (k, v) in [("cars", "cá"), ("photos", "phốt"), ("theme", "thêm"), ("defend", "đền"), ("barns", "bán")] {
        assert!(!natural(k, v), "{k}");
    }
}

#[test]
#[ignore]
fn generate() {
    let src = std::env::var("TRIOKEY_SCOWL").expect("set TRIOKEY_SCOWL to scowl-en.txt");
    let lex_dir = std::env::var("TRIOKEY_VIELOTUS_LEXICON").expect("set TRIOKEY_VIELOTUS_LEXICON");
    let text = std::fs::read_to_string(src).unwrap();
    let extra: Vec<String> = VIELOTUS_EN.iter().map(|f| std::fs::read_to_string(format!("{lex_dir}/{f}")).unwrap()).collect();
    let attested = set(include_str!("data/vi_attested.txt"));
    let common = set(include_str!("data/vi_common.txt"));
    for w in &common {
        assert!(attested.contains(w), "vi_common entry {w} is not an attested syllable");
    }
    let reading = |w: &str| compose(&w.chars().collect::<Vec<_>>(), ToneStyle::Old);

    // Line = one digit level (0 = most common) + word.
    let mut rows: Vec<(u8, &str)> = text
        .lines()
        .filter_map(|l| {
            let (lvl, w) = l.split_at_checked(1)?;
            let lvl = lvl.parse().ok()?;
            (!w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase())).then_some((lvl, w))
        })
        .collect();
    // VieLotus English words, minus any whose Telex reading is a real Vietnamese syllable
    // (`cors` = `có`, `uri` = `ủi`): adding those would turn them English after an English word.
    for w in extra.iter().flat_map(|t| set(t)) {
        if w.bytes().all(|b| b.is_ascii_lowercase()) && !attested.contains(reading(w).as_str()) {
            rows.push((EXTRA_LEVEL, w));
        }
    }
    rows.sort();
    rows.sort_by_key(|r| r.1);
    rows.dedup_by(|a, b| a.1 == b.1); // keeps the lowest level of each word
    rows.sort();

    let mut words: Vec<&str> = rows.iter().map(|r| r.1).collect();
    words.sort();
    let mut en = String::from(
        "# SCOWL via VieLotus-IM scowl-en.txt (see SCOWL-Copyright.txt), plus VieLotus-IM lexicon\n\
         # 04/05 words whose reading is not a Vietnamese syllable (MIT, see NOTICE.md)\n",
    );
    for w in words {
        en.push_str(w);
        en.push('\n');
    }

    let mut amb = String::from(
        "# word<TAB>en|vi: English words whose Telex reading is a different, valid Vietnamese\n\
         # syllable. Most common first. Generated by tests/gen_lexicon.rs, then hand-reviewed.\n",
    );
    for (lvl, w) in rows {
        let vi = reading(w);
        if vi == w || !is_syllable(&vi) {
            continue;
        }
        // Level 0 (SCOWL's most common tier, ~4k words) stands in for "top ~2000".
        // Other levels: English unless typed the usual Vietnamese way and the reading is a
        // common Vietnamese word or (level 2+, rare English) an open marked syllable.
        let nat = natural(w, &vi);
        let is_vi = if lvl == 0 { VI_OVERRIDES.contains(&w) } else { nat && (common.contains(vi.as_str()) || (lvl >= 2 && open_marked(&vi))) };
        // Review aid: `-- --nocapture` lists level, keys, reading, attested/natural flags.
        let att = attested.contains(vi.as_str());
        println!("REV {lvl} {w} {vi} att={att} nat={nat} -> {}", if is_vi { "vi" } else { "en" });
        amb.push_str(&format!("{w}\t{}\n", if is_vi { "vi" } else { "en" }));
    }

    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
    std::fs::write(format!("{dir}/en_words.txt"), en).unwrap();
    std::fs::write(format!("{dir}/ambiguous_defaults.txt"), amb).unwrap();
}
