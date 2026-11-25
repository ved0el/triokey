use triokey_engine::syllable::{is_prefix, is_syllable};

#[test]
fn valid_syllables() {
    for w in [
        "tiếng",
        "việt",
        "người",
        "đường",
        "khuỷu",
        "xoong",
        "gì",
        "quá",
        "già",
        "thí",
        "bên",
        "mĩ",
        "sê",
        "hòa",
        "hoà",
        "ngoẹo",
        "nghĩa",
        "ở",
        "ừ",
        "kha",
        "ka",
        "ca",
        "ki",
        "tác",
        "tạp",
        "Việt",
        "VIỆT",
        "Đường",
        "quýt",
        "quỳnh",
        "Quỳnh",
        "quých",
        "quy",
        "quyên",
    ] {
        assert!(is_syllable(w), "{w} should be a syllable");
    }
}

#[test]
fn invalid_syllables() {
    for w in ["uindows", "tẽt", "goặp", "bãt", "ce", "ci", "ngí", "ghá", "tàp", "textt", "", "tap", "hóà", "týt", "hỳnh", "kan", "kao", "kanh"] {
        assert!(!is_syllable(w), "{w} should not be a syllable");
    }
}

#[test]
fn prefixes() {
    for w in ["ti", "tiê", "ng", "ngh", "qu", "đươ", "", "tiế", "Ngh", "quý", "quỳn"] {
        assert!(is_prefix(w), "{w} should be a prefix");
    }
    for w in ["tx", "bl", "uindows", "tàp"] {
        assert!(!is_prefix(w), "{w} should not be a prefix");
    }
}
