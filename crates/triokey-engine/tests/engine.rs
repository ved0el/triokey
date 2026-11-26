mod sim;

use sim::type_str;
use triokey_engine::lexicon::{Learned, Lexicon};
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
    // Both readings are words: as typed, and ⌫ gives the other when it is not far rarer.
    err: "err " => "err ";
    err_undo: "err ⌫" => "er";
    off: "off " => "off ";
    off_undo: "off ⌫" => "of";
    boss: "boss " => "boss ";
    verr_typing: "verr" => "ver";
    terrible: "terrible " => "terrible ";
    sorry: "sorry " => "sorry ";
    mixx: "mixx " => "mix ";
    usser: "usser " => "user ";
    // A doubled letter that escaped no Vietnamese reading stays as typed.
    doubled_letters: "offline passwd lossless Terraform " => "offline passwd lossless Terraform ";
    offline_typing: "offl" => "offl";
    upper_uye: "TRUYEENF " => "TRUYỀN ";
    xooong: "xooong " => "xoong ";
    bijj: "bijj " => "bij ";
    this: "this " => "this ";
    has: "has " => "has ";
    // Plain `toi` cannot give `tôi` in Telex, so `tooi` is typed.
    context_vi: "tooi thaasy this " => "tôi thấy thí ";
    // Mixed lines that must keep coming out right.
    accept_tech_in_vi: "tooi caanf test laij " => "tôi cần test lại ";
    accept_list_backspace: "mowr list ⌫" => "mở list";
    accept_ca_si: "ca six " => "ca sĩ ";
    accept_os: "OS naof cungx dduwowcj " => "OS nào cũng được ";
    accept_doubled: "offline passwd lossless diff " => "offline passwd lossless diff ";
    accept_abbr: "ko ddc ddaau " => "ko đc đâu ";
    accept_abbr_typing: "ddc" => "đc";
    accept_sure_word_unarmed: "tooi ⌫" => "tôi";
    // A swapped word counts for the line as swapped: `thí` makes the line Vietnamese.
    undo_moves_line: "this ⌫ songs " => "thí sóng ";
    english_line: "this songs " => "this songs ";
    newline_resets_line: "the the the
songs " => "the the the
sóng ";
    shouted_line: "TOOI DDI HOCJ
CA SIX " => "TÔI ĐI HỌC
CA SĨ ";
    lone_caps: "USA EOF CORS " => "USA EOF CORS ";
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
    // `mixx` is no word, so `mix` is near-certain: ⌫ only deletes the space.
    undo_double: "mixx ⌫" => "mix";
    backspace_mid_word: "tieesng⌫⌫ng " => "tiếng ";
    // After ⌫ falls back to raw keys, the word stays raw through the commit.
    backspace_raw_xoong: "xooong⌫g " => "xoong ";
    backspace_raw_bos: "boss⌫ " => "bos ";
    upper_first: "Vieejt " => "Việt ";
    upper_all: "VIEEJT " => "VIỆT ";
    upper_dd: "DDuowngf " => "Đường ";
    // `nguoi` is not a strict prefix but can still take marks: must not go raw.
    late_marks: "nguoiwf " => "người ";
    // Words with two readings follow the line: English after English words...
    line_en_plurals: "the songs photos teens guns trips cats loops " => "the songs photos teens guns trips cats loops ";
    line_en_words: "we defend born theme nest reset virus taxi rows beef " => "we defend born theme nest reset virus taxi rows beef ";
    // ...and on a fresh line, which leans Vietnamese, the commoner reading wins.
    fresh_line_vi: "hats
bans
teen
sangs
toots
tees
bees
songs
cows
moos
been " => "hát
bán
tên
sáng
tốt
tế
bế
sóng
cớ
mố
bên ";
    fresh_line_en: "ox
