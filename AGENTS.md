# AGENTS.md

This file provides guidance to coding agents working with code in this repository.

TrioKey is a Vietnamese Telex input method for Windows (plus an experimental macOS build that has never run on a Mac) for people who mix English, Vietnamese
and a CJK language. It registers no keyboard layout or text service: a low-level keyboard hook
rewrites the word being typed with injected text, decides English or Vietnamese per word without
a mode switch, and steps aside while a CJK IME is composing. The design and its reasons are in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md); read the section you are about to change.

## Commands

The only supported toolchain is `stable-x86_64-pc-windows-gnu` (pinned in `mise.toml`; the app's
`build.rs` refuses anything else). Prefix cargo with `mise x --` when using mise.

```sh
# the four gates CI runs (.github/workflows/ci.yml); all must pass before a change is done
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p triokey --release --locked

# one test file / one test
cargo test -p triokey-engine --test engine
cargo test -p triokey-engine --test engine readme_examples
cargo test -p triokey app::tests            # unit tests inside the app crate

# macOS app, type-checked from Windows (it cannot be linked or run here; needs
# `rustup target add aarch64-apple-darwin` once)
cargo check -p triokey-mac --target aarch64-apple-darwin
cargo clippy -p triokey-mac --target aarch64-apple-darwin --all-targets -- -D warnings

# measurement reports (ignored tests; neither is a gate)
cargo test -p triokey-engine --release --test realtext -- --ignored --nocapture
cargo test -p triokey-engine --release --test vielotus -- --ignored --nocapture
```

`realtext` takes `REALTEXT_SETS` (`mixed,web` default, `final`), `REALTEXT_SPLIT` (`dev` default,
`heldout`, `all`), `REALTEXT_LEARN=1`, `REALTEXT_JSON=<file>`, `REALTEXT_DUMP=<file>`. Its columns
are explained in `CONTRIBUTING.md`. The `web` sets exist only after `python tools/corpus/fetch.py`
has filled the gitignored `.corpus/` folder.

Do not start `triokey.exe` to test a change: it installs a system-wide keyboard hook, and a second
input method running next to it rewrites the same keys. Behaviour that needs the real program goes
into `docs/HANDS_ON_TEST.md` for a person to run.

## Architecture in brief

Four crates under `crates/`:

- **`triokey-engine`**: pure Rust, no OS calls. `Engine::key / boundary / backspace / reset` take
  one event and return `Pass` or one `Edit` (delete N characters, insert text) that touches only
  the word being typed. After every key the whole word is recomputed from its raw keys.
- **`triokey`**: the Windows tray app that feeds the engine and sends its edits.
- **`triokey-mac`**: the macOS menu bar app (experimental, ARCHITECTURE 12). The platform-free
  parts (`keymap`, `plan`, `source`, `text`, `diag`) are tested on every system; `src/mac/` is built
  only on macOS, and on Windows the binary is an empty stub.
- **`xtask`**: release tool (`keygen`, `sign`, `verify` for update signatures); not shipped.

How a word is decided (`engine.rs`, `lexicon.rs`; ARCHITECTURE 5.3): at the word boundary the raw
keys and their Telex composition are two candidates, scored with per-million rates from
`data/freq_vi.tsv` and `data/freq_en.tsv` (plus floors from `en_words.txt` and `en_tech.txt`) and
weighted by a per-line language mode that every committed word nudges. The loser stays one
Backspace away only when the decision was close (the "swap"). A swap the person keeps is
remembered in a shared `Learned` map and saved by the app as `learned.txt`. Authority order: the
user's `force_vi.txt` / `force_en.txt`, then remembered words, then scoring. Text that looks like
code (paths, flags, text after `.` `:` `\`, digits, mixed case) is left as typed.

Threads in the app (ARCHITECTURE 2): the **hook thread** owns the `Engine` and runs the keyboard
callback; the **watcher thread** (`ime_watch.rs`, 100 ms tick) reads IME state, password fields and
focus, and does timed work such as saving `learned.txt`; the **tray/UI thread** owns the icon, menu
and settings window; the **updater thread** checks GitHub releases. Settings live in `app.rs`,
which is also the only place that reads or writes files under `%APPDATA%\triokey`.

## Rules that are easy to break

- **Inside the hook callback** (`hook.rs` `kb_proc` and everything it calls): no file or network
  access, no blocking lock, nothing slow. Tables the engine needs must be parsed in the warm-up in
  `hook::spawn`, before the hook is installed.
- **The elevated copy** (admin mode) must never call `ShellExecute`, never create files under the
  user's folders, write there only through the checked open in `app.rs`, never turn keys injected
  by a lower-integrity process into text, and never install an update on its own
  (ARCHITECTURE 8). Changes near `admin.rs`, `app.rs` file access or `update.rs` need that section
  re-read.
- **The macOS tap callback** (`tap.rs` `callback` and everything it calls) fails open: it runs
  under `catch_unwind`, and a panic or an error disables the tap for good and passes the event
  unchanged. Never let a panic unwind into CoreGraphics or AppKit, never swallow a key on an error
  path. Same rules as the Windows hook callback: no file or network access, nothing slow.
- **On macOS everything runs on the main thread** (AppKit, the tap's run loop source, every TIS
  call, the engine). Do not add threads; TIS crashes off the main thread.
- **The macOS build has no file or network access** at all; its only output besides keys is the
  diagnostics text on the clipboard. Do not add either without changing ARCHITECTURE 12.
- **macOS CI uses `-p triokey-engine -p triokey-mac`, never `--workspace`**
  (`.github/workflows/macos.yml`): `crates/triokey/build.rs` refuses any toolchain but
  `x86_64-pc-windows-gnu`. The macOS workflow runs only on the `macos` branch and never releases.
- **No per-word fixes in engine code.** Change a general rule or the data it reads, and show with
  the benchmark that it helps more words than it hurts. Data files are not edited to fix one
  benchmark row.
- **Tune on the dev split only.** Never read, dump or act on individual failures of the held-out
  split or of `tests/data/final/` (the blind sets); use their totals only.
- **Corpus text is never committed.** Everything under `.corpus/` stays local; only derived counts
  in `crates/triokey-engine/data/` are tracked. No code, data tables or tests copied from GPL input
  methods (see `CONTRIBUTING.md`); third-party material is credited in `NOTICE.md`.
- **Typed text is stored in exactly one place**, `learned.txt` (Windows), and only for swapped
  words. The only network code is the signed update check in `update.rs`. Do not add either elsewhere.
- **Examples in the READMEs are tested.** Every "keys typed -> what appears" line in `README.md`
  and `README.vi.md` is asserted by `readme_examples` in `crates/triokey-engine/tests/engine.rs`;
  change them together. The two READMEs say the same things; update both.
- **Source comments cite the design by section**, as "docs/ARCHITECTURE.md, 5.3 Scoring model".
  When behaviour changes, update that section in the same change.
- **A tag is a release.** Pushing a tag `vX.Y.Z` on a commit of `main` builds, signs and publishes
  to every installed copy with no approval step (`docs/RELEASING.md`). Never create or push a tag
  unless the owner asks for a release. User-visible changes go under `## Unreleased` in
  `CHANGELOG.md`.
- **Line width is 160** (`rustfmt.toml`); the code is written dense on purpose, keep the style of
  the file you edit.
