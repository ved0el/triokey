# Third-party notices

TrioKey itself is MIT-licensed (`LICENSE`). It contains or uses the following third-party
material.

## Code compiled into `triokey.exe`

- **vi** 0.8.0 (https://github.com/ZeroX-DG/vi-rs), MIT License, Copyright 2020, Hung Nguyen.
  Used as a crates.io dependency for the Telex transform (`crates/triokey-engine/src/telex.rs`).
  Its own dependencies compiled in: `log` (MIT OR Apache-2.0), `nom` (MIT), `memchr`
  (Unlicense OR MIT), `phf` and `phf_shared` (MIT), `siphasher` (MIT OR Apache-2.0),
  `smallvec` (MIT OR Apache-2.0).
- **minisign-verify** 0.2.5 (https://github.com/jedisct1/rust-minisign-verify), MIT License,
  Copyright (c) 2019-2025 Frank Denis. Verifies the signature of a downloaded update
  (`crates/triokey/src/update.rs`).
- **windows-sys** 0.59 (https://github.com/microsoft/windows-rs), MIT OR Apache-2.0,
  Copyright (c) Microsoft Corporation. Bindings to the Windows API, with `windows-targets` and
  `windows_x86_64_gnu` 0.52 (import libraries), same licence.

## Build-time and tooling only (no code in the exe)

- **embed-manifest** 1.5.1 (https://codeberg.org/carey/embed-manifest), MIT License,
  Copyright (c) 2021, 2022, 2025, 2026 Carey Evans. Build-time dependency of `crates/triokey`
  (`build.rs`): it generates the application manifest, which `build.rs` writes into
  `triokey.exe` as a resource.
- **minisign** 0.7.9 (https://github.com/jedisct1/rust-minisign), MIT License, Copyright (c)
  2019-2024 Frank Denis, Copyright (c) 2017 Daniel Rangel. Used by the release tool
  `crates/xtask` and by the tests only.

## Data

- **FineWeb-2** (https://huggingface.co/datasets/HuggingFaceFW/fineweb-2, config `vie_Latn`)
  and **FineWeb** (https://huggingface.co/datasets/HuggingFaceFW/fineweb, config
  `sample-10BT`), published by the HuggingFaceFW organisation on Hugging Face, Open Data
  Commons Attribution License (ODC-By) v1.0 (use also subject to Common Crawl's Terms of
  Use, as the dataset cards state). A sample of documents is read by `tools/corpus/fetch.py`;
  only token counts derived from it are tracked, in `crates/triokey-engine/data/freq_vi.tsv`
  and `freq_en.tsv` (built by `tools/corpus/build_freq.py`). No text from either dataset is
  in the repository.
- **SCOWL** (Spell Checker Oriented Word Lists), Copyright 2000-2018 Kevin Atkinson and
  others; permissive licence reproduced in `crates/triokey-engine/data/SCOWL-Copyright.txt`.
  Used only for `crates/triokey-engine/data/en_words.txt`, taken from VieLotus-IM's
  `scowl-en.txt` (below). No script in this repository regenerates that file.
- **VieLotus-IM** (https://github.com/stevedat/VieLotus-IM), MIT License, Copyright (c) 2026
  Trần Phát Đạt (Steve Dat); licence text reproduced in
  `crates/triokey-engine/tests/data/VieLotus-LICENSE.txt` (copied unchanged from its `LICENSE`
  at commit `3519761`). Read as a reference; credited here wherever behaviour or data is
  adapted from it:
  - its `scowl-en.txt` (SCOWL word list) is the source of `en_words.txt`, and words from its
    `Sources/VieLotusCoreWindows/lexicon/04_unconditional_english_words.txt` and
    `05_context_english_collision_words.txt` are merged into `en_words.txt`;
  - its `Sources/VieLotusCLI/Resources/test_suite.csv` is copied unchanged to
    `crates/triokey-engine/tests/data/vielotus_test_suite.csv` for the regression report in
    `crates/triokey-engine/tests/vielotus.rs`;
  - the syllable list of `Sources/VieLotusCore/VietnameseSyllableDictionary.swift` is copied
    to `crates/triokey-engine/tests/data/vi_attested.txt` (plus `tý`). It is not compiled into
    the exe; `tools/corpus/fetch.py` reads it to pick lines with English words for the mixed
    web benchmark set.

`crates/triokey-engine/data/en_tech.txt` and the line sets in
`crates/triokey-engine/tests/data/mixed/` and `final/` were written for this project and are
MIT-licensed like the rest of the repository.

No source, data tables or tests are taken from VKey or Vikey (GPL-3.0).
