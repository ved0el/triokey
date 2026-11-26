<p align="center"><img src="https://raw.githubusercontent.com/ved0el/triokey/main/assets/brand/social-preview.png" width="640" alt="TrioKey: Vietnamese typing for Windows"></p>

<p align="center">
  <a href="https://github.com/ved0el/triokey/actions/workflows/ci.yml"><img src="https://github.com/ved0el/triokey/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/ved0el/triokey/releases/latest"><img src="https://img.shields.io/github/v/release/ved0el/triokey" alt="Latest release"></a>
  <a href="https://github.com/ved0el/triokey/releases"><img src="https://img.shields.io/github/downloads/ved0el/triokey/total" alt="Downloads"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/ved0el/triokey" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-3565B8" alt="Platform: Windows 10 and 11">
</p>

# TrioKey

**English** · [Tiếng Việt](README.vi.md)

A Vietnamese input method for Windows, made for people who type English, Vietnamese and a CJK language (Chinese, Japanese or Korean) every day. In everyday mixed typing you do not toggle Vietnamese on or off: the only switch you touch is your CJK IME's own on/off.

```text
keys typed          what appears
tieesng vieejt␣     tiếng việt
windows␣            windows
songs␣              sóng         (on a fresh line, the commoner reading wins)
the songs␣          the songs    (after an English word)
this␣⌫              thí          (one Backspace swaps a word the engine was unsure of)
```