last " => "ox
last ";
    vi_not_english: "ddi\ncos\nlaf\nhoaf\nthuys\nnhuw " => "đi\ncó\nlà\nhòa\nthúy\nnhư ";
    // `có` is far commoner than `cos`, even on an English line.
    vi_after_english: "server cos " => "server có ";
    // On a Vietnamese line the Vietnamese reading wins what a fresh line gives to English.
    tie_en_after_vi: "tooi thaasy songs " => "tôi thấy sóng ";
    // However Vietnamese the line, a common English word that lost stays one ⌫ away.
    saturated_line_undo: "tooi ddax nois vowis anh aays raats nhieeuf laanf sets ⌫" => "tôi đã nói với anh ấy rất nhiều lần sets";
    // ⌫ into the previous word keeps the line's mode.
    backspace_keeps_line: "the code is ⌫ there " => "the code is there ";
    // A correction inside a Vietnamese word still composes; a real word stays as typed.
    backspace_fix_composes: "tooi ddi hocc⌫j " => "tôi đi học ";
    // Lone letters and vowel fragments are no abbreviations.
    lone_dd: "dd/mm/yyyy aa aw " => "dd/mm/yyyy aa aw ";
    vowel_fragment: "tooi wow " => "tôi wow ";
    // `z` removes the tone.
    tone_removed: "toansz " => "toan ";
    // Two capitals are an acronym wherever they stand; a one-letter word is typed `As`.
    acronym_after_word: "Mac OS " => "Mac OS ";
    acronym_in_vi: "caif OS mowis " => "cài OS mới ";
    acronym_us: "team US ddang online " => "team US đang online ";
    lone_cap_vi: "chaau As " => "châu Á ";
    // A shouted line still composes: each word after an ALL CAPS Vietnamese one is scored normally.
    shouted_lone_cap: "VIEEJT NAM CHAAU AS " => "VIỆT NAM CHÂU Á ";
}

