# 0005: English/Vietnamese tie defaults beyond SCOWL level 0

Date: 2026-11-25. Status: accepted. Changes the default rule of 0002; spec 5.4 rule 4 itself is
unchanged (forced file, then context, then the per-word default).

## Problem

0002 defaulted every tie outside SCOWL level 0 to `vi`. With no clear previous word, ordinary
English came out as Vietnamese non-words: `songs` -> `sóng`, `photos` -> `phốt`, `defend` ->
`đền`, `born` -> `bỏn`, `teens`, `guns`, `trips`. This was the biggest failure group in 0004.

## Rule

The runtime is unchanged; `tests/gen_lexicon.rs` now decides each row of
`ambiguous_defaults.txt`:

- Level 0: as before (`en`, except the hand-reviewed `cums`, `hes`, `sex`).
- Level 1+ (and VieLotus-only words): `vi` only if the keys are the **natural Telex spelling**
  of the reading AND the reading is either an **everyday Vietnamese word**
  (`tests/data/vi_common.txt`) or, for level 2+ only, an **open marked syllable**. Otherwise
  `en`.
  - Natural spelling: `derive_keys(reading)` (also `w` for `uw`, `uow` for `ươ`), with the one
    tone key typed anywhere. So `songs`/`sosng` count, `cars` (two tone keys), `photos` (`o..o`
    split by a consonant), `theme`, and `defend` (`d..d`, `e..e` split) do not.
  - Open marked syllable: no coda and either a vowel mark (`cows` = `cớ`, `bees` = `bế`) or a
    lone vowel (`ox` = `õ`). These are basic Telex drills (`syllable_primitive`), and at
    level 2 the English side is a short, rare word. Level 1 is left out so `rows`, `roof`,
    `beer`, `beef`, `bow` stay English.
  - `vi_common.txt` holds 36 syllables, hand-picked from the readings of the natural, attested
    level 1+ rows: `bán bản bố bốn tên quên tốt tế sáng hát mát môn rút cớ ...`. It is not a
    general frequency list. Contested rows: `hats`, `bans`, `teen`, `queen`, `moon`, `rust`,
    `mats` stay `vi`, and `songs` (`sóng`), `cats`/`cast` (`cát`), `nest`/`nets` (`nét`),
    `loops` (`lốp`), `sown` (`sơn`) go `en`. VieLotus's collision frequency table
    (`BilingualIntentArbiter.vietnameseCollisionLogFrequencies`) was used only as a cross-check.
- English list additions: words from VieLotus-IM lexicon `04_unconditional_english_words.txt`
  and `05_context_english_collision_words.txt` are added to `en_words.txt` (+103) **only if
  their reading is not an attested Vietnamese syllable** (`tests/data/vi_attested.txt`, 7,112
  syllables from VieLotus `VietnameseSyllableDictionary`, including an added `tý`). Adding `cors` (`có`),
  `cos`, `uri` (`ủi`), `teest` (`tết`) would make them English after any English word, since
  context comes before the default. `11_extended_english_escapes.txt` (repeated-key escapes,
  0004 group 2) is not used.

Result: the table has 681 rows (was 651), 72 `vi`. VieLotus suite 8009 -> 8361 / 9084; every
`vn_*`, `boundary_vn` and `syllable_primitive` row passes; two rows that passed before now fail
on purpose (`beef`, `ax` read English) (0004).

## Cost

Level 1+ ties whose reading is an attested Vietnamese syllable typed the usual way, but not in
`vi_common.txt`, now default to English without Vietnamese context (138 rows). Level 1:
`ax` ã, `beef` bề, `beer` bể, `boom` bôm, `boost`/`boots` bốt, `bow` bơ, `burn` bủn, `caps` cáp,
`cast`/`cats` cát, `chats` chát, `chest` chét, `choir` chỏi, `chops` chóp, `corn` cỏn, `cups` cúp,
`dawn` dăn, `dons` dón, `dots` dót, `dust` dút, `gaps`/`gasp` gáp, `guts` gút, `hero` hẻo, `horn`
hỏn, `kits` kít, `lawn` lăn, `lays` láy, `lens` lén, `lest` lét, `lips`/`lisp` líp, `loops` lốp,
`lungs` lúng, `lust` lút, `mans` mán, `mist` mít, `nest`/`nets` nét, `noon` nôn, `norm` nỏm,
`owns` ớn, `quest` quét, `rats` rát, `reset` rết, `roof` rồ, `roots` rốt, `rows` rớ, `sir` sỉ,
`songs` sóng, `sons` són, `taxi` tãi, `thee` thê, `tips` típ, `toast` toát, `toes` tóe, `tops`
tóp, `tore` tỏe, `towns` tớn, `ups` úp. Level 2 (rarer English), among others: `chef` chè, `cur`
củ, `gays` gáy, `hams` hám, `lax` lã, `mar` mả, `rams` rám, `raps` ráp, `sofa` sòa, `sown` sơn,
`tans` tán, `taps` táp, `vest`/`vets` vét, `visa` vía. After a clear Vietnamese word
(any word with a diacritic) they still read as Vietnamese, the backspace undo swaps the
reading, and the user force-Vietnamese file fixes a word for good. The generator's
`--nocapture` output (`REV level keys reading att= nat= -> en|vi`) lists every row for review.

Assumption: tone-key position is free, so the rule does not depend on whether the user puts
the tone right after the vowel (`tieesng`) or at the end (`tieengs`).
