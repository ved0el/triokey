# Changelog

## 0.1.1 (2026-11-25)

### Windows app (`triokey`)

- Keys that Windows flags as injected by a lower-integrity process (`LLKHF_LOWER_IL_INJECTED`) pass through untouched and drop the word being typed, so an elevated TrioKey never turns such keys into text. Effect: while TrioKey runs elevated (admin mode), tools that inject keys from a normal process (AutoHotkey, on-screen keyboards) get no Vietnamese. Not yet run on a real machine.
- App icon for the exe (Explorer, taskbar, settings window). `build.rs` writes one COFF `.rsrc` object holding the icon group and the manifest, so no `windres` is needed; building now needs the `x86_64-pc-windows-gnu` toolchain. The settings window uses this icon instead of the V drawn at run time. Source files are in `assets/brand/` (`make.py` generates `triokey.ico`, `icon-512.png` and `social-preview.png`). The tray icons (monochrome `V` / `E`) are unchanged. Not yet run on a real machine.

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
