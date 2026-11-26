use triokey_engine::lexicon::Lexicon;
use triokey_engine::token::{is_mixed_case, starts_code};
use triokey_engine::Lang;

#[test]
fn rates() {
    let lx = Lexicon::builtin();
    for w in ["this", "has", "been", "less", "class", "error", "terrible", "sorry", "tomorrow", "mix", "user", "windows", "text", "err", "boss"] {
        let (vi, en) = lx.rates(w);
        assert!(en > vi, "{w} should be English");
    }
    for w in ["tiếng", "tôi", "được", "Việt"] {
        let (vi, en) = lx.rates(w);
        assert!(vi > en, "{w} should be Vietnamese");
    }
    assert_eq!(lx.rates("tieesng"), (0.0, 0.0));
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
