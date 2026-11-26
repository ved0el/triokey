//! Real-text benchmark: how often does a person who mixes English and Vietnamese get the
//! wrong word, and how many keystrokes does fixing it cost? Measures, does not gate:
//! `cargo test -p triokey-engine --release --test realtext -- --ignored --nocapture`.
//!
//! Model: each input line is a message as the person wants it to appear. A simulated
//! typist types it with a fresh engine: English words letter by letter as written,
//! Vietnamese words in Telex with a deterministic, per-word choice of where the tone key
//! goes (end of word, after the toned letter, after the vowel group) and of `uow`/`uwow`.
//! Every cluster (text between spaces) on the final screen is compared with the wanted
//! one, ignoring old/new tone placement (`hòa` == `hoà`). A wrong cluster is "flip
//! fixable" when one backspace right after its space swaps it to the wanted reading.
//! Cost = keystrokes spent fixing. A correct cluster is "armed" when one backspace after
//! its space does anything but delete that space (it swaps to the other reading instead).
//! vi_lost = Vietnamese clusters left raw or wrong. objective = cost/100 + 0.05 * armed/100:
//! roughly a 1.5% chance that the person presses Backspace right after a correct word,
//! times about 3 keystrokes to repair the surprise.
//!
//! Sets (`REALTEXT_SETS`, comma separated, default `mixed,web`): `mixed` = `tests/data/mixed/`,
//! `web` = `$TRIOKEY_CORPUS` (default `<repo>/.corpus/bench`), both split dev/heldout by
//! FNV-1a (`REALTEXT_SPLIT`); `final` = `tests/data/final/`, always whole. Pooled rows from
//! summed counts: `own` (all `mixed` sets) and `final`. The "top wrong" lists are printed for
//! a `dev` run only and never for a `final` set; the table always. `REALTEXT_DUMP=<file>` writes
//! a TSV of failures (a `dev` run only, never of `final`), `REALTEXT_JSON=<file>` a summary.
//!
//! repeat = wrong clusters whose lowercased keys and expected text already came out wrong
//! and flip fixable earlier in the same set ("fixing the same thing twice"). Keys, not words:
//! the typist varies where the tone key goes, so `soonsg` and `soongs` count apart, as do
//! their remembered entries.
//! `REALTEXT_LEARN=1`: each set shares one remembered map (fresh per set), and the typist
//! fixes a flip-fixable cluster at once (backspace, then the space or Enter again), so the swap is
//! kept and remembered; clusters are graded as they first came out. harm = wrong clusters
//! that would have been right with their words' remembered entries removed.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use triokey_engine::lexicon::Lexicon;
use triokey_engine::telex::compose;
use triokey_engine::{derive_keys, Config, Edit, Engine, Outcome, ToneStyle};

/// Base vowel then its sắc, huyền, hỏi, ngã, nặng forms.
const TONED: &[&str] = &["aáàảãạ", "ăắằẳẵặ", "âấầẩẫậ", "eéèẻẽẹ", "êếềểễệ", "iíìỉĩị", "oóòỏõọ", "ôốồổỗộ", "ơớờởỡợ", "uúùủũụ", "ưứừửữự", "yýỳỷỹỵ"];
const TONE_KEYS: [char; 5] = ['s', 'f', 'r', 'x', 'j'];
const CLASSES: [&str; 5] = ["en_mangled", "en_altered", "vi_raw", "vi_wrong", "line_shift"];

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3))
}

/// (lowercase base letter keeping its hat/horn/breve, tone 0..=5, was uppercase).
fn split_tone(c: char) -> (char, usize, bool) {
    let lower = c.to_lowercase().next().unwrap_or(c);
    let (base, tone) = TONED.iter().find_map(|row| row.chars().position(|x| x == lower).map(|i| (row.chars().next().unwrap(), i))).unwrap_or((lower, 0));
    (base, tone, c.is_uppercase())
}

