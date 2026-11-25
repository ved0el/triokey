# Contributing to TrioKey

Thanks for looking. TrioKey is a small pre-release project; reports about words that convert wrongly are the most useful contribution right now.

## Build and test

You need a stable Rust toolchain. With [mise](https://mise.jdx.dev), `mise install` in the repository sets up the stable GNU toolchain named in `mise.toml`; prefix the commands below with `mise x --`.

```sh
cargo build --release -p triokey      # the tray app: target\release\triokey.exe
cargo test --workspace                # all tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all                       # CI runs it with --check; settings in rustfmt.toml
```

CI (`.github/workflows/ci.yml`) runs these four on every pull request, on Windows with the same GNU toolchain. `crates/xtask` is release tooling (update-signing keys and signatures, see [docs/RELEASING.md](docs/RELEASING.md)); it is not part of the app.

The engine (`crates/triokey-engine`) has no operating-system calls. The app crate (`crates/triokey`) is Windows only.

The report against VieLotus-IM's regression suite is a measurement, not a gate:

```sh
cargo test -p triokey-engine --test vielotus -- --ignored --nocapture
```

Quit every other Vietnamese input method before running `triokey.exe`; two of them will rewrite each other's text.

## Reporting a word that converts wrongly

Open an issue with these four things:

1. **Raw keys typed**, exactly, including the key that ended the word (for example `songs` then space).
2. **Expected** text.
3. **Actual** text.
4. **The previous word** as it appeared on screen (it decides ambiguous words), or "start of line".

Also useful: the app you typed in, the keyboard layout, and whether a CJK IME was on.
Please do not paste real passwords or private text; a made-up example that shows the same problem is enough.

## Changing the engine

- Add a test case for the behaviour you change (`crates/triokey-engine/tests/engine.rs` is a table of `keys => screen`).
- A rule change that trades one group of words against another needs a short note in `docs/decisions/`.
- Keep the engine free of OS calls, and the app free of anything that writes typed text to disk. The only network code is the updater (`crates/triokey/src/update.rs`): plain GETs of two files at fixed GitHub URLs (the signature, then the exe, each through GitHub's redirects) that send nothing about the user. Do not add another network call, and do not add data to those requests.

## The no-GPL-code rule

TrioKey is MIT-licensed. Do not copy source code, data tables or test files from GPL projects (VKey, Vikey, OpenKey, UniKey and others) into this repository, and do not translate their code line by line. Reading their documentation and describing behaviour is fine. Material from MIT or similarly permissive projects is welcome when it is credited in `NOTICE.md`.

By contributing you agree that your contribution is released under the MIT licence.

## Tiếng Việt

Cảm ơn bạn đã ghé qua. Cách đóng góp có ích nhất lúc này là báo những từ bị chuyển sai. Khi mở issue, xin ghi đủ bốn thứ: (1) đúng các phím đã gõ, kể cả phím kết thúc từ; (2) kết quả bạn mong đợi; (3) kết quả thực tế; (4) từ đứng ngay trước đó trên màn hình, vì nó quyết định các từ có hai cách đọc. Đừng dán mật khẩu hay nội dung riêng tư; một ví dụ tự đặt là đủ. Build bằng `cargo build --release -p triokey`, chạy test bằng `cargo test --workspace`. TrioKey dùng giấy phép MIT, nên xin đừng chép mã nguồn, bảng dữ liệu hay file test từ các dự án GPL (VKey, Vikey, OpenKey, UniKey…) vào repo; đọc tài liệu của họ rồi mô tả lại hành vi thì được.