Early version: 0.2.1 is released and signed, and the examples on this page describe its engine. Much of the Windows app, including the updater that is on by default, has not been checked on a real machine. Read [Status](#status) first.

## Why

**1. Two toggles for one intention.**
With other input methods, typing a Japanese word in the middle of Vietnamese means turning hiragana on and also turning Vietnamese off, because Telex would mangle the romaji; then both again on the way back.
With TrioKey you turn hiragana on, type, and turn it off. TrioKey reads whether the CJK IME is actually open in the window you are typing in and steps aside by itself. When the IME is closed (direct input), it types Vietnamese, even under the Japanese keyboard layout.

**2. No English/Vietnamese mode.**
Each word is decided when you finish it. TrioKey compares the keys as typed with their Telex reading, using how often each is used as a word in English and in Vietnamese text, and how Vietnamese the current line has been so far. When it is unsure, one Backspace right after the word swaps to the other reading, and a swap you keep is remembered.

**3. Nothing changed in your Windows keyboard setup.**
TrioKey adds no keyboard layout, registers no text service (TSF) and leaves your input-language list alone. It is a small tray app: a keyboard hook (plus a mouse hook that only notes clicks) and injected text.

**4. Admin windows.**
A normal-privilege input method cannot type into elevated windows. TrioKey has an optional admin mode: one UAC prompt, a logon task, and the exe copied to a folder only administrators can write. (Not yet run on a real machine.)

**5. Small on purpose.**
No macros, no per-app memory, no cloud, no telemetry, no typing log. The only network use is a check for signed updates on GitHub, and you can switch it off. One small settings window and plain-text files.

## Status

- **Released:** [0.2.1](https://github.com/ved0el/triokey/releases), built and signed by CI ([CHANGELOG.md](CHANGELOG.md)). 0.2.0 brought the engine described on this page (word frequencies and the line's language), remembering kept swaps, and the red/blue tray icons and new app icon; 0.2.1 reworked the settings window. 0.1.0 and 0.1.1 had an earlier engine, which decided words from a word list and the word before.
- **Typed with by the author:** the 0.2.0 build was used for ordinary mixed English and Vietnamese typing on one Windows 11 machine and found good enough to release; a few words still came out wrong. No swap was remembered in that session, and no row of the checklist was run except one timing reading (AT7).
- **Hand-tested:** an earlier build with Japanese (Microsoft IME, in Notepad and Brave) passed AT1 and AT2 of [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md). Neither has been repeated with 0.2.0.
- **Not checked on a real machine** (no checklist row has a recorded result), only compiled and unit-tested where possible: the tray icon and menu, the app icon, the hotkeys, the settings window (its light and dark looks have been seen only as images rendered by a test at 96 DPI, never on a desktop; the high-contrast look has never been seen with a contrast theme on), start-with-Windows, admin mode, the updater (AT10 in [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) is still open), saving of remembered words, the uninstall steps and the skipping of Win32 password fields. The sections below describe them as written in the code. If you would rather not have an untested updater replace the exe, untick `Tự động cập nhật` in Settings.
- **Chinese and Korean:** only the decision rule (IME open and in native mode) is unit-tested, with made-up values. Whether those IMEs report their state correctly has not been tried.
- Windows 11 only so far; Windows 10 is untried. Telex only, no VNI. The interface (tray menu, tooltip, settings, messages) is in Vietnamese only.
- **Measured** by typing text through the engine in a test (not on a real machine): on 10,274 words of mixed English/Vietnamese text written by people who never saw the engine, wrong words went from 0.54% with the engine of 0.1.1 to 0.16% with the current one, and keystrokes spent fixing words from 2.74 to 0.59 per 100 words. With remembering on, wrong words went from 16 to 12 and no word had to be fixed twice. The VieLotus-IM regression suite, a guardrail of hard single words typed without context, went from 8,361 to 8,524 of 9,084. How both are run: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), section 11.
- Terminals get no special handling. Fields that insert suggestions after the cursor (browser address bar, Excel cells) get none either.
- Password fields in browsers and Electron apps cannot be detected; see [Privacy and security](#privacy-and-security).
- Some apps hide the IME state. When it cannot be read, TrioKey keeps the window's last known state, and on a CJK layout steps aside when there is none.
- macOS and Linux are not started.

## How it behaves

| Keyboard layout of the window | CJK IME state | What your keys do | Tray |
| --- | --- | --- | --- |
| English, or any non-CJK layout | (not relevant) | Telex Vietnamese, English words left as typed | `V` |
| Japanese | closed (direct input) | Telex Vietnamese, English words left as typed | `V` |
| Japanese | open (hiragana, katakana…) | every key goes straight to the IME, untouched | `E` |
| Chinese or Korean † | closed, or open in alphanumeric mode | Telex Vietnamese, English words left as typed | `V` |
| Chinese or Korean † | open in native mode | every key goes straight to the IME, untouched | `E` |
| CJK, in an app that hides the IME state | unknown | TrioKey steps aside | `E` |
| any, after you switch TrioKey off | (not relevant) | every key passes through | `E` |

† Untested with a real IME: this is the coded rule, checked only by unit tests. The Japanese rows were hand-tested on an earlier build (see [Status](#status)).

TrioKey does not change the keyboard layout for you. "One switch" holds when you stay on the CJK layout and use the IME's own on/off; if you sit on the English layout, you still switch layouts to reach Japanese, you just no longer toggle Vietnamese.

### English or Vietnamese

When a word ends (space, Enter, punctuation), TrioKey has two readings: the keys as typed, and their Telex result. It rates each by how often it is used as a word, from frequency tables built from web text in each language, plus an English dictionary list and a list of technical terms. The line keeps a running lean: each finished word moves it toward Vietnamese or English, and Enter or moving the cursor starts a new line, which leans slightly Vietnamese. The reading that is likelier on this line wins. Words with digits, `_` or mixed case, paths, flags and text right after `.` `:` `\` are code and stay as typed.

One Backspace right after a word swaps it to the other reading and deletes the space in one go, but only when the engine was unsure: the other reading was not far less likely, or it is a common English word that lost to Vietnamese. Words in the force files, remembered words and ALL CAPS words left as typed always offer the swap when the word has a second reading. After a word it is sure of, Backspace only deletes the space. A second Backspace always deletes as usual.

### Typed examples

Every line below is checked by the test `readme_examples` in `crates/triokey-engine/tests/engine.rs`. Each line is typed on a fresh line. `␣` is the space bar, `⌫` is Backspace; the space a `␣` leaves at the end is not shown.

```text
keys typed                     what appears
tieesng vieejt␣                tiếng việt
windows terrible␣              windows terrible
DDuowngf␣                      Đường
baajt/taawst␣                  bật/tắt
my_var obj.dir src/main␣       my_var obj.dir src/main       (code is left alone)
server cos␣                    server có          (có is far commoner, even after English)

songs␣                         sóng               (valid both ways: a fresh line leans Vietnamese)
the songs␣                     the songs          (after an English word)
tooi thaasy songs␣             tôi thấy sóng      (after Vietnamese words)

this␣                          this
this␣⌫                         thí                (unsure: one Backspace swaps)
this␣⌫⌫                        th                 (a second Backspace deletes)
off␣⌫                          of
tooi␣⌫                         tôi                (sure: Backspace only deletes the space)
tooi thaasy sets␣              tôi thấy sét       (a common English word after Vietnamese ones...)
tooi thaasy sets␣⌫             tôi thấy sets      (...is always one Backspace away)
There␣                         Thể                (the first word of a line can go wrong...)
There␣⌫                        There              (...one Backspace gives it back)

mixx␣                          mix                (a doubled tone key cancels the mark)
offline passwd Terraform␣      offline passwd Terraform
ko ddc ddaau␣                  ko đc đâu          (common abbreviations with đ compose)
dd/mm/yyyy aa aw␣              dd/mm/yyyy aa aw   (lone dd, aa, aw stay as typed)
toansz␣                        toan               (z removes the tone)
OS naof cungx dduwowcj␣        OS nào cũng được   (ALL CAPS stays unless common Vietnamese)
VIEEJT␣                        VIỆT
chaau As␣                      châu Á             (a one-letter word: Shift on the letter only)
Mac OS␣                        Mac OS             (two capitals are an acronym anywhere)
```

## How it differs

As of November 2026. VKey, Vikey and VieLotus-IM were described from reading their repositories and documentation; none was installed or run. UniKey, EVKey and OpenKey were not read; they are named only for their well-known Vietnamese/English mode with a toggle.

| | TrioKey | Common approaches elsewhere |
| --- | --- | --- |
| Stepping aside for a CJK IME | reads the IME's open/closed state in the focused window | step aside whenever a CJK layout is active (VKey, Vikey), or be a separate entry in the Windows input list (VieLotus-IM) |
| English words in Vietnamese text | no mode: decided per word from word frequencies and the line | a Vietnamese/English mode with a toggle (UniKey, EVKey, OpenKey, VKey), or automatic detection (Vikey, VieLotus-IM) |
| Windows integration | keyboard hook and injected text, nothing registered | the same for hook-based tools; a TSF text service for VieLotus-IM and for VKey's optional component |
| Input styles, macros | Telex only, no macros | VNI and other styles are common; VKey and VieLotus-IM have macros |

The others are further along: they are packaged, already distributed, and cover more input styles and, for some, more platforms. TrioKey learned from several of them; see [Acknowledgements](#acknowledgements).

## Install

There is no installer. Releases are built and signed by CI from a version tag ([docs/RELEASING.md](docs/RELEASING.md)).

1. **Quit every other Vietnamese input method first** (UniKey, EVKey, OpenKey, VKey, Vikey…). Two of them running together will rewrite each other's text.
2. Download `triokey-X.Y.Z-windows-x64.zip` from the [latest release](https://github.com/ved0el/triokey/releases/latest) and unpack it into a folder of its own, or build from source (below).
3. Run `triokey.exe`. No window opens; a red `V` appears in the system tray (possibly behind the `^` overflow arrow). 0.1.0 and 0.1.1 drew it in one colour.

### Build from source

Only the GNU toolchain is supported: the build script writes the exe's resources (icon and manifest) for `x86_64-pc-windows-gnu` only, so the default MSVC toolchain does not build TrioKey.

1. Install Rust. Either:
   - [mise](https://mise.jdx.dev): run `mise install` in the repository (it reads `mise.toml`, which selects the stable GNU toolchain; Visual Studio Build Tools are not needed), or
   - [rustup](https://rustup.rs) with the `stable-x86_64-pc-windows-gnu` toolchain.
2. Build:

   ```sh
   git clone https://github.com/ved0el/triokey
   cd triokey
   cargo build --release -p triokey        # with mise: mise x -- cargo build --release -p triokey
   ```

3. The exe is `target\release\triokey.exe`. It carries the same update key as a release, so it checks for updates too.

Tests and the other checks CI runs are in [CONTRIBUTING.md](CONTRIBUTING.md).

The exe is not code-signed. SmartScreen or antivirus software may warn about it, as they do for many keyboard-hook programs. The `triokey.exe.minisig` published next to each released exe can be checked by hand with [minisign](https://jedisct1.github.io/minisign/) and the public key in `crates/triokey/update-pubkey.txt`; that is the same check the built-in updater makes.

## Usage

The interface is in Vietnamese only, so the Vietnamese labels are given next to the English names. This section describes 0.2.1 (the settings window looked different in 0.2.0); the red/blue icon, the current Backspace rules and remembered words are not in 0.1.0 and 0.1.1. Most of it has not been checked on a real machine (see [Status](#status)).

- **Tray icon.** A red `V` means TrioKey is typing Vietnamese. A blue `E` means keys pass through, either because you switched it off or because it is stepping aside for a CJK IME; the tooltip says which (`Tiếng Anh — đã tắt` = switched off, `Tiếng Anh — đang nhường IME Nhật` = stepping aside for the Japanese IME; `IME Trung` and `IME Hàn` are Chinese and Korean). The letters sit on a transparent background, in deep shades on a light taskbar and light shades on a dark one, and are redrawn when the Windows theme changes.
- **Left click** the icon to switch TrioKey on or off. **Right click** for the menu: `Tiếng Việt` (on/off), `Cài đặt…` (Settings), `Thoát` (Quit).
- **Hotkey** for on/off: Ctrl+Shift (default; press and release with no other key), Alt+Z, Ctrl+Space, or none (`Không dùng`). Switching with the hotkey or the tray is not saved; the on/off box in Settings is the one that is saved.
- **Settings** (`Cài đặt…`): on/off (`Bật gõ tiếng Việt`), hotkey (`Phím tắt`), tone placement (`Kiểu bỏ dấu`: `hòa, thúy` or `hoà, thuý`), deciding English words automatically (`Tự giữ nguyên từ tiếng Anh`; off means plain Telex for every word), step aside for CJK IMEs (`Nhường khi IME Nhật/Trung/Hàn đang bật`), start with Windows (`Khởi động cùng Windows`), admin mode (`Chạy quyền admin (gõ được trong cửa sổ admin)`), automatic updates (`Tự động cập nhật`) with a check-now button (`Kiểm tra bản mới`), and developer mode (`Chế độ phát triển`, off by default), which adds a button that opens the config folder (`Mở thư mục cấu hình…`) and the hook timing line under it. While a newer version waits, the line under the update box reads `Có bản mới vX.Y.Z.` with a link `Cập nhật tại đây` that installs it or opens the download page. The footer credits the author (`TrioKey vX.Y.Z · © 2026 ved0el`) and links to this repository (`GitHub`); the admin-mode copy shows that address in a message box instead of opening a browser. Changes apply at once. The window follows the Windows app mode (light or dark; on Windows 11 its title bar is the same colour as the rest) and keeps the plain system look when high contrast is on; switching either while it is open recolours it.
- **Telex**: inside a word, `aa aw ee oo ow uw dd` → `â ă ê ô ơ ư đ` (`caan` → `cân`, `ddi` → `đi`); `s f r x j` → tone marks; `z` removes the tone. The tone key may come right after the vowel or at the end of the word (`tieesng`, `tieengs` → `tiếng`). A lone `w` shows `ư` while you type it but becomes `w` when the word ends (`w␣` → `w`, then `⌫` → `ư`). Pressing a tone key twice cancels the mark (`mixx␣` → `mix`); English words with doubled letters stay as typed.
- **Remembering a swap.** If you swap a word with one Backspace and then type a space, Tab or Enter, TrioKey remembers the swap at once and uses it from then on. A letter or punctuation right after the swap stores nothing. Swapping a remembered word back with one Backspace changes or forgets the entry. One exception protects Vietnamese words: in the middle of a line that has already turned Vietnamese, a remembered English reading never replaces the Vietnamese word; it stays one Backspace away, and keeping that swap leaves the entry as it is. Each step below starts a new line, in this order:

  ```text
  keys typed               what appears
  this␣⌫␣                  thí               (swapped and kept: remembered)
  this␣                    thí
  this␣⌫␣                  this              (swapped back and kept: forgotten)
  this␣                    this
  songs␣⌫␣                 songs             (remembered as English)
  songs␣                   songs
  tooi thaasy songs␣       tôi thấy sóng     (a Vietnamese line keeps the Vietnamese word...)
  tooi thaasy songs␣⌫      tôi thấy songs    (...one Backspace away)
  ```

  Remembered words are kept in `%APPDATA%\triokey\learned.txt`, one `keys<TAB>text` line each. Delete a line to forget that word; edit the file only after quitting TrioKey (`Thoát`), or it is overwritten. At most 5,000 entries are kept, and they never expire. The file is saved about 2 seconds after the last change, on quit and at logoff.
- **Force a word.** For a permanent choice, create `force_en.txt` (always English) or `force_vi.txt` (always Vietnamese) in the same folder. One word per line, written as the keys you type (`this`, not `thí`); `#` starts a comment. They beat remembered words, swaps of forced words are not remembered, and one Backspace still offers the other reading when there is one. They are read at startup, so restart TrioKey after editing.
- **Config** lives in `%APPDATA%\triokey\config.txt` (`key = value` lines). `dev_mode = true` is the developer mode box (`Chế độ phát triển`).
- **Updates.** With `Tự động cập nhật` on (the default), TrioKey asks GitHub once a day whether a newer signed version exists (the first try comes about a minute after it starts; a check that gets no answer, for example offline, is tried again an hour later). A normal copy downloads it, verifies the signature and replaces its own exe; a notification says `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau` (updated, active from the next start), and the previous exe stays next to it as `triokey.old.exe` until that next start. The admin-mode copy never installs by itself: it downloads the new exe into memory, verifies the signature, shows `Có bản X.Y.Z — bấm để cập nhật` as a notification and as a tray menu item, and updates when you click one of them. If TrioKey cannot write to its own folder, or its exe has been renamed to something other than `triokey.exe`, it only says that a new version exists; clicking the notice opens the download page. A version is announced only after its signature has been verified.

## Uninstall

These steps follow the code; they have not been run on a real machine yet. If one fails, the manual commands (`schtasks /Delete`, `reg delete`) are in the last section of [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) (in Vietnamese).

1. If you turned on **admin mode** or **start with Windows**, undo them in Settings, in this order:
   1. Untick admin mode (`Chạy quyền admin (gõ được trong cửa sổ admin)`). This removes the `TrioKey` logon task and the copy in `%ProgramFiles%\TrioKey`. If you do it from the admin copy itself, Windows removes that folder at the next restart; quit that copy and start the normal one before the next step.
   2. Then untick start with Windows (`Khởi động cùng Windows`); the box is greyed out while admin mode is on. Turning admin mode off from the normal copy puts the Run value back if it was set before, so look at this box even if you did not tick it lately. Unticking it removes the `TrioKey` value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
2. Quit from the tray menu (`Thoát`).
3. Delete the folder you unpacked, built or copied TrioKey into, and `%APPDATA%\triokey` if you also want the config, force files and remembered words gone.

If you never turned on either option, steps 2 and 3 are the whole uninstall. TrioKey registers nothing else: no keyboard layout, no language-list entry, nothing under `HKCU\Keyboard Layout`.

## Privacy and security

What it does:

- Installs a low-level keyboard hook (`WH_KEYBOARD_LL`). Like every hook-based input method, it therefore sees every key you press while it runs.
- Installs a low-level mouse hook (`WH_MOUSE_LL`). It uses button presses only, to know that the caret may have moved and that the IME may have been toggled.
- Listens for foreground and focus changes (two WinEvent hooks) and reads the time of the last input.
- Sends Backspaces and Unicode text to the focused window with `SendInput`.
- Reads the focused window's keyboard layout and IME open/closed state.
- Stores, in `%APPDATA%\triokey`: `config.txt` (created on first run, rewritten when you change a setting), the two force files you create yourself, and `learned.txt` (created on first run, from 0.2.0 on). `learned.txt` holds only the words you swapped with Backspace and kept: their keys and the text shown, nothing else, no times, no order of typing.
- Reads two values under `HKCU`: the Windows light/dark setting (for the icon and settings window colours) and its own Run value.
- If `Tự động cập nhật` is on: once a day (first try about a minute after start; a check that gets no answer is tried again an hour later), fetches the signature file of the latest release (a few hundred bytes) from `github.com`. Only when that file names a newer version does it download `triokey.exe` from the same release, check its [minisign](https://jedisct1.github.io/minisign/) signature against the public key built into the running exe, and put the file in place for the next start. Each of the two files takes two or three HTTPS requests, because GitHub redirects the download to its release-asset server. The requests carry a fixed `User-Agent: TrioKey-updater` and nothing else: no typed text, no remembered words, no machine or user identifier, not even the running version. GitHub sees your IP address, as any site you download from does. The time of the last check is kept in `config.txt`; the admin-mode copy keeps it in memory only, so it may check once more after each start, then once a day.
- On first run, if a folder `%APPDATA%\evj` exists (the project's working name before release), copies `config.txt`, `force_vi.txt` and `force_en.txt` from it. A new user has no such folder.

What it never does:

- Keep a typing log. Apart from the swapped words in `learned.txt`, no keys or typed text are written to disk. The timing figure shown in Settings (developer mode) is kept in memory only.
- Connect to anything but GitHub (`github.com` and the host its release downloads redirect to), or for anything but that update check. With `Tự động cập nhật` off, TrioKey uses no network at all unless you press `Kiểm tra bản mới`, which checks once and, in a normal copy, also installs what it finds. A build without an update key has both controls greyed out and never connects. There is no telemetry or crash reporting. Its direct third-party dependencies are the `vi`, `windows-sys` and `minisign-verify` crates, plus `embed-manifest` at build time only (it generates the application manifest, which the build script writes into the exe with the icon; none of its code runs).
- Register a text service, add a keyboard layout or change the input-language list.

Things to know:

- **Passwords.** Classic Win32 password fields (Edit/RichEdit controls with `ES_PASSWORD`) are meant to be skipped; this is not yet verified by hand. Password fields in browsers and Electron apps cannot be detected. There, a part of a password made of plain letters can happen to be Telex for a Vietnamese syllable and be changed (`mas!x9` → `má!x9`). So switch TrioKey off with the hotkey while typing a password in such a field; the tray icon shows the state.
- **Admin mode** is off by default and has not been run on a real machine yet. Turning it on shows one UAC prompt. The elevated helper copies the exe to `%ProgramFiles%\TrioKey` (writable only by administrators, so another program cannot swap the exe to gain admin rights) and registers a logon task that runs it with highest privileges. The elevated copy starts no other program: its config-folder button (developer mode) shows the path instead of opening Explorer, and the settings links show the address instead of opening a browser. It creates nothing under `%APPDATA%`, and it reads `learned.txt` and rewrites `learned.txt` or `config.txt` only when the file already exists as a plain file in the real `%APPDATA%\triokey`, with no link anywhere on the path; otherwise settings and remembered words stay in memory. Turn admin mode on from a folder that other programs cannot write to: the UAC prompt elevates whatever `triokey.exe` is in that folder at that moment. Design details: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), section 8.
- **Updates** are installed only with a valid signature from the project's update key, for the file name `triokey.exe`, and for a version newer than the running one; anything else is refused, including an older release offered again. The signature is the whole trust. The key is held by this project's release workflow, which signs and publishes a version tag pushed on `main` with no manual approval: whoever obtains the key, or can push to this repository (only its owner), can ship an update to every installed copy. Someone who can only replace the release files cannot. The exe has no Authenticode signature, so SmartScreen treats a release like any other unsigned download. In admin mode the download, the check and the swap all happen inside `%ProgramFiles%\TrioKey`, and only after you click. A TrioKey started with "Run as administrator" from any other folder writes nothing there: it only reports that a new version exists. Details: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), section 9, and [docs/RELEASING.md](docs/RELEASING.md).
- **Keys injected by other software** (remote desktop, key remappers) as ordinary key presses are treated as real keys; text injected as Unicode characters is left alone. Not tested with specific tools. That is also why two input methods must not run together. Since 0.1.1, keys that Windows marks as injected by a lower-integrity process pass through untouched, so a program without admin rights cannot make an elevated TrioKey type; while TrioKey runs elevated, keys sent by such tools from a normal process (AutoHotkey, an on-screen keyboard) therefore get no Vietnamese.

## How it works

Two crates:

- **`triokey-engine`**: pure Rust with no operating-system calls. It keeps the raw keys of the word being typed and, after every key, recomputes from scratch what those keys should look like, compares that with what is on screen and returns one edit: delete N characters, insert a string. Between words it keeps little: the line's lean, a few flags about the current run of text (code or not), the Backspace swap on offer and the remembered words. It does not read the screen back, so the real text can still differ in fields that insert suggestions. Telex composition and tone placement come from the [`vi`](https://github.com/ZeroX-DG/vi-rs) crate; the syllable check, the frequency tables, the code rules and the per-word decision are TrioKey's. It has its own test suite and a typing benchmark.
- **`triokey`**: the Windows tray app. The hook callback asks the engine and, when the text must change, swallows the key and sends Backspaces plus the new text with `SendInput` from inside the callback, so no later key can slip in between. Keys that change nothing pass through untouched. TrioKey's own injected keys are marked and ignored. The hook callback's p99 time is shown in the settings window, under the developer mode box, while developer mode (`Chế độ phát triển`) is on. The only reading so far, on the author's machine with 0.2.0: 314 µs for keys passed through and 5,600 µs for keys that replace text, over 502 keys.

IME state is read from the focused window's default IME window (`ImmGetDefaultIMEWnd` + `WM_IME_CONTROL`): the open status for Japanese, and the open status plus conversion mode for Chinese and Korean. A separate thread keeps the value fresh, so the key path only reads a cached value.

The full design, threads, data files, the elevated copy, updates, release flow and tests: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Roadmap

Not in a fixed order:

- Run the hands-on test list ([docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md)) on the current build and record the results: Japanese, tray, settings, start-with-Windows, admin mode, the updater, remembered words.
- Fields that insert suggestions (browser address bar, Excel cells).
- Hand-test Chinese and Korean IMEs.
- VNI.
- macOS and Linux front-ends on the same engine.

Not planned: macros, per-app memory, legacy code tables, cloud features.

## Acknowledgements

- [`vi`](https://github.com/ZeroX-DG/vi-rs) (MIT): the Telex transform.
- [FineWeb-2](https://huggingface.co/datasets/HuggingFaceFW/fineweb-2) and [FineWeb](https://huggingface.co/datasets/HuggingFaceFW/fineweb) (ODC-By 1.0): the word counts behind the Vietnamese and English frequency tables.
- [SCOWL](http://wordlist.aspell.net): the English dictionary list.
- [VieLotus-IM](https://github.com/stevedat/VieLotus-IM) (MIT): its public regression suite, which TrioKey runs as a guardrail, and the SCOWL-based word lists that feed TrioKey's English list.
- [VKey](https://github.com/phatMT97/VKey): stepping aside on a CJK layout, which TrioKey still falls back to. Behaviour only.
- [Vikey](https://github.com/tonyminibot/Vikey): its published decisions on a doubled tone key and on giving the typed keys back. Behaviour only.
- [UniKey](https://www.unikey.org), EVKey and [OpenKey](https://github.com/tuyenvm/OpenKey): the long-standing input methods most Vietnamese typists already know; TrioKey follows the Telex keys people know from them.

No source code, data or tests are taken from the GPL projects.

Copyright lines, licence texts and details are in [NOTICE.md](NOTICE.md). Contributions: [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

[MIT](LICENSE).
