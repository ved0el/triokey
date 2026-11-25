# Changelog

## 0.1.0 (unreleased)

First public version. Pre-release: no binaries, build from source, tried on Windows 11 only. See the Status section of the README for what has and has not been verified.

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
- Steps aside for a CJK IME by reading its open/closed state per window; types Vietnamese in direct input under a CJK layout. An earlier build was hand-tested with Japanese (Microsoft IME); the results were not recorded and the current build has not been re-run. For Chinese and Korean only the decision rule is unit-tested.
- Plain-text config in `%APPDATA%\triokey`.
- Interface in Vietnamese only.

Implemented, not yet run on a real machine:

- Tray icon `V` / `E`, left click to switch on or off, hotkey presets (Ctrl+Shift, Alt+Z, Ctrl+Space, none).
- Skipping of classic Win32 password fields (Edit/RichEdit controls with `ES_PASSWORD`).
- Settings window, start with Windows and optional admin mode.
- Signed automatic updates from this project's GitHub releases: minisign (Ed25519) signature checked with a public key compiled into the exe, no downgrade, install on click only in admin mode, setting `auto_update` (`Tự động cập nhật`, on by default) and a check-now button. **This changes an earlier promise**: TrioKey used to say it never opens a network connection. It now connects to GitHub, and only to GitHub, to check for and download signed updates; it sends no typing data and no identifier, and with the setting off it uses no network. The repository carries a placeholder key until the first release, and a build with the placeholder never connects.
- Embedded application manifest: current visual style for the settings window (Common Controls 6), per-monitor DPI awareness (v2), `asInvoker`.

### Project

- CI on every push to `main` and every pull request (format, clippy, tests, release build), and a release workflow that builds, signs and publishes a version from a `vX.Y.Z` tag. Neither had run before the first push.

### Not in this version

- VNI and other input styles, macros, per-app memory.
- Special handling for terminals and for fields that insert suggestions (browser address bar, Excel cells).
- Detection of password fields in browsers and Electron apps.
- macOS and Linux front-ends.
