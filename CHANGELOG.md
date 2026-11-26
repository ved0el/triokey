# Changelog

## 0.3.0 (2026-11-26)

First release for Windows and macOS together. The Windows changes below were seen only as images rendered by a test; the macOS app is experimental (see its section).

### Windows app (`triokey`)

- **Developer mode shows two buttons** side by side under `Chế độ phát triển`: `Mở thư mục cấu hình…` as before, and `Chép thông tin debug`, which copies a short English report to the clipboard (version, Windows version, normal or admin copy, paths with your user folder shown as `~` and any path outside it replaced by `outside the profile folder`, every setting, on/off and IME state, hook timings, update status, how many remembered words and force-file entries there are, window look and DPI) and says so in a message box. It never contains typed text, remembered words or your user name. The `Hook p99: …` line and its one-second timer are gone from the window; its numbers are in the report.
- **The update line always keeps room for one line**, so pressing `Kiểm tra bản mới` no longer changes the window size for the usual answers. The no-network answer is now `Chưa kiểm tra được: không tải được từ GitHub.`

### macOS app (`triokey-mac`, experimental)

Written and type-checked on Windows. The owner installed the first macOS build on a Mac, granted the permission and typed with it; everything added since (settings window, files, hotkey, start at login, version check, debug information) has not run on a real Mac yet. The engine is unchanged; the Windows app changes are listed above.

