# Contributing to TrioKey

Thanks for looking. TrioKey is a small, early project; reports about words that convert wrongly are the most useful contribution right now. How the program is built, and why, is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Build and test

You need the stable GNU Rust toolchain (`stable-x86_64-pc-windows-gnu`); it is the only supported one, and the app's build script writes its resources for that toolchain only. With [mise](https://mise.jdx.dev), `mise install` in the repository sets up the toolchain named in `mise.toml`; prefix the commands below with `mise x --`.

```sh
cargo build --release -p triokey      # the tray app: target\release\triokey.exe
cargo test --workspace                # all tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all                       # CI runs it with --check; settings in rustfmt.toml
```

CI (`.github/workflows/ci.yml`) runs these four gates on every push to `main` and every pull request, on Windows with the same GNU toolchain and `--locked`: `cargo fmt --all -- --check`, clippy as above, `cargo test --workspace`, and the release build. A change should pass all four locally first. `crates/xtask` is release tooling (update-signing keys and signatures, see [docs/RELEASING.md](docs/RELEASING.md)); it is not part of the app.

The engine (`crates/triokey-engine`) has no operating-system calls. The app crate (`crates/triokey`) is Windows only.

Quit every other Vietnamese input method before running `triokey.exe`; two of them will rewrite each other's text.

## Measuring the engine

Two reports measure the engine; neither is a gate.

**Typing benchmark** (`crates/triokey-engine/tests/realtext.rs`). A simulated typist types whole lines, each a message as its author wants it on screen, into a fresh engine, and every word on the final screen is compared with the wanted one:

```sh
cargo test -p triokey-engine --release --test realtext -- --ignored --nocapture
```

By default it runs the dev split of `tests/data/mixed/` and, if present, of the web sets under `.corpus/bench` (made by `tools/corpus/fetch.py`, see below). Environment variables: `REALTEXT_SETS` (`mixed`, `web`, `final`, comma separated), `REALTEXT_SPLIT` (`dev`, `heldout`, `all`), `REALTEXT_LEARN=1` (remembering on), `REALTEXT_DUMP=<file>` (failures as TSV, never for `final`), `REALTEXT_JSON=<file>` (summary).

Columns, per set and pooled (`own` = all `mixed` sets):

| Column | Meaning |
|---|---|
| `lines`, `skip` | lines typed; lines skipped (empty, untypable, or with characters other than printable ASCII and Vietnamese letters) |
| `clusters`, `wrong`, `wrong%` | words (text between spaces), words that came out wrong, and their share |
| `en_mangled`, `en_altered` | English that got Vietnamese marks; English changed but still ASCII |
| `vi_raw`, `vi_wrong` | Vietnamese left as plain keys; Vietnamese with the wrong marks |
| `line_shift` | words of a line whose word count changed (the whole line counts as wrong) |
| `flip` | wrong words that one Backspace right after the space would fix |
| `cost/100` | keystrokes spent fixing, per 100 words: 2 for a flip, otherwise deleting and retyping |
| `armed`, `armed/100` | correct words where one Backspace after the space would swap the word instead of deleting the space |
| `vi_lost/100` | `vi_raw` + `vi_wrong` per 100 words |
| `objective` | `cost/100 + 0.05 × armed/100`, the number tuning tries to lower |
| `repeat` | wrong words whose typed keys and wanted text were already wrong, and flip-fixable, earlier in the same set |
| `harm` | with `REALTEXT_LEARN=1` only: words wrong only because of a remembered entry |

More detail and the current figures: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), section 11.2.

**VieLotus-IM suite** (`crates/triokey-engine/tests/vielotus.rs`): 9,084 hard single words typed without context, a guardrail against regressions rather than a measure of real typing:

```sh
cargo test -p triokey-engine --release --test vielotus -- --ignored --nocapture
```

### Held-out and blind sets

Tune only on the dev split. Lines of `mixed` and `web` are split by a hash of the line, one in five held out; `tests/data/final/` is a blind set. For held-out and `final` runs, read the totals in the table only: do not read their failures, do not dump them, and do not change the engine or the data because of a single word seen there. The report prints its "top wrong" lists only for a `dev` run and never for a `final` set, and `REALTEXT_DUMP` is ignored on anything but a `dev` run. A set whose failures have been read is no longer blind.

## Word data

The engine's word data is in `crates/triokey-engine/data/` and is compiled into the exe:

- `freq_vi.tsv` and `freq_en.tsv`: token counts. To rebuild them, run `python tools/corpus/fetch.py` (samples FineWeb-2 and FineWeb through the Hugging Face datasets-server API into `.corpus/`, resumable) and then `python tools/corpus/build_freq.py` (writes the two tables from `.corpus/counts/`). Both scripts use the Python standard library only. Re-run the benchmark afterwards and say in the pull request how the figures moved.
- `en_words.txt` (SCOWL) and `en_tech.txt` (developer terms written for this project). No script in the repository regenerates `en_words.txt`.

