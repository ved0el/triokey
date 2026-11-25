# 0002: English lexicon and English/Vietnamese tie defaults

Date: 2026-11-25. Status: accepted.

## Source

`scowl-en.txt` from VieLotus-IM (MIT), `Sources/VieLotusCore/Resources/English/`: 39,094
lines, each a one-digit SCOWL size level (0 = most common tier, 3,973 words; 1: 6,926;
2: 28,195) followed by a lowercase word. SCOWL licence copied to
`crates/triokey-engine/data/SCOWL-Copyright.txt`.

## Generator

`crates/triokey-engine/tests/gen_lexicon.rs`, an `#[ignore]`d test (kept inside the crate so it
can call `compose` and `is_syllable`, and so it runs with `cargo test`, not `cargo run`):

    TRIOKEY_SCOWL=<path>/scowl-en.txt TRIOKEY_VIELOTUS_LEXICON=<VieLotus>/Sources/VieLotusCoreWindows/lexicon \n      mise x -- cargo test -p triokey-engine --test gen_lexicon -- --ignored

(`TRIOKEY_VIELOTUS_LEXICON` and the level 1+ rule were added by 0005.)

Add `--nocapture` to print `level keys reading` for each tie row (review aid).
Output is deterministic; rerunning it reproduces both files byte for byte.

- `data/en_words.txt`: every SCOWL word (all levels, all `[a-z]+`, 39,094 words, under the
  50k cap), sorted, one per line.
- `data/ambiguous_defaults.txt`: `word<TAB>en|vi`. A word is listed when
  `compose(word, Old)` differs from the word and `is_syllable` accepts it (`this` -> `thí`,
  `has` -> `há`). Words whose Telex reading equals the keys (`an`, `to`) are left out: both
  readings look the same, so there is nothing to decide. 651 rows, ordered by level then
  alphabetically, so the most common words come first.
- Default: `en` for level 0, `vi` otherwise (superseded for level 1+ by 0005). Level 0 (~4k words) stands in for the plan's
  "top ~2000": SCOWL ships size tiers, not ranks.
- Hand review of all 138 level-0 rows (more than the plan's 50; the table sorts by level then
  alphabet, so "first 50" would have stopped at `his`) flipped three to `vi`: `cums` (cúm),
  `hes` (hé), `sex` (sẽ). Criterion: the Vietnamese word is everyday and typed exactly this
  standard way, and the English word is rare, not a real word, or far less frequent.
  Considered and kept `en`: `chaos`, `car`, `these`, `its`, `own`, `did`. Kept in
  `VI_OVERRIDES` in the generator.

## Runtime (`src/lexicon.rs`)

Both files embedded with `include_str!`, parsed lazily into `HashSet<&'static str>` /
`HashMap<&'static str, Lang>` behind `OnceLock`. Everything from `#` to end of line is a comment; blank lines are
skipped (all files).

- `tie_default` of keys not in the table returns `Vi` (not a tie, so rule 3 applies).
- User force files: entries are the typed keys (`this`, not `thí`), trimmed and lowercased. `forced` returns the file's language.
  `is_english` follows the force files too (force-en adds a word, force-vi removes one), so
  "always English" also fixes the doubled-key rule (`Larry`, spec 5.4). A word in both
  files ends up English.

## Known limits

- `is_syllable` accepts phonotactically valid but unused syllables (`bỏe`, `choén`), so
  the tie table has rows whose Vietnamese reading is not a real word.
- (Fixed by 0005.) Ties outside level 0 defaulted to `vi`, so after a neutral previous word some ordinary
  English words turn into non-words: `reset` -> `rết`, `virus` -> `víu`, `taxi` -> `tãi`,
  `maps` -> `máp`, `texts` -> `tét`. This follows the plan's "top ~2000 -> en" rule; to
  revisit with Task 4 / AT4 (e.g. default `en` when the reading is not a real word).
