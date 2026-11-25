# 0004: VieLotus regression baseline

Date: 2026-11-25. Status: measurement (does not gate the build).

(The plan calls this file `0003-regression-baseline.md`; `0003` was already taken by the
Task 4 engine edge-case notes, so it is numbered 0004.)

Suite: VieLotus-IM `Sources/VieLotusCLI/Resources/test_suite.csv` (MIT), copied unchanged to
`crates/triokey-engine/tests/data/vielotus_test_suite.csv`. Run:

    mise x -- cargo test -p triokey-engine --test vielotus -- --ignored --nocapture

Each row whose `input` has no whitespace is typed as `input + " "` with `Config::default()` and
a fresh `Engine`; it passes when the screen equals `expected_output + " "`. An expected value
written `a / b` (VieLotus for "either is fine", e.g. `w / ư`) passes on either alternative.

## Result

**8361 / 9084 (92.0%)** after decision 0005 (tie defaults), up from **8009 (88.2%)** at the
first measurement; 9 rows skipped for containing spaces. Two rows that passed before now fail,
both on purpose (English wins a level-1 tie): `beef` (was `bề`) and `ax` (was `ã`).

| category | before | after | category | before | after |
|---|---|---|---|---|---|
| mangled_garbage | 5235 | 5296/5319 | collision_vn_word_cased | 126 | 260/384 |
| letter_drop_ascii_cased | 1156 | 1156/1228 | collision_vn_word | 64 | 131/194 |
| letter_drop_ascii | 578 | 578/614 | inflection_trap_cased | 14 | 66/282 |
| collision_unchanged | 233 | 233/234 | inflection_trap | 8 | 34/141 |
| syllable_primitive | 186 | 186/186 | free_tone_position | 63 | 63/64 |
| passthrough | 127 | 127/128 | boundary_reset | 50 | 50/55 |
| tone_primitive | 32 | 31/42 | letter_drop_en_word(_cased) | 27 | 27/60 |
| collision_vn_syllable | 13 | 27/36 | tone_primitive_en_conflict | 12 | 12/17 |
| double_press_semantics | 15 | 15/16 | diacritic_primitive_*_en_conflict | 12 | 12/21 |
| syllable_primitive_en_conflict | 6 | 5/7 | mangled_garbage_cased | 4 | 4/8 |
| vn_* (basic, double, gi_qu, tone_placement, uow, w), boundary_vn, tone_placement_style, diacritic_primitive(_cased) | all | all pass | | | |

Review aid: `VIELOTUS_DUMP=<file>` writes every failing row to that file.

## Failure groups and why they stay

No group points to a bug in the Task 4 engine, so the engine was not changed.

1. **English/Vietnamese ties that stay Vietnamese without context** (`inflection_trap`,
   `collision_vn_*`): about 520 rows left in these categories (was about 810). Decision 0005 now defaults level 1+
   ties to `en`; what remains is on purpose:
   - everyday Vietnamese readings typed the usual way (`hats` = `hát`, `bans` = `bán`,
     `teen` = `tên`, `queen` = `quên`, `moon` = `môn`, `rust` = `rút`, `mats` = `mát`);
   - level-2 coda-less marked syllables (`bee` = `bê`, `bees` = `bế`, `moos` = `mố`), which
     `syllable_primitive` needs;
   - level-0 rows kept `vi` in 0002 (`sex`, `hes`, `cums`);
   - tech words outside SCOWL whose reading is a real Vietnamese syllable (`cors` = `có`,
     `uri` = `ủi`, `usr` = `ủ`, `max` = `mã`, `los` = `ló`) and VieLotus's synthetic inflection
     traps (`teest` = `tết`, `heest` = `hết`): not added to the English list (0005).
   (The first measurement said `defend`, `photos`, `teens`, `guns`, `trips` were missing from
   `ambiguous_defaults.txt`. That was wrong: they were listed as level-1/2 `vi` rows. The
   generator reproduced the old table byte for byte, so it was not stale.)
2. **Doubled tone key in non-words** (`letter_drop_ascii*`, `mangled_garbage`): `passwd` ->
   `paswd`, `terraform` -> `teraform`, `harrison` -> `harison`. Spec 5.4 "Phím dấu gõ lặp"
   accepts this cost (`Larry` -> `Lary`); the user force-English file fixes single words.
3. **English words with a doubled key kept whole** (`letter_drop_en_word*`): `less`, `off`,
   `pass` stay as typed; VieLotus wants `les`, `of`. Intended (spec, Task 4 tests).
4. **Rule-4 defaults `en`** (`*_en_conflict`, `has`, `of`, `is`, `as`, `us`, `if`): VieLotus
   expects Vietnamese without context; TrioKey defaults these to English (Task 4 test `has `).
5. **Bare vowel + mark not a syllable** (`tone_primitive`, `diacritic_primitive_*`): `aa`,
   `aaf`, `aw`, `dd` stay raw because `â`, `ầ`, `ă`, `đ` alone fail `is_syllable`.
6. **Code rules** (`boundary_reset`, `double_press_semantics`): `50%off`, `ddAAy` (mixed
   case), `os.path` (`os` is committed as a word before `.`, spec rule 1) follow spec 5.4.
7. **Valid Vietnamese readings of short keys** (`mangled_garbage_cased`, `passthrough`,
   `collision_unchanged`): `WA` -> `ƯA`, `momo` -> `môm`, `mono` -> `môn`.
8. **`qusy`** (`free_tone_position`, 1 row): `vi` 0.8.0 `compose` does not move a tone typed
   right after `qu` (`qusy` -> `qusy`; `quys` works). Telex-core limit from Task 1.