- **Same engine, macOS front-end.** A session event tap reads key-downs, modifier changes (for the hotkey only) and mouse-downs; for an edit the key is swallowed and TrioKey posts Deletes, the text as Unicode key events (at most 20 UTF-16 units each) and, after a word ended with Return or Tab, the real key with the original Shift and Option. No input source or keyboard layout is registered. A panic or an internal error in the tap turns the tap off for the rest of the run and lets the key through.
- **Menu bar item** `V`/`E` with a menu: `Tiếng Việt` (on/off), `Cài đặt…`, `Hướng dẫn cấp quyền…`, `Có bản … — mở trang tải về` while a newer version waits, `TrioKey <version>`, `Thoát`. No Dock icon.
- **Settings window** with the structure and texts of the Windows one: on/off and the hotkey (`Control+Shift`, `Option+Z` or none; no Control+Space, which is macOS's input-source switch), tone style, keeping English words, stepping aside for CJK input methods (off: Vietnamese under any input source), `Khởi động cùng máy`, developer mode with `Mở thư mục cấu hình…` and `Chép thông tin debug`, `Tự động kiểm tra bản mới` and `Kiểm tra bản mới` with one reserved line for the answer, and the credit with a GitHub link. Standard controls only, so it follows light and dark. Settings apply at once.
- **Files** in `~/Library/Application Support/triokey`, in the Windows formats: `config.txt`, `learned.txt` (remembered swaps now survive a restart), `force_vi.txt` and `force_en.txt`. Writes go through a temporary file renamed over the old one; a `config.txt` that is not UTF-8 is never overwritten.
- **Start at login** through a LaunchAgent file without KeepAlive, available only when TrioKey runs from Applications.
- **Version check, never an installation:** daily (if on) and on request, `/usr/bin/curl` asks only where the releases page redirects, with a fixed user agent and nothing about you, your Mac or the running version; a newer version is shown with a link to the releases page.
- **Debug information** (`Chép thông tin debug`, replaces the menu item `Chẩn đoán`): macOS version, bundle path, tap and permission state, input sources seen, counters, last error, every setting, config folder, counts of remembered words and force-file entries, start-at-login and update state; your home folder is shown as `~`, never typed text.
- **Accessibility permission** with a guide window that opens the Accessibility page of System Settings, is written to notice the grant within about 2 seconds (unverified), and warns never to remove the permission while TrioKey runs.
- **Steps aside** whenever the input source is anything but a plain ASCII-capable keyboard layout (unless turned off in the settings), and while Secure Input is on.
- **Packaging:** one universal app (Apple silicon and Intel, macOS 11 or later) in a dmg, signed ad hoc, not notarized: the first open needs Open Anyway, and each new build needs the Accessibility entry removed and added again (quit TrioKey first).
- **Part of the release:** the tag's release workflow now also runs the macOS workflow and publishes `TrioKey-X.Y.Z-macos.dmg` with the Windows files; `SHA256SUMS.txt` lists it too, and the tag must match the version of both apps. The macOS workflow also runs on `main` and on pull requests, and self-tests the Intel half under Rosetta when the runner has it.

## 0.2.1 (2026-11-26)

The owner reviewed images of the window rendered by a test at 96 DPI (hidden or cloaked, never on screen) and asked for the footer and the notice as they are now. The window itself has not been checked on a desktop (rows 2n to 2x in `docs/HANDS_ON_TEST.md`).

### Windows app (`triokey`)

- **Settings window look.** Light and dark colours that follow the Windows app mode (Settings > Personalization > Colors), switched while the window is open. On Windows 11 the title bar has the same colour as the rest of the window, and the window border is a thin line that stays visible on a desktop of the same colour. The options are grouped under the headings `Gõ`, `Hệ thống` and `Cập nhật`, without frames. Below a thin line, the footer is one line of credit, `TrioKey vX.Y.Z · © 2026 ved0el`, and a `GitHub` link to the repository page. With high contrast on, the window keeps the plain system look and colours, and links take the system's hyperlink colour.
- **Developer mode** (`Chế độ phát triển`, off by default, `dev_mode` in `config.txt`), the last box under `Hệ thống`. The config folder button (`Mở thư mục cấu hình…`) and the hook timing line (`Hook p99: …`) are now shown only while it is on, directly under the box; the window grows to show them. Timings are still recorded either way.
- **Shorter update notice.** While a newer version waits, the line under `Tự động cập nhật` reads `Có bản mới vX.Y.Z.` followed by the link `Cập nhật tại đây`, which does what clicking the tray item does (install, or open the download page); the address is no longer written out. `Đang dùng bản mới nhất.` no longer repeats the version, which the footer shows. The status line takes no room when there is nothing to say and takes only as many lines as the text and the link need. The tray menu item and the notifications are unchanged.
- **Links** (`Cập nhật tại đây`, `GitHub`) are reached with Tab and activated with Space, Enter or a click. The admin-mode copy opens no browser: the `GitHub` link shows the address in a message box, as the folder button shows the path. Enter presses the focused button (`Kiểm tra bản mới`, `Mở thư mục cấu hình…`) too.

## 0.2.0 (2026-11-26)

The owner typed ordinary mixed English and Vietnamese text with this build on one Windows 11 machine and found it good enough to release; a few words still came out wrong. One hook timing reading was taken there (AT7 in `docs/HANDS_ON_TEST.md`): p99 314 µs for keys passed through and 5,600 µs for keys that replace text, over 502 keys. Everything else is covered by the automated tests only: no swap was remembered in that session, and admin mode, the icon checks and the updater (AT10) have not been run on a real machine.

### Engine (`triokey-engine`)

- **English or Vietnamese is now decided by word frequencies and a per-line language mode.** Each word's two readings (the keys as typed, and the composed Vietnamese) are rated by how often they are used, from token counts of web text, and weighed by how Vietnamese the current line looks so far. A fresh line leans Vietnamese; each committed word moves the line towards one language; Enter starts a fresh line. What changes when typing:
  - Words valid both ways follow the line: `the songs␣` stays `the songs`, `tooi thaasy songs␣` gives `tôi thấy sóng`, and `songs␣` on a fresh line now gives `sóng` (it gave `songs` before).
  - As before, English terms inside Vietnamese sentences stay English: `tooi caanf push code leen server␣` gives `tôi cần push code lên server`.
  - The hand-made table of defaults for words valid in both languages and the "previous word" rule are gone; the whole line counts.
- **Doubled letters.** A doubled tone key that cancelled a mark is collapsed when only the single form is a known word (`mixx` gives `mix`, `usser` gives `user`), or, when neither form is known, when the single form is Vietnamese. When both forms are words the keys stay as typed and Backspace now offers the other (`off`, then ⌫ gives `of`). Words outside every table now stay as typed: `passwd`, `Terraform`, `lossless` (0.1.x gave `paswd`, `Teraform`, `losless`).
- **Two capitals are an acronym wherever they stand**: `Mac OS`, `caif OS mowis` (`cài OS mới`) and `team US ddang online` keep them; 0.1.1 composed a two-key capital after another word (`Mac OS` gave `Mac Ó`). A one-letter word is typed with Shift on the letter only and is scored like any word: `chaau As` gives `châu Á`. At the start of a line an acronym stays as typed too (`OS naof cungx dduwowcj` gives `OS nào cũng được`), and longer ALL CAPS words stay unless the Vietnamese word is common (`USA`, `HTML` stay; `VIEEJT` gives `VIỆT`).
- **Abbreviations with `đ`.** A composed form that is no syllable but is common in Vietnamese web text, has two or more letters and a consonant, is kept: `ko ddc ddaau` gives `ko đc đâu`. Lone letters and vowel fragments are not abbreviations: `dd`, `aa`, `aw` typed alone stay as typed, and `dd/mm/yyyy` stays.
- **The Backspace swap is armed only when the engine is unsure.** One Backspace right after a word swaps it to its other reading only when the losing reading is at most 100 times less likely; for a near-certain word it just deletes the space (`tooi␣⌫` gives `tôi`). A common English word that lost to Vietnamese always stays one Backspace away, however Vietnamese the line is. A swap also moves the line mode as if the swapped word had been typed.
- **Remembering kept swaps.** If a Backspace swap is followed by a space, Tab or Enter, the engine remembers it (typed keys → shown text) and uses that reading the next time the same keys are typed, in any letter case. Swapping it back and keeping that changes or forgets the entry. The force files still come first. In the middle of a line that already leans Vietnamese, a remembered English reading does not replace the Vietnamese word; it stays one Backspace away. A swap followed by a letter or punctuation is not remembered. At most 5,000 entries; they never expire.
- **ALL CAPS tones.** `TRUYEENF` gave `TRUỲÊN` (the `vi` crate misplaces the tone in all-caps `uyê`); ALL CAPS keys are now composed in lower case and uppercased, giving `TRUYỀN`.

Behaviour that may surprise (each fixed with one Backspace unless noted):

- The first word of an English line can come out Vietnamese, because a fresh line leans Vietnamese: `There` gives `Thể`, `These` `Thế`, `Did` `Đi`.
- A common English word after several Vietnamese words can come out Vietnamese (`… laanf sets` gives `… lần sét`).
- `w` typed alone now stays `w` (it gave `ư`); Backspace gives `ư`.
- `dd` typed alone still stays `dd`, and Backspace does not swap it; there is no way to get a lone `đ` from `dd`.
- A remembered reading also applies at the start of a line and in ALL CAPS: once `this` → `thí` is remembered, `THIS` gives `THÍ`.

### Windows app (`triokey`)

- **The first word no longer pays for the word tables.** The warm-up before the hook is installed now also parses the frequency tables (about 70 ms), which the first typed word used to do inside the keyboard hook.
- **`learned.txt`.** Remembered words are stored in `%APPDATA%\triokey\learned.txt` (UTF-8, `keys<TAB>text` per line, local only, never sent anywhere). It is written about 2 seconds after the last change, on exit and at logoff or shutdown; up to 2 seconds of changes are lost if TrioKey is killed. Delete a line to forget a word, while TrioKey is not running. A file that cannot be read (not UTF-8, over 256 KB, locked) is left alone and words are kept in memory only.
- **Hardened write for the elevated copy.** In admin mode TrioKey writes `config.txt` and `learned.txt`, and reads `learned.txt`, only through one checked open: the folder is held open so it cannot be renamed or swapped, it must be a plain folder whose final path is the expected one (a junction anywhere higher up the path is refused), and the file must be an existing regular file with one name. It still creates no files: the normal copy creates `learned.txt` at start and again right before admin mode is turned on, and if the file is missing the admin copy keeps words in memory only. If any part of the profile path goes through a junction, the admin copy keeps settings and words in memory only. Tested on temporary folders only.
- **New icons.** The tray letter is drawn with rounded strokes on a transparent background: red `V` while typing Vietnamese, blue `E` when keys pass through, in a deep shade (#B23B33, #3565B8) on a light taskbar and a light shade (#F28B82, #8AB4F8) on a dark one, redrawn when the Windows theme changes. The app icon is a blue tile (#3565B8) with three white keys, with 18 pixel-aligned frames from 16 to 256 px. The settings window icon follows the window's DPI and is reloaded when it moves to a monitor with another scale.

### Project

- Word data: `freq_vi.tsv` and `freq_en.tsv`, token counts derived from samples of FineWeb-2 (`vie_Latn`) and FineWeb (`sample-10BT`) (ODC-By 1.0; counts only, no text), built by the scripts in `tools/corpus/`; `en_tech.txt`, developer terms written for this project. `en_words.txt` (SCOWL) stays. `ambiguous_defaults.txt` and its generator are removed.
- Typing benchmark (`crates/triokey-engine/tests/realtext.rs`): types whole lines of mixed English and Vietnamese and counts wrong words and the keystrokes spent fixing them. Line sets written for this project are in `tests/data/mixed/` (for tuning) and `tests/data/final/` (blind). On the blind sets (10,274 words, written by people who never saw the engine), compared with the previous engine: wrong words 0.54% → 0.16%, keystrokes spent fixing per 100 words 2.74 → 0.59; with remembering on, 16 → 12 wrong words and no word had to be fixed twice.
- VieLotus-IM regression suite (hard single words without context, a guardrail rather than a measure of real typing): 8,361 → 8,524 of 9,084 (93.8%).
- `docs/ARCHITECTURE.md` (design, threads, data, elevated copy, updates, release flow, tests) and `SECURITY.md` added.

## 0.1.1 (2026-11-25)

### Windows app (`triokey`)

- Keys that Windows flags as injected by a lower-integrity process (`LLKHF_LOWER_IL_INJECTED`) pass through untouched and drop the word being typed, so an elevated TrioKey never turns such keys into text. Effect: while TrioKey runs elevated (admin mode), tools that inject keys from a normal process (AutoHotkey, on-screen keyboards) get no Vietnamese. Not yet run on a real machine.
- App icon for the exe (Explorer, taskbar, settings window). `build.rs` writes one COFF `.rsrc` object holding the icon group and the manifest, so no `windres` is needed; building now needs the `x86_64-pc-windows-gnu` toolchain. The settings window uses this icon instead of the V drawn at run time. The generated images are committed in `assets/brand/` (`triokey.ico`, `icon-512.png`, `social-preview.png`). The tray icons (monochrome `V` / `E`) are unchanged. Not yet run on a real machine.

### Project

- Releases no longer wait for a manual approval: pushing a tag `vX.Y.Z` that points at a commit on `main` builds, signs and publishes that version. The release workflow now first checks that the tagged commit is on `main`. Anyone who can push such a tag can publish a signed update to every installed copy; see `docs/RELEASING.md`.

## 0.1.0 (2026-11-25)

First public version. Released on GitHub (`triokey.exe`, `triokey.exe.minisig`, a zip and `SHA256SUMS.txt`), built and signed by the release workflow; tried on Windows 11 only. See the Status section of the README for what has and has not been verified.

### Engine (`triokey-engine`)

- Telex input with both tone-placement styles (`hòa` / `hoà`), using the `vi` crate for composition.
- Whole-word recompute: after each key the engine returns one edit (delete N characters, insert text).
- Per-word English/Vietnamese decision with no mode: Vietnamese syllable check, English word list (SCOWL), defaults for words valid in both languages, and the previous word as context.
- Code is left as typed: words with digits, `_` or mixed case, paths, flags, text right after `.` `:` `\`.
- Doubled tone key rules (`terrible` stays, `mixx` → `mix`). Known miss: a word outside the English list loses a letter (`passwd` → `paswd`).
- One Backspace right after a word flips the reading of a word that has two readings, or of a doubled-key word when exactly one of its two forms is in the English list.
- User files `force_en.txt` and `force_vi.txt`.
- Measured against VieLotus-IM's public regression suite, every row typed without context: 8,361 / 9,084 (92.0%).

### Windows app (`triokey`)

- Tray app built on a low-level keyboard hook, a low-level mouse hook (clicks only) and `SendInput`; no keyboard layout, text service or language-list change.
- Steps aside for a CJK IME by reading its open/closed state per window; types Vietnamese in direct input under a CJK layout. An earlier build was hand-tested with Japanese (Microsoft IME) and passed AT1 and AT2 of `docs/HANDS_ON_TEST.md`; the current build has not been re-run. For Chinese and Korean only the decision rule is unit-tested.
- Plain-text config in `%APPDATA%\triokey`.
- Interface in Vietnamese only.

Implemented, not yet run on a real machine:

- Tray icon `V` / `E`, left click to switch on or off, hotkey presets (Ctrl+Shift, Alt+Z, Ctrl+Space, none).
- Skipping of classic Win32 password fields (Edit/RichEdit controls with `ES_PASSWORD`).
- Settings window, start with Windows and optional admin mode.
- Signed automatic updates from this project's GitHub releases: minisign (Ed25519) signature checked with a public key compiled into the exe, no downgrade, install on click only in admin mode, setting `auto_update` (`Tự động cập nhật`, on by default) and a check-now button. **This changes an earlier promise**: TrioKey used to say it never opens a network connection. It now connects to GitHub, and only to GitHub, to check for and download signed updates; it sends no typing data and no identifier, and with the setting off it uses no network. This version carries the project's update key, so it checks for updates by default.
- Embedded application manifest: current visual style for the settings window (Common Controls 6), per-monitor DPI awareness (v2), `asInvoker`.

### Project

- CI on every push to `main` and every pull request (format, clippy, tests, release build), and a release workflow that builds, signs and publishes a version from a `vX.Y.Z` tag. This version was published by it.

### Not in this version

- VNI and other input styles, macros, per-app memory.
- Special handling for terminals and for fields that insert suggestions (browser address bar, Excel cells).
- Detection of password fields in browsers and Electron apps.
- macOS and Linux front-ends.