#[test]
fn force_files_override() {
    let lexicon = Lexicon::builtin().with_user(
        "this
",
        "off
tooi
",
    );
    let mut e = Engine::new(Config::default(), lexicon);
    assert_eq!(type_str(&mut e, "this off tooi "), "thí off tooi ");
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

fn learner(lexicon: Lexicon) -> (Learned, Engine) {
    let learned = lexicon.learned().clone();
    (learned, Engine::new(Config::default(), lexicon))
}

#[test]
fn kept_swap_is_remembered_and_swaps_back() {
    let (learned, mut e) = learner(Lexicon::builtin());
    assert_eq!(type_str(&mut e, "this ⌫ "), "thí ");
    assert_eq!(learned.get("this").as_deref(), Some("thí"));
    // next time, even on a fresh line that alone gives `this`; the case follows the keys
    assert_eq!(type_str(&mut e, "\nThis "), "\nThí ");
    // one ⌫ still gives what scoring picks; kept, and a fresh line picks it anyway: forgotten
    assert_eq!(type_str(&mut e, "\nthis ⌫ "), "\nthis ");
    assert_eq!(learned.get("this"), None);
    assert_eq!(type_str(&mut e, "\nthis "), "\nthis ");
}

#[test]
fn swapping_back_replaces_the_entry() {
    let (learned, mut e) = learner(Lexicon::builtin());
    assert_eq!(type_str(&mut e, "the songs ⌫ "), "the sóng ");
    assert_eq!(learned.get("songs").as_deref(), Some("sóng"));
    assert_eq!(type_str(&mut e, "\nthe songs "), "\nthe sóng ");
    // back to `songs`, which a fresh line would not pick: remembered instead
    assert_eq!(type_str(&mut e, "\nthe songs ⌫ "), "\nthe songs ");
    assert_eq!(learned.get("songs").as_deref(), Some("songs"));
    assert_eq!(type_str(&mut e, "\nsongs "), "\nsongs ");
}

#[test]
fn swap_then_backspace_or_reset_is_not_remembered() {
    let (learned, mut e) = learner(Lexicon::builtin());
    assert_eq!(type_str(&mut e, "this ⌫⌫is "), "this ");
    e.reset();
    assert_eq!(type_str(&mut e, "this ⌫"), "thí");
    e.reset();
    type_str(&mut e, "a ");
    assert_eq!((learned.get("this"), learned.changes()), (None, 0));
}

#[test]
fn probe_on_a_clone_remembers_nothing() {
    let (learned, mut e) = learner(Lexicon::builtin());
    type_str(&mut e, "this ");
    let mut probe = e.clone();
    assert_eq!(type_str(&mut probe, "⌫"), "thí");
    drop(probe);
    type_str(&mut e, "is ");
    assert_eq!(learned.changes(), 0);
}

#[test]
fn force_files_beat_remembered_entries() {
    let lexicon = Lexicon::builtin().with_user("", "this\n").with_learned(Learned::parse("this\tthí\n"));
    let mut e = Engine::new(Config::default(), lexicon);
    assert_eq!(type_str(&mut e, "this "), "this ");
}

#[test]
fn entry_that_is_no_current_reading_is_ignored() {
    let lexicon = Lexicon::builtin().with_learned(Learned::parse("this\ttà\ntooi\tfoo\n"));
    let mut e = Engine::new(Config::default(), lexicon);
    assert_eq!(type_str(&mut e, "this tooi "), "this tôi ");
}

#[test]
fn letter_or_punctuation_after_a_swap_is_not_remembered() {
    let (learned, mut e) = learner(Lexicon::builtin());
    for keys in ["test ⌫s ", "test ⌫. ", "this ⌫, ", "as ⌫k "] {
        type_str(&mut e, keys);
        e.reset();
    }
    assert_eq!(learned.changes(), 0);
    // Enter keeps it, like a space
    assert_eq!(type_str(&mut e, "this ⌫\n"), "thí\n");
    assert_eq!(learned.get("this").as_deref(), Some("thí"));
}

#[test]
fn swap_of_a_forced_word_is_not_stored() {
    let (learned, mut e) = learner(Lexicon::builtin().with_user("", "this\n"));
    assert_eq!(type_str(&mut e, "this ⌫ "), "thí ");
    assert_eq!(learned.changes(), 0);
}

#[test]
fn remembered_word_yields_to_vietnamese_on_a_vietnamese_line() {
    let (learned, mut e) = learner(Lexicon::builtin().with_learned(Learned::parse("tooi\ttooi\n")));
    // at the start of a line the entry applies
    assert_eq!(type_str(&mut e, "tooi "), "tooi ");
    // once the line leans Vietnamese it does not replace the Vietnamese word...
    assert_eq!(type_str(&mut e, "\nddi tooi "), "\nđi tôi ");
    // ...but stays one Backspace away, and keeping that swap leaves the entry as it was
    assert_eq!(type_str(&mut e, "\nddi tooi ⌫ "), "\nđi tooi ");
    assert_eq!((learned.get("tooi").as_deref(), learned.changes()), (Some("tooi"), 0));
    // an entry toward Vietnamese still applies on an English line
    let (_, mut e) = learner(Lexicon::builtin().with_learned(Learned::parse("songs\tsóng\n")));
    assert_eq!(type_str(&mut e, "the the songs "), "the the sóng ");
}

/// Every "keys typed -> what appears" example in README.md and README.vi.md, written as there:
/// `␣` is a space, `⌫` Backspace, and the space a final `␣` leaves is not shown.
#[test]
fn readme_examples() {
    let typed = |e: &mut Engine, keys: &str| type_str(e, &keys.replace('␣', " ")).trim_end_matches(' ').to_string();
    // each on a fresh line
    for (keys, shown) in [
        ("tieesng vieejt␣", "tiếng việt"),
        ("windows␣", "windows"),
        ("songs␣", "sóng"),
        ("the songs␣", "the songs"),
        ("this␣⌫", "thí"),
        ("windows terrible␣", "windows terrible"),
        ("DDuowngf␣", "Đường"),
        ("baajt/taawst␣", "bật/tắt"),
        ("my_var obj.dir src/main␣", "my_var obj.dir src/main"),
        ("server cos␣", "server có"),
        ("tooi thaasy songs␣", "tôi thấy sóng"),
        ("this␣", "this"),
        ("this␣⌫⌫", "th"),
        ("off␣⌫", "of"),
        ("tooi␣⌫", "tôi"),
        ("tooi thaasy sets␣", "tôi thấy sét"),
        ("tooi thaasy sets␣⌫", "tôi thấy sets"),
        ("There␣", "Thể"),
        ("There␣⌫", "There"),
        ("mixx␣", "mix"),
        ("offline passwd Terraform␣", "offline passwd Terraform"),
        ("ko ddc ddaau␣", "ko đc đâu"),
        ("dd/mm/yyyy aa aw␣", "dd/mm/yyyy aa aw"),
        ("toansz␣", "toan"),
        ("OS naof cungx dduwowcj␣", "OS nào cũng được"),
        ("VIEEJT␣", "VIỆT"),
        ("chaau As␣", "châu Á"),
        ("Mac OS␣", "Mac OS"),
        // Telex and password bullets
        ("caan", "cân"),
        ("ddi", "đi"),
        ("tieesng", "tiếng"),
        ("tieengs", "tiếng"),
        ("w", "ư"),
        ("w␣", "w"),
        ("w␣⌫", "ư"),
        ("mas!x9", "má!x9"),
    ] {
        assert_eq!(typed(&mut Engine::new(Config::default(), Lexicon::builtin()), keys), shown, "keys: {keys:?}");
    }
    // remembering: one engine, each step on a new line, in order
    let mut e = Engine::new(Config::default(), Lexicon::builtin());
    for (keys, shown) in [
        ("this␣⌫␣", "thí"),
        ("this␣", "thí"),
        ("this␣⌫␣", "this"),
        ("this␣", "this"),
        ("songs␣⌫␣", "songs"),
        ("songs␣", "songs"),
        ("tooi thaasy songs␣", "tôi thấy sóng"),
        ("tooi thaasy songs␣⌫", "tôi thấy songs"),
    ] {
        assert_eq!(typed(&mut e, &format!("\n{keys}")).trim_start_matches('\n'), shown, "keys: {keys:?}");
    }
}
