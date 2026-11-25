# Third-party notices

- **vi** 0.8.0 (https://github.com/ZeroX-DG/vi-rs), MIT License, (c) Nguyễn Việt Hưng.
  Used as a crates.io dependency for the Telex transform (`crates/triokey-engine/src/telex.rs`).
- **VieLotus-IM** (https://github.com/stevedat/VieLotus-IM), MIT License, Copyright (c) 2026
  Trần Phát Đạt (Steve Dat); licence text reproduced in
  `crates/triokey-engine/tests/data/VieLotus-LICENSE.txt` (copied unchanged from its `LICENSE`
  at commit `3519761`). Read as a reference; credited here wherever behaviour or
  test data is adapted from it. Its `scowl-en.txt` (SCOWL word list with size levels) is the
  input for `crates/triokey-engine/data/en_words.txt` and `ambiguous_defaults.txt`
  (see `docs/decisions/0002-lexicon.md`). Its `Sources/VieLotusCLI/Resources/test_suite.csv`
  is copied unchanged to `crates/triokey-engine/tests/data/vielotus_test_suite.csv` for the
  regression report in `crates/triokey-engine/tests/vielotus.rs`. Words from its
  `Sources/VieLotusCoreWindows/lexicon/04_unconditional_english_words.txt` and
  `05_context_english_collision_words.txt` are merged into `en_words.txt`, and the syllable
  list of `Sources/VieLotusCore/VietnameseSyllableDictionary.swift` is copied to
  `crates/triokey-engine/tests/data/vi_attested.txt` (generator input; see
  `docs/decisions/0005-tie-defaults.md`).
- **SCOWL** (Spell Checker Oriented Word Lists), Copyright 2000-2018 Kevin Atkinson and
  others; permissive licence reproduced in `crates/triokey-engine/data/SCOWL-Copyright.txt`.
- **embed-manifest** 1.5.1 (https://codeberg.org/carey/embed-manifest), MIT License,
  Copyright (c) 2021, 2022, 2025, 2026 Carey Evans. Build-time dependency of `crates/triokey`
  only (`build.rs`): it writes the application manifest into `triokey.exe` as a resource; none
  of its code is part of the exe.
- **minisign-verify** 0.2.5 (https://github.com/jedisct1/rust-minisign-verify), MIT License,
  Copyright (c) 2019-2025 Frank Denis. Compiled into `triokey.exe`: verifies the signature of
  a downloaded update (`crates/triokey/src/update.rs`).
- **minisign** 0.7.9 (https://github.com/jedisct1/rust-minisign), MIT License, Copyright (c)
  2019-2024 Frank Denis, Copyright (c) 2017 Daniel Rangel. Used by the release tool
  `crates/xtask` and by the tests only; none of its code is part of the exe.

No source, data tables or tests are taken from VKey or Vikey (GPL-3.0).
