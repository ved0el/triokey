mod sim;

use sim::type_str;
use triokey_engine::lexicon::Lexicon;
use triokey_engine::{Config, Engine, ToneStyle};

fn run(config: Config, keys: &str) -> String {
    type_str(&mut Engine::new(config, Lexicon::builtin()), keys)
}

macro_rules! cases {
    ($($name:ident: $keys:expr => $screen:expr;)*) => {$(
        #[test]
        fn $name() {
            assert_eq!(run(Config::default(), $keys), $screen, "keys: {:?}", $keys);
        }
    )*};
}

cases! {
    vietnamese: "tieesng vieejt " => "tiếng việt ";
    windows: "windows " => "windows ";
    text: "text " => "text ";
    windows_cap: "Windows " => "Windows ";
    html: "HTML " => "HTML ";
    less: "less " => "less ";
    class: "class " => "class ";
    err: "err " => "err ";
    boss: "boss " => "boss ";
    verr_typing: "verr" => "ver";
    terrible: "terrible " => "terrible ";
    sorry: "sorry " => "sorry ";
    mixx: "mixx " => "mix ";
    usser: "usser " => "user ";
    xooong: "xooong " => "xoong ";
    bijj: "bijj " => "bij ";
    this: "this " => "this ";
    has: "has " => "has ";
    // Plan wrote `toi`; plain `toi` cannot give `tôi` in Telex, so `tooi` is typed.
    context_vi: "tooi thaasy this " => "tôi thấy thí ";
    obj_dir: "obj.dir " => "obj.dir ";
    dash_las: "-las " => "-las ";
    usr_bin: "/usr/bin " => "/usr/bin ";
    my_var: "my_var " => "my_var ";
    has123: "has123 " => "has123 ";
    iphone: "iPhone " => "iPhone ";
    slash_vi: "baajt/taawst " => "bật/tắt ";
    src_main: "src/main " => "src/main ";
    undo_tie: "this ⌫" => "thí";
    undo_then_delete: "this ⌫⌫" => "th";
    undo_double: "mixx ⌫" => "mixx";
    backspace_mid_word: "tieesng⌫⌫ng " => "tiếng ";
    // After ⌫ falls back to raw keys, the word stays raw through the commit.
    backspace_raw_xoong: "xooong⌫g " => "xoong ";
    backspace_raw_bos: "boss⌫ " => "bos ";
    upper_first: "Vieejt " => "Việt ";
    upper_all: "VIEEJT " => "VIỆT ";
    upper_dd: "DDuowngf " => "Đường ";
    // `nguoi` is not a strict prefix but can still take marks: must not go raw.
    late_marks: "nguoiwf " => "người ";
    // Tie defaults (decision 0005): English words whose reading is not an everyday
    // Vietnamese word typed the usual way stay English...
    tie_en_plurals: "songs photos teens guns trips cats loops " => "songs photos teens guns trips cats loops ";
    tie_en_words: "defend born theme nest reset virus taxi rows beef " => "defend born theme nest reset virus taxi rows beef ";
    // ...everyday Vietnamese and basic Telex drills stay Vietnamese. One per line: a
    // Vietnamese word would otherwise decide the next tie by context.
    tie_vi_common: "hats\nbans\nteen\nsangs\ntoots\ntees\ncows " => "hát\nbán\ntên\nsáng\ntốt\ntế\ncớ ";
    tie_vi_open_marked: "bees\nmoos\nox " => "bế\nmố\nõ ";
    vi_not_english: "ddi\ncos\nlaf\nhoaf\nthuys\nnhuw " => "đi\ncó\nlà\nhòa\nthúy\nnhư ";
    // `cos` is not in the English list, so English context cannot turn it English.
    vi_after_english: "server cos " => "server có ";
    // Vietnamese context still wins a tie that defaults to English.
    tie_en_after_vi: "tooi thaasy songs " => "tôi thấy sóng ";
}

#[test]
fn forty_letters_stay_raw() {
    let keys = "q".repeat(40) + " ";
    assert_eq!(run(Config::default(), &keys), keys);
}

#[test]
fn auto_english_off_is_plain_telex() {
    let config = Config { auto_english: false, ..Config::default() };
    assert_eq!(run(config, "windows "), "uindows ");
}

#[test]
fn new_tone_style() {
    let config = Config { tone_style: ToneStyle::New, ..Config::default() };
    assert_eq!(run(config, "hoaf "), "hoà ");
}