Corpus text is never committed: everything under `.corpus/` (downloaded rows, benchmark lines, token counts) and any failure dump stays local and is gitignored. Only the derived counts in `data/` are tracked. Sources and licences are in [NOTICE.md](NOTICE.md).

## Reporting a word that converts wrongly

Open an issue with these four things:

1. **Raw keys typed**, exactly, including the key that ended the word (for example `songs` then space).
2. **Expected** text.
3. **Actual** text.
4. **The line before the word**, as it appeared on screen, or "start of line". The words already on the line decide how a word with two readings comes out.

Also useful: whether one Backspace right after the word gave the right reading, the app you typed in, the keyboard layout, and whether a CJK IME was on.
Please do not paste real passwords or private text; a made-up example that shows the same problem is enough.

## Changing the engine

- Add a test case for the behaviour you change (`crates/triokey-engine/tests/engine.rs` is a table of `keys => screen`).
- No per-word fixes in code: no lists of special words, no `if word == …`. Fix a word by changing a general rule or the data it reads, and check with the benchmark that the change helps more words than it hurts.
- A rule change that trades one group of words against another needs its reasoning and its benchmark figures in the pull request, and an update to [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) (section 5) when it changes how the engine decides.
- Keep the engine free of OS calls. The app writes typed text to disk only as remembered words in `learned.txt` (typed keys and the word shown, after a Backspace swap the person kept); do not add any other place that stores typed text. The only network code is the updater (`crates/triokey/src/update.rs`): plain GETs of two files at fixed GitHub URLs (the signature, then the exe, each through GitHub's redirects) that send nothing about the user. Do not add another network call, and do not add data to those requests.

## Commits and releases

Commit messages are short, in the imperative, and say what the change does (for example `Release 0.1.1: app icon, injected-key guard, automatic releases`). User-visible changes not yet released are listed under `## Unreleased` at the top of [CHANGELOG.md](CHANGELOG.md).

Releases are made by the owner: a tag `vX.Y.Z` on a commit of `main` builds, signs and publishes that version with no manual approval. The steps are in [docs/RELEASING.md](docs/RELEASING.md).

Security problems: please report them privately, as described in [SECURITY.md](SECURITY.md), not in a public issue.

## The no-GPL-code rule

TrioKey is MIT-licensed. Do not copy source code, data tables or test files from GPL projects (VKey, Vikey, OpenKey, UniKey and others) into this repository, and do not translate their code line by line. Reading their documentation and describing behaviour is fine. Material from MIT or similarly permissive projects is welcome when it is credited in `NOTICE.md`.

By contributing you agree that your contribution is released under the MIT licence.

## Tiếng Việt

Cảm ơn bạn đã ghé qua. Cách đóng góp có ích nhất lúc này là báo những từ bị chuyển sai. Khi mở issue, xin ghi đủ bốn thứ: (1) đúng các phím đã gõ, kể cả phím kết thúc từ; (2) kết quả bạn mong đợi; (3) kết quả thực tế; (4) phần dòng đứng trước từ đó trên màn hình, hoặc ghi "đầu dòng", vì các từ đã gõ trên dòng quyết định từ có hai cách đọc ra thế nào. Nếu bấm Backspace một lần ngay sau từ đó mà ra đúng, cũng xin ghi lại. Đừng dán mật khẩu hay nội dung riêng tư; một ví dụ tự đặt là đủ.

Build bằng `cargo build --release -p triokey` với toolchain GNU (`stable-x86_64-pc-windows-gnu`), chạy test bằng `cargo test --workspace`; CI còn chạy `cargo fmt --all -- --check` và `cargo clippy --workspace --all-targets -- -D warnings`. Khi sửa engine, xin chạy thêm bài đo gõ (`realtext`) như mô tả ở trên, chỉ chỉnh theo phần dev; với tập held-out và tập `final` chỉ xem bảng tổng, không đọc từng lỗi. Không sửa riêng từng từ trong mã. Văn bản corpus không bao giờ được commit, chỉ có bảng đếm trong `data/`. Lý do thiết kế nằm ở [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

TrioKey dùng giấy phép MIT, nên xin đừng chép mã nguồn, bảng dữ liệu hay file test từ các dự án GPL (VKey, Vikey, OpenKey, UniKey…) vào repo; đọc tài liệu của họ rồi mô tả lại hành vi thì được. Lỗ hổng bảo mật xin báo riêng theo [SECURITY.md](SECURITY.md).