fn is_vi_letter(c: char) -> bool {
    // a single-char lowercase only: `İ` lowercases to `i` plus a combining dot
    let mut lower = c.to_lowercase();
    let (Some(l), None) = (lower.next(), lower.next()) else { return false };
    c.is_ascii_alphabetic() || l == 'đ' || TONED.iter().any(|row| row.contains(l))
}

/// Lines with anything but printable ASCII and Vietnamese letters are skipped.
fn allowed(line: &str) -> bool {
    line.chars().all(|c| (' '..='~').contains(&c) || is_vi_letter(c))
}

/// Letter runs lose their tone and carry it as a suffix, so `hòa` and `hoà` compare equal.
/// A run with two tones is kept literally.
fn canon(s: &str) -> String {
    let mut out = String::new();
    let mut run: Vec<char> = Vec::new();
    let flush = |run: &mut Vec<char>, out: &mut String| {
        if run.is_empty() {
            return;
        }
        let parts: Vec<_> = run.iter().map(|&c| split_tone(c)).collect();
        let tones: Vec<usize> = parts.iter().map(|p| p.1).filter(|&t| t > 0).collect();
        if tones.len() > 1 {
            out.extend(run.iter());
            out.push('\u{2}');
        } else {
            for &(b, _, up) in &parts {
                out.extend(if up { b.to_uppercase().collect::<Vec<_>>() } else { vec![b] });
            }
            out.push('\u{1}');
            out.push(char::from(b'0' + tones.first().copied().unwrap_or(0) as u8));
        }
        run.clear();
    };
    for c in s.chars() {
        if c.is_alphabetic() {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}

/// Grading equality: CANON, except that a tone sitting elsewhere than in the expected word
/// only passes when it is the old or new standard spelling (`hòa`/`hoà`); `TRUỲÊN` is not
/// `TRUYỀN`. A non-standard expected spelling (`qúy`) accepts the standard one (`quý`).
fn same(got: &str, exp: &str) -> bool {
    if got == exp {
        return true;
    }
    if canon(got) != canon(exp) {
        return false;
    }
    let runs = |s: &str| s.split(|c: char| !c.is_alphabetic()).map(String::from).collect::<Vec<_>>();
    // standard spellings come from lowercase keys: the Telex crate misplaces ALL CAPS tones
    let standard = |e: &str, st| -> String {
        compose(&derive_keys(&e.to_lowercase()), st)
            .chars()
            .zip(e.chars())
            .flat_map(|(c, u)| if u.is_uppercase() { c.to_uppercase().collect() } else { vec![c] })
            .collect()
    };
    runs(got).iter().zip(runs(exp)).all(|(g, e)| *g == e || [ToneStyle::Old, ToneStyle::New].into_iter().any(|st| *g == standard(&e, st)))
}

fn composes_to(keys: &[char], word: &str) -> bool {
    let want = canon(word);
    canon(&compose(keys, ToneStyle::Old)) == want || canon(&compose(keys, ToneStyle::New)) == want
}

/// Telex keys for one letter run. `h` picks the tone position (60% end, 25% after the toned
/// letter, 15% after the vowel group) and `ươ` as `uow` or `uwow`. None: untypable.
fn word_keys(word: &str, h: u64) -> Option<Vec<char>> {
    if word.is_ascii() {
        return Some(word.chars().collect());
    }
    let case = |k: char, up: bool| if up { k.to_ascii_uppercase() } else { k };
    let parts: Vec<_> = word.chars().map(split_tone).collect();
    let uow = (h >> 40) & 1 == 1;
    // a one-letter word is Shift plus the letter, then the mark or tone key unshifted (`Á` is `As`)
    let one = parts.len() == 1;
    let mut groups: Vec<Vec<char>> = Vec::new();
    for (i, &(b, _, up)) in parts.iter().enumerate() {
        let (k1, k2) = match b {
            'â' => ('a', Some('a')),
            'ă' => ('a', Some('w')),
            'ê' => ('e', Some('e')),
            'ô' => ('o', Some('o')),
            'ơ' => ('o', Some('w')),
            'ư' if uow && parts.get(i + 1).is_some_and(|p| p.0 == 'ơ') => ('u', None),
            'ư' => ('u', Some('w')),
            'đ' => ('d', Some('d')),
            b => (b, None),
        };
        groups.push(std::iter::once(case(k1, up)).chain(k2.map(|k| case(k, up && !one))).collect());
    }
    let toned: Vec<usize> = (0..parts.len()).filter(|&i| parts[i].1 > 0).collect();
    if toned.len() > 1 {
        return None;
    }
    if let Some(&t) = toned.first() {
        let is_vowel = |i: usize| groups[i][0].to_ascii_lowercase().to_string().contains(['a', 'e', 'i', 'o', 'u', 'y']);
        let after = match h % 100 {
            0..=59 => parts.len() - 1,
            60..=84 => t,
            _ => (t..parts.len()).take_while(|&i| is_vowel(i)).last().unwrap_or(t),
        };
        // Shift is held for the tone key only in an ALL CAPS word of two or more letters (`Ít` is `Its`, not `ItS`).
        let caps = !one && !word.chars().any(char::is_lowercase);
        groups[after].push(case(TONE_KEYS[parts[t].1 - 1], caps));
    }
    let keys: Vec<char> = groups.concat();
    if composes_to(&keys, word) {
        return Some(keys);
    }
    let fallback = derive_keys(word);
    composes_to(&fallback, word).then_some(fallback)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Key(char),
    Bound(char),
    Back,
}

/// Steps and the typed-keys string for each cluster (without the trailing space/newline).
fn plan_line(line: &str) -> Option<Vec<(Vec<Step>, String)>> {
    let mut out = Vec::new();
    for (ci, cluster) in line.split_whitespace().enumerate() {
        let (mut steps, mut keys, mut run, mut wi) = (Vec::new(), String::new(), String::new(), 0);
        let mut flush = |run: &mut String, steps: &mut Vec<Step>, keys: &mut String| -> Option<()> {
            if !run.is_empty() {
                let h = fnv1a(format!("{line}\u{1f}{ci}\u{1f}{wi}").as_bytes());
                for k in word_keys(run, h)? {
                    steps.push(Step::Key(k));
                    keys.push(k);
                }
                wi += 1;
                run.clear();
            }
            Some(())
        };
        for c in cluster.chars() {
            if c.is_alphabetic() {
                run.push(c);
            } else {
                flush(&mut run, &mut steps, &mut keys)?;
                steps.push(Step::Bound(c));
                keys.push(c);
            }
        }
        flush(&mut run, &mut steps, &mut keys)?;
        out.push((steps, keys));
    }
    Some(out)
}

/// The engine plus what the app shows, with the same rules as `tests/sim.rs`.
#[derive(Clone)]
struct Screen {
    e: Engine,
    text: String,
}

impl Screen {
    fn press(&mut self, step: Step) {
        let out = match step {
            Step::Key(c) => self.e.key(c),
            Step::Bound(c) => self.e.boundary(c),
            Step::Back => self.e.backspace(),
        };
        match (out, step) {
            (Outcome::Pass, Step::Back) => {
                self.text.pop();
            }
            (Outcome::Pass, Step::Key(c) | Step::Bound(c)) => self.text.push(c),
            (Outcome::Replace(Edit { delete, insert }), _) => {
                for _ in 0..delete {
                    self.text.pop();
                }
                self.text.push_str(&insert);
            }
        }
    }
}

#[derive(Debug)]
struct ClusterResult {
    expected: String,
    got: String,
    keys: String,
    /// None when right.
    class: Option<&'static str>,
    flip: bool,
    /// Whether one backspace after the space does more than delete it (None: ended the line).
    armed: Option<bool>,
    /// Wrong only because of a remembered entry (learn mode).
    harm: bool,
}

/// `s` just got a space: does one backspace do anything but delete it?
fn is_armed(s: &Screen) -> bool {
    let mut f = s.clone();
    f.press(Step::Back);
    f.text != s.text[..s.text.len() - 1]
}

fn class_of(expected: &str, got: &str) -> &'static str {
    match (expected.is_ascii(), got.is_ascii()) {
        (true, false) => "en_mangled",
        (true, true) => "en_altered",
        (false, true) => "vi_raw",
        (false, false) => "vi_wrong",
    }
}

/// Keystrokes to fix one cluster.
fn cost(r: &ClusterResult) -> usize {
    match (r.class, r.flip) {
        (None, _) => 0,
        (Some(_), true) => 2,
        (Some(_), false) => r.got.chars().count() + 1 + 2 + r.expected.chars().count() + 1,
    }
}

fn last_cluster(text: &str) -> String {
    text.trim_end_matches('\n').rsplit(' ').next().unwrap_or("").to_string()
}

/// Types one line (already filtered) and grades each cluster. None: a word is untypable.
/// `learn`: fix each flip-fixable cluster at once and grade clusters as they first came out.
fn measure_line(line: &str, lexicon: &Lexicon, learn: bool) -> Option<Vec<ClusterResult>> {
    let plan = plan_line(line)?;
    let expected: Vec<&str> = line.split_whitespace().collect();
    let mut s = Screen { e: Engine::new(Config::default(), lexicon.clone()), text: String::new() };
    let mut flips = Vec::new();
    let mut armed = Vec::new();
    let (mut first, mut harm) = (Vec::new(), Vec::new());
    for (ci, (steps, keys)) in plan.iter().enumerate() {
        let before = learn.then(|| s.clone());
        for &step in steps {
            s.press(step);
        }
        let last = ci + 1 == plan.len();
        let sep = Step::Bound(if last { '\n' } else { ' ' });
        s.press(sep);
        let mut f = s.clone();
        f.press(Step::Back);
        flips.push(last_cluster(&f.text));
        armed.push((!last).then(|| is_armed(&s)));
        let Some(before) = before else { continue };
        // without the separator just typed
        let now = last_cluster(&s.text[..s.text.len() - 1]);
        let wrong = !same(&now, expected[ci]);
        harm.push(wrong && !learned_harmless(&before, steps, sep, keys, expected[ci], lexicon));
        if wrong && same(&flips[ci], expected[ci]) {
            s.press(Step::Back);
            s.press(sep);
        }
        first.push(now);
    }
    let fin: Vec<&str> = s.text.strip_suffix('\n').unwrap_or(&s.text).split(' ').collect();
    let shifted = fin.len() != expected.len();
    let got: Vec<&str> = if learn { first.iter().map(String::as_str).collect() } else { fin };
    Some(
        expected
            .iter()
            .enumerate()
            .map(|(i, &exp)| {
                let g = got.get(i).copied().unwrap_or("");
                let class = if shifted { Some("line_shift") } else { (!same(g, exp)).then(|| class_of(exp, g)) };
                ClusterResult {
                    expected: exp.to_string(),
                    got: g.to_string(),
                    keys: plan[i].1.clone(),
                    class,
                    flip: class.is_some() && same(&flips[i], exp),
                    armed: armed[i],
                    harm: harm.get(i).copied().unwrap_or(false),
                }
            })
            .collect(),
    )
}

/// False when the cluster typed from `before` comes out right once the remembered entries for
/// its words are taken away (they are put back afterwards).
fn learned_harmless(before: &Screen, steps: &[Step], sep: Step, keys: &str, exp: &str, lexicon: &Lexicon) -> bool {
    let learned = lexicon.learned();
    let words: Vec<String> = keys.split(|c: char| !c.is_ascii_alphabetic()).filter(|w| !w.is_empty()).map(str::to_lowercase).collect();
    let old: Vec<(String, String)> = words.into_iter().filter_map(|w| learned.get(&w).map(|t| (w, t))).collect();
    if old.is_empty() {
        return true;
    }
    old.iter().for_each(|(w, _)| learned.set(w, None));
    let mut b = before.clone();
    steps.iter().chain([&sep]).for_each(|&step| b.press(step));
    old.iter().for_each(|(w, t)| learned.set(w, Some(t)));
    !same(&last_cluster(&b.text[..b.text.len() - 1]), exp)
}

#[derive(Default)]
struct SetStats {
    lines: usize,
    skipped: usize,
    clusters: usize,
    wrong: usize,
    classes: BTreeMap<&'static str, usize>,
    flip_fixable: usize,
    cost: usize,
    armed_correct: usize,
    repeat: usize,
    harm: usize,
    pairs: HashMap<(String, String, &'static str, bool), usize>,
}

impl SetStats {
    fn per_100(&self, x: f64) -> f64 {
        if self.clusters == 0 {
            0.0
        } else {
            100.0 * x / self.clusters as f64
        }
    }

    fn class(&self, k: &str) -> usize {
        self.classes.get(k).copied().unwrap_or(0)
    }

    /// (armed/100, vi_lost/100, objective)
    fn derived(&self) -> (f64, f64, f64) {
        let armed = self.per_100(self.armed_correct as f64);
        (armed, self.per_100((self.class("vi_raw") + self.class("vi_wrong")) as f64), self.per_100(self.cost as f64) + 0.05 * armed)
    }

    /// Pools counts (not the pair list) for a summary row.
    fn add(&mut self, o: &SetStats) {
        self.lines += o.lines;
        self.skipped += o.skipped;
        self.clusters += o.clusters;
        self.wrong += o.wrong;
        self.flip_fixable += o.flip_fixable;
        self.cost += o.cost;
        self.armed_correct += o.armed_correct;
        self.repeat += o.repeat;
        self.harm += o.harm;
        for (k, n) in &o.classes {
            *self.classes.entry(k).or_default() += n;
        }
    }
}

fn txt_files(dir: &Path) -> Option<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(dir).ok()?.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect();
    files.sort();
    Some(files)
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[test]
fn fnv1a_matches_reference() {
    assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn canon_ignores_tone_position_only() {
    assert_eq!(canon("hòa"), canon("hoà"));
    assert_eq!(canon("Hòa,"), canon("Hoà,"));
    assert_ne!(canon("hòa"), canon("hoá"));
    assert_ne!(canon("hoa"), canon("hòa"));
    assert_ne!(canon("viet"), canon("việt"));
    assert_ne!(canon("hòà"), canon("hoàà"));
}

#[test]
fn same_accepts_only_standard_tone_moves() {
    assert!(same("hòa", "hoà") && same("Thủy,", "Thuỷ,") && same("quý", "qúy") && same("test", "test"));
    // caps-lock `TRUYEENF` gives `TRUỲÊN`: same letters and tone, but misplaced
    assert!(!same("TRUỲÊN", "TRUYỀN") && same("THUỶ", "THỦY") && same("Hoà", "Hòa"));
    assert!(!same("hoá", "hòa") && !same("ban", "bạn"));
}

#[test]
fn allowed_chars() {
    assert!(allowed("Tôi push lên main, OK?"));
    assert!(allowed("ĐƯỢC rồi"));
    assert!(!allowed("café ☕"));
    assert!(!allowed("tab\there"));
    assert!(!allowed("İ"));
}

#[test]
fn word_keys_tone_positions_and_uow() {
    let k = |w, h| word_keys(w, h).map(|v| v.into_iter().collect::<String>());
    assert_eq!(k("push", 0).as_deref(), Some("push"));
    assert_eq!(k("hòa", 0).as_deref(), Some("hoaf"));
    assert_eq!(k("hòa", 60).as_deref(), Some("hofa"));
    assert_eq!(k("hòa", 85).as_deref(), Some("hoaf"));
    assert_eq!(k("người", 0).as_deref(), Some("nguwowif"));
    // 25 << 40: bit 40 set (uow) and h % 100 == 0 (tone at end)
    assert_eq!(k("người", 25 << 40).as_deref(), Some("nguowif"));
    assert_eq!(k("người", 60).as_deref(), Some("nguwowfi"));
    assert_eq!(k("Đường", 0).as_deref(), Some("DDuwowngf"));
    assert_eq!(k("VIỆT", 0).as_deref(), Some("VIEETJ"));
    assert_eq!(k("Ít", 0).as_deref(), Some("Its"));
    assert_eq!(k("Ấy", 60).as_deref(), Some("AAsy"));
    for (w, keys) in [("Á", "As"), ("Ô", "Oo"), ("Đ", "Dd"), ("Ư", "Uw"), ("Ố", "Oos"), ("ĐI", "DDI")] {
        assert_eq!(k(w, 0).as_deref(), Some(keys));
    }
    assert_eq!(k("hòà", 0), None);
    // every variant must compose back
    for w in ["được", "thuở", "khuyến", "giữa", "quả", "Nguyễn", "ăn", "đâu"] {
        for h in [0, 60, 85, 1 << 40, (1 << 40) + 61, (1 << 40) + 90] {
            let keys = word_keys(w, h).unwrap_or_else(|| panic!("{w} untypable"));
            assert!(composes_to(&keys, w), "{w} {keys:?}");
        }
    }
}

#[test]
fn plan_line_splits_words_and_boundaries() {
    let plan = plan_line("sửa  file.rs nhé").unwrap();
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[1].1, "file.rs");
    assert_eq!(plan[1].0[4], Step::Bound('.'));
}

#[test]
fn measure_line_grades_clusters() {
    let r = measure_line("chào bạn, push code", &Lexicon::builtin(), false).unwrap();
    assert_eq!(r.len(), 4);
    assert!(r.iter().all(|c| c.class.is_none()), "{r:?}");
    assert!(r[..3].iter().all(|c| c.armed.is_some()) && r[3].armed.is_none());
    let wrong =
        ClusterResult { expected: "bạn".into(), got: "banj".into(), keys: "banj".into(), class: Some("vi_raw"), flip: false, armed: None, harm: false };
    assert_eq!(cost(&wrong), 5 + 2 + 3 + 1);
    assert_eq!(class_of("code", "côde"), "en_mangled");
    assert_eq!(class_of("bạn", "ban"), "vi_raw");
}

#[test]
fn armed_check() {
    let lex = Lexicon::builtin();
    // with the current engine a backspace after `this ` swaps it to its other reading
    let this = measure_line("this is", &lex, false).unwrap();
    assert!(this[0].class.is_none() && this[0].armed == Some(true), "{this:?}");
    // `tooi ` -> `tôi `: a backspace only deletes the space
    let toi = measure_line("tôi đi", &lex, false).unwrap();
    assert!(toi[0].class.is_none() && toi[0].armed == Some(false), "{toi:?}");
}

#[test]
#[ignore]
fn realtext_report() {
    let lexicon = Lexicon::builtin();
    let split = std::env::var("REALTEXT_SPLIT").unwrap_or_else(|_| "dev".into());
    assert!(matches!(split.as_str(), "dev" | "heldout" | "all"), "REALTEXT_SPLIT must be dev, heldout or all");
    let kinds_env = std::env::var("REALTEXT_SETS").unwrap_or_else(|_| "mixed,web".into());
    let kinds: Vec<&str> = kinds_env.split(',').map(str::trim).filter(|k| !k.is_empty()).collect();
    assert!(kinds.iter().all(|k| matches!(*k, "mixed" | "web" | "final")), "REALTEXT_SETS takes mixed, web, final");
    let learn = std::env::var("REALTEXT_LEARN").is_ok_and(|v| v == "1");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<(&str, PathBuf)> = Vec::new();
    for &kind in ["mixed", "web", "final"].iter().filter(|k| kinds.contains(k)) {
        let dir = match kind {
            "mixed" => root.join("tests/data/mixed"),
            "final" => root.join("tests/data/final"),
            _ => std::env::var_os("TRIOKEY_CORPUS").map(PathBuf::from).unwrap_or_else(|| root.join("../../.corpus/bench")),
        };
        match txt_files(&dir) {
            Some(f) => files.extend(f.into_iter().map(|p| (kind, p))),
            None => println!("note: no {kind} sets at {}, skipped", dir.display()),
        }
    }
    let mut sets: Vec<(String, SetStats)> = Vec::new();
    let (mut own, mut fin) = (SetStats::default(), SetStats::default());
    type DumpKey = (String, &'static str, bool, String, String, String, String);
    let mut dump: HashMap<DumpKey, usize> = HashMap::new();
    for (kind, path) in &files {
        let stem = path.file_stem().unwrap().to_string_lossy();
        let name = if *kind == "final" { format!("final/{stem}") } else { stem.into_owned() };
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut st = SetStats::default();
        // learn: one remembered map per set
        let lexicon = if learn { Lexicon::builtin() } else { lexicon.clone() };
        let mut fixed: HashSet<(String, String)> = HashSet::new();
        for line in text.trim_start_matches('\u{feff}').lines() {
            let held = fnv1a(line.as_bytes()).is_multiple_of(5);
            if *kind != "final" && ((split == "dev" && held) || (split == "heldout" && !held)) {
                continue;
            }
            let results = (!line.trim().is_empty() && allowed(line)).then(|| measure_line(line, &lexicon, learn)).flatten();
            let Some(results) = results else {
                st.skipped += 1;
                continue;
            };
            st.lines += 1;
            for (i, r) in results.iter().enumerate() {
                st.clusters += 1;
                st.cost += cost(r);
                let Some(class) = r.class else {
                    st.armed_correct += (r.armed == Some(true)) as usize;
                    continue;
                };
                st.wrong += 1;
                *st.classes.entry(class).or_default() += 1;
                st.flip_fixable += r.flip as usize;
                st.harm += r.harm as usize;
                let seen = (r.keys.to_lowercase(), r.expected.to_lowercase());
                st.repeat += fixed.contains(&seen) as usize;
                if r.flip {
                    fixed.insert(seen);
                }
                *st.pairs.entry((r.expected.clone(), r.got.clone(), class, r.flip)).or_default() += 1;
                if *kind != "final" {
                    let prev = if i == 0 { "^".to_string() } else { results[i - 1].expected.clone() };
                    *dump.entry((name.clone(), class, r.flip, prev, r.expected.clone(), r.keys.clone(), r.got.clone())).or_default() += 1;
                }
            }
        }
        match *kind {
            "mixed" => own.add(&st),
            "final" => fin.add(&st),
            _ => {}
        }
        sets.push((name, st));
    }
    let mut rows: Vec<(&str, &SetStats)> = sets.iter().map(|(n, st)| (n.as_str(), st)).collect();
    if kinds.contains(&"mixed") {
        rows.push(("own", &own));
    }
    if kinds.contains(&"final") {
        rows.push(("final", &fin));
    }

    println!("split: {split}  sets: {}{}", kinds.join(","), if learn { "  learn: on" } else { "" });
    print!(
        "{:20} {:>6} {:>5} {:>8} {:>6} {:>6} {:>10} {:>10} {:>6} {:>8} {:>10} {:>5} {:>8} {:>6} {:>9} {:>11} {:>9}",
        "set",
        "lines",
        "skip",
        "clusters",
        "wrong",
        "wrong%",
        "en_mangled",
        "en_altered",
        "vi_raw",
        "vi_wrong",
        "line_shift",
        "flip",
        "cost/100",
        "armed",
        "armed/100",
        "vi_lost/100",
        "objective"
    );
    println!(" {:>6}{}", "repeat", if learn { format!(" {:>6}", "harm") } else { String::new() });
    for (name, st) in &rows {
        let (armed, vi_lost, objective) = st.derived();
        print!(
            "{:20} {:>6} {:>5} {:>8} {:>6} {:>6.2} {:>10} {:>10} {:>6} {:>8} {:>10} {:>5} {:>8.2} {:>6} {:>9.2} {:>11.2} {:>9.3}",
            name,
            st.lines,
            st.skipped,
            st.clusters,
            st.wrong,
            st.per_100(st.wrong as f64),
            st.class("en_mangled"),
            st.class("en_altered"),
            st.class("vi_raw"),
            st.class("vi_wrong"),
            st.class("line_shift"),
            st.flip_fixable,
            st.per_100(st.cost as f64),
            st.armed_correct,
            armed,
            vi_lost,
            objective
        );
        println!(" {:>6}{}", st.repeat, if learn { format!(" {:>6}", st.harm) } else { String::new() });
    }
    // Failures are shown for dev only: held-out lines and the final sets stay unread.
    for (name, st) in sets.iter().filter(|(n, _)| split == "dev" && !n.starts_with("final/")) {
        let mut pairs: Vec<_> = st.pairs.iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        println!("--- {name}: top wrong (expected -> got) ---");
        for ((exp, got, class, flip), n) in pairs.into_iter().take(40) {
            println!("{n:5}  {exp} -> {got}  [{class}{}]", if *flip { ", flip" } else { "" });
        }
    }

    if let Ok(path) = std::env::var("REALTEXT_DUMP").map_err(|_| ()).and_then(|p| if split == "dev" { Ok(p) } else { Err(()) }) {
        if kinds.contains(&"final") {
            println!("note: final sets are never dumped; {path} holds the other sets only");
        }
        let mut dumped: Vec<_> = dump.into_iter().collect();
        dumped.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut tsv = String::from("set\tclass\tflip\tprev\texpected\tkeys\tgot\tcount\n");
        for ((set, class, flip, prev, exp, keys, got), n) in dumped {
            writeln!(tsv, "{set}\t{class}\t{}\t{prev}\t{exp}\t{keys}\t{got}\t{n}", flip as u8).unwrap();
        }
        std::fs::write(path, tsv).unwrap();
    }
    if let Ok(path) = std::env::var("REALTEXT_JSON") {
        let body: Vec<String> = rows
            .iter()
            .map(|(name, st)| {
                let classes: Vec<String> = CLASSES.iter().map(|k| format!("{}:{}", json_str(k), st.class(k))).collect();
                let (armed, vi_lost, objective) = st.derived();
                format!(
                    "    {{\"name\":{},\"lines\":{},\"skipped\":{},\"clusters\":{},\"wrong\":{},\"classes\":{{{}}},\"flip_fixable\":{},\"cost_per_100\":{:.3},\"armed_correct\":{},\"armed_per_100\":{:.3},\"vi_lost_per_100\":{:.3},\"objective\":{:.3},\"repeat\":{}{}}}",
                    json_str(name),
                    st.lines,
                    st.skipped,
                    st.clusters,
                    st.wrong,
                    classes.join(","),
                    st.flip_fixable,
                    st.per_100(st.cost as f64),
                    st.armed_correct,
                    armed,
                    vi_lost,
                    objective,
                    st.repeat,
                    if learn { format!(",\"harm\":{}", st.harm) } else { String::new() }
                )
            })
            .collect();
        std::fs::write(path, format!("{{\"split\":{}{},\"sets\":[\n{}\n]}}\n", json_str(&split), if learn { ",\"learn\":true" } else { "" }, body.join(",\n")))
            .unwrap();
    }
}

#[test]
fn learn_mode_fixes_once() {
    let lex = Lexicon::builtin();
    let first = measure_line("the sóng", &lex, true).unwrap();
    assert!(first[1].class.is_some() && first[1].flip, "{first:?}");
    assert_eq!(lex.learned().get("songs").as_deref(), Some("sóng"));
    let again = measure_line("the sóng", &lex, true).unwrap();
    assert!(again[1].class.is_none(), "{again:?}");
    // without learning the same line stays wrong
    assert!(measure_line("the sóng", &Lexicon::builtin(), false).unwrap()[1].class.is_some());
}
