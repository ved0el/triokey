use triokey_engine::lexicon::Lexicon;
use triokey_engine::token::{is_mixed_case, starts_code};
use triokey_engine::Lang;

#[test]
fn english_words() {
    let lx = Lexicon::builtin();
    for w in ["this", "has", "been", "less", "class", "error", "terrible", "sorry", "tomorrow", "mix", "user", "windows", "text", "err", "boss"] {
        assert!(lx.is_english(w), "{w} should be English");
    }
    for w in ["tieesng", "xoong", "toi", "bij"] {
        assert!(!lx.is_english(w), "{w} should not be English");
    }
}

#[test]
fn tie_defaults() {
    let lx = Lexicon::builtin();
    for w in ["this", "has", "of"] {
        assert_eq!(lx.tie_default(w), Lang::En, "{w}");
    }
    // Every entry of the data file is honoured.
    for line in include_str!("../data/ambiguous_defaults.txt").lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (w, lang) = line.split_once('\t').expect("word<TAB>lang");
        let want = match lang {
            "en" => Lang::En,
            "vi" => Lang::Vi,
            _ => panic!("bad lang in line {line:?}"),
        };
        assert_eq!(lx.tie_default(w), want, "{w}");
    }
}

#[test]
fn user_force_files() {
    let lx = Lexicon::builtin();
    assert_eq!(lx.forced("larry"), None);
    let lx = lx.with_user("# always Vietnamese\nthis\n\n", "# always English\n Larry \nnassr  # football club\n");
    assert_eq!(lx.forced("larry"), Some(Lang::En));
    assert_eq!(lx.forced("nassr"), Some(Lang::En));
    assert_eq!(lx.forced("this"), Some(Lang::Vi));
    assert_eq!(lx.forced("has"), None);
    assert_eq!(lx.forced("# always english"), None);
    assert!(lx.is_english("larry"));
    assert!(!lx.is_english("this"));
}

#[test]
fn code_start() {
    for c in ['-', '/', '\\', '.', '~', '$', '@', '#'] {
        assert!(starts_code(c), "{c}");
    }
    for c in ['a', 'Z', '1', ' ', '_', ':'] {
        assert!(!starts_code(c), "{c}");
    }
}

#[test]
fn mixed_case() {
    for w in ["iPhone", "macOS", "JavaScript", "ViỆt"] {
        assert!(is_mixed_case(w), "{w}");
    }
    for w in ["Việt", "VIỆT", "viet", "Windows", "HTML", "a", "A", ""] {
        assert!(!is_mixed_case(w), "{w}");
    }
}
