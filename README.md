<p align="center"><img src="https://raw.githubusercontent.com/ved0el/triokey/main/assets/brand/icon-512.png" width="96" alt="TrioKey"></p>

# TrioKey

**English** · [Tiếng Việt](README.vi.md)

[![CI](https://github.com/ved0el/triokey/actions/workflows/ci.yml/badge.svg)](https://github.com/ved0el/triokey/actions/workflows/ci.yml)

A Vietnamese input method for Windows, made for people who type English, Vietnamese and a CJK language (Chinese, Japanese or Korean) every day. In everyday mixed typing you do not toggle Vietnamese on or off: the only switch you touch is your CJK IME's own on/off.

<!-- TODO: demo GIF goes here (docs/assets/demo.gif): typing English, Vietnamese and Japanese in one line without touching a Vietnamese toggle -->

```text
keys typed          what appears
tieesng vieejt␣     tiếng việt
windows␣            windows
songs␣              songs        (valid both ways: English by default)
this␣⌫              thí          (Backspace right after the word flips the reading)
```

Early version 0.1.0: a signed download exists, but much of the Windows app, including the updater that is on by default, has not been run on a real machine. Read [Status](#status) first.

## Why

TrioKey exists because of five daily annoyances. The first two are the reason it was written; the other three are limits it keeps to.

**1. Two toggles for one intention.**
Before: you are typing Vietnamese and want a Japanese word. You turn hiragana on, and you also turn Vietnamese mode off, because otherwise Telex mangles the romaji. Then both again on the way back.
With TrioKey: you turn hiragana on, type, and turn it off. TrioKey reads whether the CJK IME is actually open in the window you are typing in and steps aside by itself. When the IME is closed (direct input), it types Vietnamese, even under the Japanese keyboard layout.

**2. Switching between English and Vietnamese mode.**
Before: in Vietnamese mode, `terrible` and `windows` come out as nonsense, so you hit a toggle every few words.
With TrioKey: there is no mode. Each word is decided on its own. Keys that form a valid Vietnamese syllable become Vietnamese; anything else is put back as you typed it (one known exception: a doubled tone key in a word outside the English word list collapses to one, see [Status](#status)). Words that are valid both ways (`songs` / `sóng`) are decided by a word list and by the word before; one Backspace flips the guess for a word that has two readings.

**3. Input methods that change your Windows keyboard setup.**
TrioKey adds no keyboard layout, registers no text service (TSF) and leaves your input-language list alone. It is a small tray app: a keyboard hook (plus a mouse hook that only notes clicks) and injected text.

**4. Admin windows.**
A normal-privilege input method cannot type into elevated windows. TrioKey has an optional admin mode: one UAC prompt, a logon task, and the exe copied to a folder only administrators can write. (Written, but not yet run on a real machine; see [Status](#status).)

**5. Features you never asked for.**
No macros, no per-app memory, no cloud, no telemetry, no key logging. The only network use is a check for signed updates on GitHub, and you can switch it off. One small settings window and a plain-text config file.

## Status

**Early version 0.1.0. Please read this before trying it.**

- 0.1.0 is on the [releases page](https://github.com/ved0el/triokey/releases/latest), built and signed by CI. You can also build from source.
- The updater (signed downloads from this project's GitHub releases) is in the released exe and in builds from `main`, and is on by default, but it has never been run on a real machine (AT10 in [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) is still open). If you would rather not have an untested updater replace the exe, untick `Tự động cập nhật` in Settings.
- Tested on Windows 11 only; Windows 10 is untried.
- Telex only. No VNI yet.
- Japanese: an earlier build was hand-tested with Microsoft IME in Notepad and Brave, and passed AT1 and AT2 of [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) (the per-step result columns there were not filled in). The current build has not been re-run.
- Chinese and Korean: only the decision rule (IME open and in native mode) is unit-tested, with made-up values. Whether those IMEs report their state correctly has not been tried.
- **Never run on a real machine yet**, only compiled and unit-tested where possible: the current tray icon and menu, the app icon, the hotkey presets, the settings window, start-with-Windows, admin mode, the updater, the uninstall steps and the skipping of Win32 password fields. The sections below describe them as designed.
- The interface (tray menu, tooltip, settings, messages) is in Vietnamese only.
- English/Vietnamese auto-detection scores **92.0% (8,361 / 9,084)** on [VieLotus-IM](https://github.com/stevedat/VieLotus-IM)'s public regression suite, with every row typed without context (see [docs/COMPARISON.md](docs/COMPARISON.md) for how that differs from VieLotus-IM's own run). The known misses are described in [docs/decisions/0004-regression-baseline.md](docs/decisions/0004-regression-baseline.md) and [0005-tie-defaults.md](docs/decisions/0005-tie-defaults.md). Two you will meet: a doubled tone key (`s f r x j`) in a word outside the English word list loses one letter (`Larry` → `Lary`, `passwd` → `paswd`, `terraform` → `teraform`), and some short words that are real in both languages go the wrong way without context. Both can be fixed per word with `force_en.txt` / `force_vi.txt` (see [Usage](#usage)).
- Terminals get no special handling.
- Password fields in browsers and Electron apps cannot be detected (classic Win32 password fields are meant to be skipped; not yet verified by hand). See [Privacy and security](#privacy-and-security).
- Fields that insert suggestions after the cursor (browser address bar, Excel cells) get no special handling yet.
- Some apps hide the IME state. There TrioKey falls back to "step aside whenever the layout is CJK".
- macOS and Linux are planned, not started.

## How it behaves

| Keyboard layout of the window | CJK IME state | What your keys do | Tray |
| --- | --- | --- | --- |
| English, or any non-CJK layout | (not relevant) | Telex Vietnamese; English words stay as typed | `V` |
| Japanese | closed (direct input) | Telex Vietnamese; English words stay as typed | `V` |
| Japanese | open (hiragana, katakana…) | every key goes straight to the IME, untouched | `E` |
| Chinese or Korean † | closed, or open in alphanumeric mode | Telex Vietnamese; English words stay as typed | `V` |
| Chinese or Korean † | open in native mode | every key goes straight to the IME, untouched | `E` |
| CJK, in an app that hides the IME state | unknown | TrioKey steps aside | `E` |
| any, after you switch TrioKey off | (not relevant) | every key passes through | `E` |

† Untested with a real IME: this is the coded rule, checked only by unit tests. The Japanese rows were hand-tested on an earlier build (see [Status](#status)).

TrioKey does not change the keyboard layout for you. "One switch" holds when you stay on the CJK layout and use the IME's own on/off; if you sit on the English layout, you still switch layouts to reach Japanese, you just no longer toggle Vietnamese.

### Typed examples

Each line is checked by the engine's test suite (`crates/triokey-engine/tests/engine.rs`). `␣` is the space bar, `⌫` is Backspace.

```text
keys typed                 what appears
tieesng vieejt␣            tiếng việt
windows␣                   windows
terrible␣                  terrible
DDuowngf␣                  Đường
baajt/taawst␣              bật/tắt
server cos␣                server có
my_var␣   obj.dir␣         my_var   obj.dir      (code is left alone)
songs␣                     songs                 (ambiguous: English by default)
tooi thaasy songs␣         tôi thấy sóng         (same keys after Vietnamese words)
this␣                      this
this␣⌫                     thí                   (one Backspace flips the reading)
```

## Comparison

As of November 2026. This is the short version; sources, commit ids and the things the other projects do better are in [docs/COMPARISON.md](docs/COMPARISON.md).

| | TrioKey | [VKey](https://github.com/phatMT97/VKey) | [Vikey](https://github.com/tonyminibot/Vikey) (Windows) | [VieLotus-IM](https://github.com/stevedat/VieLotus-IM) (Windows) |
| --- | --- | --- | --- | --- |
| Steps aside for a CJK IME | by the IME's open/closed state, per window | by the layout's language (Japanese, Chinese, Korean) | by the layout's language (anything other than English or Vietnamese) | is its own entry in the Windows input list, next to the CJK IME |
| English words while typing Vietnamese | decided per word, no mode | V/E mode with a toggle; a spell check restores words that are not Vietnamese | automatic: fixes the word and switches the mode | automatic |
| Windows integration | hook + injected text, nothing registered by default | hook + injected text; optional TSF component | hook + injected text | TSF text service, added to the input list |
| Input styles | Telex | Telex, VNI, Simple Telex, combined | Telex, Simple Telex | Telex, VNI |
| Macros | no | yes | no | yes |
| Platforms | Windows 11 (Windows 10 untried) | Windows 10/11 | macOS, Windows, Linux, iOS, Android | macOS, Windows, Linux, web, iOS, Android |
| Releases | 0.1.0 (GitHub, signed by CI) | yes (winget) | numbered versions shipped (its `DECISIONS.md`); Windows download not checked | yes (Microsoft Store) |
| Licence | MIT | GPL-3.0; an optional closed-source spell-check library is outside it (the Classic edition is fully open source) | GPL-3.0 | MIT |

All three are further along than TrioKey and are already distributed to users; TrioKey learned from each of them. UniKey, EVKey and OpenKey, the long-standing choices, are built around a Vietnamese/English mode with a toggle; they were not read for this comparison, so whatever else they do is not described here.

## Install

There is no installer. Releases are built and signed by CI from a version tag ([docs/RELEASING.md](docs/RELEASING.md)).

1. **Quit every other Vietnamese input method first** (UniKey, EVKey, OpenKey, VKey, Vikey…). Two of them running together will rewrite each other's text.
2. Download `triokey-X.Y.Z-windows-x64.zip` from the [latest release](https://github.com/ved0el/triokey/releases/latest) and unpack it into a folder of its own, or build from source (below).
3. Run `triokey.exe`. No window opens; a `V` appears in the system tray (possibly behind the `^` overflow arrow).

### Build from source

1. Install Rust. Either:
   - [mise](https://mise.jdx.dev): run `mise install` in the repository (it reads `mise.toml`, which selects the stable GNU toolchain, so Visual Studio Build Tools are not needed), or
   - [rustup](https://rustup.rs) with the `stable-x86_64-pc-windows-gnu` toolchain. The default MSVC toolchain does not build TrioKey: its build script writes the exe's resources (icon and manifest) for the GNU toolchain only.
2. Build:

   ```sh
   git clone https://github.com/ved0el/triokey
   cd triokey
   cargo build --release -p triokey        # with mise: mise x -- cargo build --release -p triokey
   ```

3. The exe is `target\release\triokey.exe`. It carries the same update key as a release, so it checks for updates too.

The exe is not code-signed. SmartScreen or antivirus software may warn about it, as they do for many keyboard-hook programs. The `triokey.exe.minisig` published next to each released exe can be checked by hand with [minisign](https://jedisct1.github.io/minisign/) and the public key in `crates/triokey/update-pubkey.txt`; that is the same check the built-in updater makes.

## Usage

The interface is in Vietnamese only, so the Vietnamese labels are given next to the English names. The tray, hotkey and settings behaviour below is as designed; it has not been run on a real machine yet (see [Status](#status)).

- **Tray icon.** `V` means TrioKey is typing Vietnamese. `E` means keys pass through, either because you switched it off or because it is stepping aside for a CJK IME; the tooltip says which (`Tiếng Anh — đã tắt` = switched off, `Tiếng Anh — đang nhường IME Nhật` = stepping aside for the Japanese IME; `IME Trung` and `IME Hàn` are Chinese and Korean).
- **Left click** the icon to switch TrioKey on or off. **Right click** for the menu: `Tiếng Việt` (on/off), `Cài đặt…` (Settings), `Thoát` (Quit).
- **Hotkey** for on/off: Ctrl+Shift (default; press and release with no other key), Alt+Z, Ctrl+Space, or none (`Không dùng`).
- **Settings** (`Cài đặt…`): on/off (`Bật gõ tiếng Việt`), hotkey (`Phím tắt`), tone placement (`Kiểu bỏ dấu`: `hòa, thúy` or `hoà, thuý`), keep English words as typed (`Tự giữ nguyên từ tiếng Anh`), step aside for CJK IMEs (`Nhường khi IME Nhật/Trung/Hàn đang bật`), start with Windows (`Khởi động cùng Windows`), admin mode (`Chạy quyền admin (gõ được trong cửa sổ admin)`), automatic updates (`Tự động cập nhật`) with a check-now button (`Kiểm tra bản mới`), and a button that opens the config folder (`Mở thư mục cấu hình…`). Changes apply at once.
- **Updates.** With `Tự động cập nhật` on (the default), TrioKey asks GitHub once a day whether a newer signed version exists (the first try comes about a minute after it starts; a check that gets no answer, for example offline, is tried again an hour later). A normal copy downloads it, verifies the signature and replaces its own exe; a notification says `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau` (updated, active from the next start), and the previous exe stays next to it as `triokey.old.exe` until that next start. The admin-mode copy never installs by itself: it downloads the new exe into memory, verifies the signature, shows `Có bản X.Y.Z — bấm để cập nhật` as a notification and as a tray menu item, and updates when you click one of them. If TrioKey cannot write to its own folder, or its exe has been renamed to something other than `triokey.exe`, it only says that a new version exists and where to download it. A version is announced only after its signature has been verified. Not run on a real machine yet.
- **Config** lives in `%APPDATA%\triokey\config.txt` (`key = value` lines).
- **Force a word.** Create `force_en.txt` (always English) or `force_vi.txt` (always Vietnamese) in the same folder. One word per line, written as the keys you type (`this`, not `thí`); `#` starts a comment. They are read at startup, so restart TrioKey after editing.
- **Backspace flips a guess.** Right after a word is committed (space, punctuation), the first Backspace swaps the reading of an ambiguous word (`this␣⌫` → `thí`) or of a word changed by the doubled-key rule (`mixx␣` → `mix`, then `⌫` → `mixx`). A second Backspace deletes as usual. For every other word, Backspace is a plain Backspace.
- **Telex**: inside a word, `aa aw ee oo ow uw dd` → `â ă ê ô ơ ư đ` (`caan` → `cân`, `ddi` → `đi`); typed alone, `aa`, `aw` and `dd` stay as typed; `s f r x j` → tone marks; `z` removes the tone; `w` alone → `ư`. The tone key may come right after the vowel or at the end of the word (`tieesng`, `tieengs` → `tiếng`).

## Uninstall

These steps follow the code; they have not been run on a real machine yet. If one fails, the manual commands (`schtasks /Delete`, `reg delete`) are in the last section of [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) (in Vietnamese).

1. If you turned on **admin mode** or **start with Windows**, undo them in Settings, in this order:
   1. Untick admin mode (`Chạy quyền admin (gõ được trong cửa sổ admin)`). This removes the `TrioKey` logon task and the copy in `%ProgramFiles%\TrioKey`. If you do it from the admin copy itself, Windows removes that folder at the next restart; quit that copy and start the normal one before the next step.
   2. Then untick start with Windows (`Khởi động cùng Windows`); the box is greyed out while admin mode is on. Turning admin mode off from the normal copy puts the Run value back if it was set before, so look at this box even if you did not tick it lately. Unticking it removes the `TrioKey` value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
2. Quit from the tray menu (`Thoát`).
3. Delete the folder you unpacked, built or copied TrioKey into, and `%APPDATA%\triokey` if you also want the config gone.

If you never turned on either option, steps 2 and 3 are the whole uninstall. TrioKey registers nothing else: no keyboard layout, no language-list entry, nothing under `HKCU\Keyboard Layout`.

## Privacy and security

What it does:

- Installs a low-level keyboard hook (`WH_KEYBOARD_LL`). Like every hook-based input method, it therefore sees every key you press while it runs.
- Installs a low-level mouse hook (`WH_MOUSE_LL`). It uses button presses only, to know that the caret may have moved and that the IME may have been toggled.
- Listens for foreground and focus changes (two WinEvent hooks) and reads the time of the last input.
- Sends Backspaces and Unicode text to the focused window with `SendInput`.
- Reads the focused window's keyboard layout and IME open/closed state.
- Reads its config and the two force files, creates `config.txt` on first run and rewrites it when you change a setting.
- Reads two values under `HKCU`: the Windows light/dark setting (for the icon colour) and its own Run value.
- If `Tự động cập nhật` is on: once a day (first try about a minute after start; a check that gets no answer, for example offline, is tried again an hour later), fetches the signature file of the latest release (a few hundred bytes) from `github.com`. Only when that file names a newer version does it download `triokey.exe` from the same release, check its [minisign](https://jedisct1.github.io/minisign/) signature against the public key built into the running exe, and put the file in place for the next start. Each of the two files takes two or three HTTPS requests, because GitHub redirects the download to its release-asset server. The requests carry a fixed `User-Agent: TrioKey-updater` and nothing else: no typed text, no machine or user identifier, not even the running version. GitHub sees your IP address, as any site you download from does. The time of the last check is kept in `config.txt`; the admin-mode copy keeps it in memory only, so it may check once more after each start, then once a day.
- On first run, if a folder `%APPDATA%\evj` exists (the project's working name before release), copies `config.txt`, `force_vi.txt` and `force_en.txt` from it. A new user has no such folder.

What it never does:

- Write keys or typed text to disk. There is no typing log. The timing figure shown in Settings is kept in memory only.
- Connect to anything but GitHub (`github.com` and the host its release downloads redirect to), or for anything but that update check. With `Tự động cập nhật` off, TrioKey uses no network at all unless you press `Kiểm tra bản mới`, which checks once and, in a normal copy, also installs what it finds. A build without an update key has both controls greyed out and never connects. There is no telemetry or crash reporting. Its direct third-party dependencies are the `vi`, `windows-sys` and `minisign-verify` crates, plus `embed-manifest` at build time only (it generates the application manifest, which the build script writes into the exe with the icon; none of its code runs).
- Register a text service, add a keyboard layout or change the input-language list.

Things to know:

- **Passwords.** Classic Win32 password fields (Edit/RichEdit controls with `ES_PASSWORD`) are meant to be skipped; this is not yet verified by hand. Password fields in browsers and Electron apps cannot be detected. There, a fragment of a password that is plain letters in normal case and comes before the first digit or `_` can be changed in two ways: it happens to be Telex for a Vietnamese syllable (the `mas` in `mas!x9` becomes `má`), or it contains a doubled tone key (`s f r x j`) and is not in the English word list, in which case it loses one letter (`Tess!` → `Tes!`, `passwd!` → `paswd!`). So switch TrioKey off with the hotkey while typing a password in such a field; the tray icon shows the state.
- **Admin mode** is off by default. It has not been run on a real machine yet; what follows is the design. Turning it on shows one UAC prompt. The elevated helper copies the exe to `%ProgramFiles%\TrioKey` (writable only by administrators, so another program cannot swap the exe to gain admin rights) and registers a logon task that runs it with highest privileges. The elevated copy reads only data files with fixed names and starts no other program: its config-folder button shows the path instead of opening Explorer. It rewrites `config.txt` only when that file already exists and neither it nor its folder is a link, and it creates nothing under `%APPDATA%`. Turn admin mode on from a folder that other programs cannot write to: the UAC prompt elevates whatever `triokey.exe` is in that folder at that moment.
- **Updates** are installed only with a valid signature from the project's update key, for the file name `triokey.exe`, and for a version newer than the running one; anything else is refused, including an older release offered again. The signature is the whole trust. The key is held by this project's release workflow, which signs and publishes a version tag pushed on `main` with no manual approval: whoever obtains the key, or can push to this repository (only its owner), can ship an update to every installed copy. Someone who can only replace the release files cannot. The exe has no Authenticode signature, so SmartScreen treats a release like any other unsigned download. In admin mode the download, the check and the swap all happen inside `%ProgramFiles%\TrioKey`, and only after you click. A TrioKey started with "Run as administrator" from any other folder writes nothing there: it only reports that a new version exists. Not run on a real machine yet; design notes in [docs/decisions/0007-auto-update.md](docs/decisions/0007-auto-update.md).
- **Keys injected by other software** (remote desktop, key remappers) as ordinary key presses are treated as real keys; text injected as Unicode characters is left alone. Not tested with specific tools. That is also why two input methods must not run together. After 0.1.0 (in builds from `main`, not in a release yet, not run on a real machine): keys that Windows marks as injected by a lower-integrity process pass through untouched, so a program without admin rights cannot make an elevated TrioKey type. While TrioKey runs elevated (admin mode), keys sent by such tools from a normal process, for example AutoHotkey or an on-screen keyboard, therefore get no Vietnamese.

## How it works

Two crates:

- **`triokey-engine`**: pure Rust with no operating-system calls. It keeps the raw keys of the word being typed and, after every key, recomputes from scratch what those keys should look like, compares that with what is on screen and returns one edit: delete N characters, insert a string. Besides a few flags (code, raw, previous-word context) it keeps no composition state built up key by key, so it cannot drift from its own model of the screen. It does not read the screen back, so the real text can still differ in fields that insert suggestions. Telex composition and tone placement come from the [`vi`](https://github.com/ZeroX-DG/vi-rs) crate; the Vietnamese syllable check, the English word list, the code rules and the per-word decision are TrioKey's. It has its own test suite.
- **`triokey`**: the Windows tray app. The hook callback asks the engine and, when the text must change, swallows the key and sends Backspaces plus the new text with `SendInput` from inside the callback, so no later key can slip in between. Keys that change nothing pass through untouched. TrioKey's own injected keys are marked and ignored. The hook callback's p99 time is shown at the bottom of the settings window; no figure is published yet.

IME state is read from the focused window's default IME window (`ImmGetDefaultIMEWnd` + `WM_IME_CONTROL`): the open status for Japanese, and the open status plus conversion mode for Chinese and Korean. A separate thread keeps the value fresh (on focus changes; after keys that type nothing, such as F-keys and the Kanji/IME keys, after shortcuts and after clicks; and on a 100 ms tick), so the key path only reads a cached value. A toggle made with a bare modifier such as Shift is seen on the next tick. The first key after a possible toggle may wait about 30 ms for a fresh read.

Design notes: [docs/decisions/](docs/decisions/).

## Roadmap

Not in a fixed order:

- Run the hands-on test list ([docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md)) on the current build and record the results: Japanese, tray, settings, start-with-Windows, admin mode, the updater.
- Fields that insert suggestions (browser address bar, Excel cells).
- Hand-test Chinese and Korean IMEs.
- Better decisions for words that are valid in both languages.
- VNI.
- macOS and Linux front-ends on the same engine.

Not planned: macros, per-app memory, legacy code tables, cloud features.

## Acknowledgements

- [`vi`](https://github.com/ZeroX-DG/vi-rs) (MIT): the Telex transform.
- [SCOWL](http://wordlist.aspell.net): the English word list.
- [VieLotus-IM](https://github.com/stevedat/VieLotus-IM) (MIT): its public regression suite is what TrioKey measures itself against, and its word lists feed TrioKey's lexicon.
- [VKey](https://github.com/phatMT97/VKey): stepping aside on a CJK layout, which TrioKey still falls back to. Behaviour only.
- [Vikey](https://github.com/tonyminibot/Vikey): its published decisions on a doubled tone key and on giving the typed keys back. Behaviour only.
- [UniKey](https://www.unikey.org), EVKey and [OpenKey](https://github.com/tuyenvm/OpenKey): the long-standing input methods most Vietnamese typists already know; TrioKey follows the Telex keys people know from them.

No source code, data or tests are taken from the GPL projects.

Copyright lines, licence texts and details are in [NOTICE.md](NOTICE.md).

## Licence

[MIT](LICENSE).
