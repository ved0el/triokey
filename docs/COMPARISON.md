# TrioKey and similar input methods

**English** · [Tiếng Việt](COMPARISON.vi.md)

As of November 2026.

This page compares TrioKey with three open-source Vietnamese input methods it learned from. It is written by the TrioKey side, so read it with that in mind. Every statement about another project names the file it was read from, at the commit listed below. Where something was not checked, the cell says "not checked" instead of guessing. Corrections are welcome: open an issue.

All three projects are further along than TrioKey and are already distributed: VKey is on winget, VieLotus-IM is in the Microsoft Store and has Debian packaging, and Vikey ships numbered versions with an installer and a self-updater. TrioKey is a pre-release with no binaries yet, and much of its Windows app has not been run on a real machine yet (see the Status section of the [README](../README.md)).

## What was read

| Project | Repository | Commit read | Licence |
| --- | --- | --- | --- |
| VKey | https://github.com/phatMT97/VKey | `da22eab` (2026-11-19) | GPL-3.0 (`LICENSE`), with the exception its `README.md` describes (see Sources) |
| Vikey | https://github.com/tonyminibot/Vikey | `c9007fe` (2026-11-21) | GPL-3.0 (`LICENSE`, `README.md`) |
| VieLotus-IM | https://github.com/stevedat/VieLotus-IM | `3519761` (2026-11-25) | MIT (`LICENSE`) |

Only the repositories were read. Release pages, stores and issue trackers were not. None of the three programs was installed or run for this comparison, so everything below describes documented or coded behaviour, not observed behaviour.

UniKey, EVKey and OpenKey are not compared. In general terms they are the long-standing choices, built around a Vietnamese/English mode with a toggle. They were not read for this page, so whatever else they do is not described here.

## The table

| | TrioKey | VKey | Vikey (Windows app) | VieLotus-IM (Windows adapter) |
| --- | --- | --- | --- | --- |
| Language | Rust | C++20 | C++ engine and Windows app (Swift app on macOS) | C++20 engine and adapter (Swift engine on macOS) |
| Licence | MIT | GPL-3.0; an optional closed-source spell-check library is outside it (the Classic edition is fully open source). Rewritten from a fork of OpenKey | GPL-3.0, based on OpenKey | MIT |
| How it plugs into Windows | low-level keyboard hook (plus a mouse hook for clicks) + `SendInput`; nothing registered by default | keyboard hook as the main path; an optional TSF component that registers as a Windows input method | low-level keyboard hook + `SendInput` | TSF text service, registered under Vietnamese in the Windows input list |
| Typing next to a CJK IME | reads the IME's open/closed state per window; types Vietnamese in direct input under a CJK layout (hand-tested with Japanese on an earlier build only) | switches itself to English mode while the layout's language is Japanese, Chinese or Korean; back to Vietnamese on leaving it (a setting; apps locked to Vietnamese and excluded apps are exempt) | passes keys through while the layout's language is anything other than English or Vietnamese (a setting) | is a separate input-list entry; how it coexists with a CJK IME was not checked |
| English words in Vietnamese text | no mode; each word decided on its own | V/E mode with a toggle hotkey and per-app memory; spell check restores the typed keys of a word that is not Vietnamese | automatic: fixes the word and switches the mode; undo with Backspace (two presses when the fix came on Space) | automatic bilingual handling |
| Words valid in both languages | word list + previous word; Backspace flips | not checked | word statistics, including word pairs; the neighbouring words decide | handled; 369/369 (C++ engine; 362/369 on the Swift engine) when its harness replays the collision words of its suite without context |
| Input styles | Telex | Telex, VNI, Simple Telex, combined Telex+VNI | Telex, Simple Telex 1 and 2; no VNI | Telex, VNI |
| Legacy code tables | no | yes (TCVN3, VNI Windows…) | no (removed) | not checked |
| Macros / text expansion | no | yes | no (removed) | yes |
| Per-app memory | no | yes; per-domain rules through an experimental browser extension | yes, per app and per website | not checked |
| Elevated (admin) windows | optional admin mode: logon task, exe in an admin-only folder (implemented, not yet run on a real machine) | can run as administrator | not found in the Windows source | not checked |
| Network use | signed update check against GitHub only, can be switched off (implemented, not yet run) | update checker | self-updater; crash reports that ask first | not checked for Windows |
| Platforms | Windows 11 (Windows 10 untried) | Windows 10/11 | macOS, Windows, Linux, iOS, Android | macOS, Windows, Linux, web (WASM), iOS, Android |
| Distribution | build from source; a signed-release pipeline with a built-in updater exists, no release published yet | releases, winget | installer script and self-updater in the repository; numbered versions (`DECISIONS.md` 61 names 1.0.33); release page not read | releases, Microsoft Store |

## Where TrioKey is different

- **It reads the IME's state, not only the layout.** Under a Japanese layout with the IME closed, TrioKey types Vietnamese; with hiragana on, it steps aside. With their setting on, VKey and Vikey step aside whenever the CJK layout is active, whether or not the IME is composing (VKey exempts apps locked to Vietnamese). In apps that hide the IME state, TrioKey does the same as they do.
- **No Vietnamese/English mode.** There is an on/off switch, but ordinary mixed typing does not use it. Vikey and VieLotus-IM also handle English automatically; the difference is that TrioKey has no mode to switch at all.
- **Nothing registered in Windows** by default: no TSF service, no layout, no language-list entry. Optional start-with-Windows adds one Run value; optional admin mode adds one task and one folder.
- **Small on purpose**: one settings window, a text config, and no network code beyond the update check (two fixed GitHub URLs, nothing sent).
- **MIT, with an engine crate that has no OS calls**, so it can be reused elsewhere.

## Where the others are better

- **Maturity and distribution.** VKey is on winget, VieLotus-IM is in the Microsoft Store and has Debian packaging, and Vikey ships numbered versions with an installer and a self-updater. TrioKey has only been typed on by its author, on an earlier build, and has no binary release.
- **Automatic English/Vietnamese quality.** Vikey decides ambiguous words from word and word-pair statistics built from large corpora and tunes against a locked test set (`DECISIONS.md` 29, 30, 34; `ACKNOWLEDGEMENTS.md`). VieLotus-IM reports 9,091/9,091 on its suite as its own harness runs it (English context primed for ambiguous words) and 362/369 (Swift) or 369/369 (C++) on the collision words of that suite without context; TrioKey scores 92.0% running every row without context. TrioKey uses a word list, a hand-tuned table of defaults and the previous word only.
- **Input styles.** VKey and VieLotus-IM support VNI; VKey also has Simple Telex, a combined mode and legacy code tables. Vikey has Simple Telex variants. TrioKey is Telex only.
- **Macros.** VKey and VieLotus-IM have them. TrioKey does not and does not plan to.
- **Per-app behaviour.** VKey and Vikey remember the mode per app; Vikey also per website, and VKey has per-domain rules through an experimental browser extension. VKey also has per-app configuration, app exclusion and a game mode. TrioKey has none of this.
- **Platform coverage.** Vikey and VieLotus-IM cover macOS, Linux and mobile; VKey supports Windows 10 as well as 11. TrioKey has only been tried on Windows 11.
- **The TSF route.** A text service edits text through the application instead of sending Backspaces, which avoids a class of problems hook-based tools have with fields that insert suggestions and with some apps. VieLotus-IM is built on it and VKey offers it per app. TrioKey chose not to register a text service; the cost is that such fields need special handling it does not have yet.
- **Signing and packaging.** VKey is on winget with project-signed update packages; VieLotus-IM is in the Microsoft Store. TrioKey has no package and no release yet; its updater accepts only files signed with the project's minisign key, but the exe carries no Authenticode signature, so Windows treats it as unsigned.
- **Password fields and terminals.** TrioKey is only designed to skip classic Win32 password fields (not yet verified by hand) and has no terminal handling. How the other three treat these was not checked, so they may well do better.
- **Interface language.** TrioKey's interface is in Vietnamese only.

## What TrioKey took from them

- From Vikey's published decisions: the behaviour of a doubled tone key and of giving the typed keys back. Behaviour only; no code, data or tests.
- From VieLotus-IM (MIT): the regression suite, the SCOWL-based word list and parts of its lexicon, credited in [NOTICE.md](../NOTICE.md) with its licence text.
- From VKey: the baseline behaviour "step aside on a CJK layout", which TrioKey still falls back to when an app hides its IME state. Behaviour only, read from its README and source; no code.

## TrioKey on VieLotus-IM's test suite

TrioKey runs VieLotus-IM's `test_suite.csv` (copied unchanged from commit `3519761`, MIT) to get a figure for reference, not as a test that must pass. The two results below come from the same file but are **not measured the same way**, so they are not like for like:

| | Result | How the rows are run |
| --- | --- | --- |
| VieLotus-IM (its own figures) | 9,091 / 9,091 | its own harness: the 1,033 `ambiguous_needs_context` rows are typed after an English phrase ("he is"), and `transform` rows run with bilingual handling off (`Sources/VieLotusCLI/main.swift`) |
| VieLotus-IM, collision words without context | 362 / 369 (Swift), 369 / 369 (C++) | the same harness replaying the collision rows with no context (`README.md`, `docs/TEST_STATUS.md`, `docs/BENCHMARK_REPORT.md`) |
| TrioKey 0.1.0 | 8,361 / 9,084 (92.0%); 9 rows with spaces skipped | every row typed into a fresh engine with default settings and no context (`crates/triokey-engine/tests/vielotus.rs`) |

Two caveats. First, the row counts differ and the reason was not found: the copied file has 9,093 data rows (9,084 run + 9 skipped), while VieLotus-IM's published total is 9,091. Second, VieLotus-IM's README says its numbers are regression-suite results, not a claim of error-free typing; the same holds for TrioKey's.

The suite encodes VieLotus-IM's choices, so the gap is partly quality, partly disagreement and partly the missing context. Some TrioKey misses are deliberate: it keeps `less` and `off` as typed where the suite expects `les` and `of`, and it defaults `has`, `is`, `of` to English where the suite expects the Vietnamese reading without context. Others are plain weaknesses: a doubled tone key in a word outside the English list drops a letter (`passwd` → `paswd`), and many words that exist in both languages are decided by a hand-tuned default rather than by statistics. The full breakdown is in [decisions/0004-regression-baseline.md](decisions/0004-regression-baseline.md) and [decisions/0005-tie-defaults.md](decisions/0005-tie-defaults.md).

## Sources, row by row

### VKey

- **C++20, GPL-3.0, Windows 10/11, rewritten from an OpenKey fork**: `README.md` (introduction, requirements, architecture table). The introduction says VKey was rewritten from NextKey, a fork of OpenKey, and that its engine consulted UniKey's source for phonology rules.
- **Licence exception**: `README.md`, privacy section: since v4.3 an optional closed-source library written in Rust provides the advanced spell-check; it is off by default, and the officially released Classic edition is 100% open source and cannot load it.
- **Hook main path with optional TSF**: `README.md` architecture table (the app hosts the hook engine; `VKeyTSF` is a DLL that registers as a Windows input method) and `docs/GUIDE.md` section 11, which says the hook stays the main input path unless an app is added to the TSF list.
- **CJK by layout language**: `README.md` ("Auto-disable on CJK — disables Vietnamese input when keyboard layout is CJK"), `docs/GUIDE.md` section 5, and the code: `src/app/system/HookEngine.cpp`, `IsIncompatibleLayout`, tests the layout's primary language against Japanese, Chinese and Korean; `src/core/CjkSwitchDecision.h` describes the state machine in its header comment, including the user setting (`cjkAutoSwitchEnabled`) and the two exemptions (`isExcluded`, `isForcedVietnamese`). A search of the hook-engine sources for the IME open-status APIs (`ImmGetOpenStatus`, `WM_IME_CONTROL`, `ImmGetDefaultIMEWnd`, the TSF keyboard open/close compartment) found no use of them for this decision.
- **V/E mode, toggle, per-app memory, per-domain rules**: `README.md` (Smart Switch per app; "VKey Browser", marked experimental, applies a mode per domain), `docs/GUIDE.md`.
- **Spell check restores typed keys**: `docs/GUIDE.md` section 9, "restore word with wrong key", on by default: when a word fails the Vietnamese spelling check and Space is pressed, the original keys come back. `README.md` also lists English detection.
- **Input styles, code tables, macros**: `README.md`, English feature list.
- **Run as administrator**: `docs/GUIDE.md` (the note on games that run elevated) and `src/app/system/StartupHelper.h` (admin restart, Task Scheduler start-up).
- **Update checker, winget**: `README.md`.

### Vikey

- **GPL-3.0, based on OpenKey, platforms**: `README.md`.
- **C++ engine and Windows app**: `Engine/*.cpp`, `Windows/App/*.cpp`; the macOS build in `README.md` uses Xcode.
- **Hook + `SendInput`**: `Windows/App/Core.cpp` (`SetWindowsHookExW(WH_KEYBOARD_LL, …)`, `SendInput`).
- **Steps aside by layout language**: `Windows/App/Core.cpp`, `updateOtherInputLanguage`: the foreground thread's layout language is compared with English, Vietnamese and neutral, and the hook passes keys through otherwise, when the `vOtherLanguage` setting is on. `README.md`: "steps aside automatically for other input languages (Japanese, Chinese…)". A search of `Windows/` and `Engine/` for the IME open-status APIs found nothing.
- **Automatic English/Vietnamese, programming words left alone**: `README.md` feature list; `DECISIONS.md` (for example 22, 30 and 34 on ties and word pairs).
- **Undo with Backspace**: `DECISIONS.md` 61: after a fix made on Space, the first Backspace takes the space and the second undoes the fix; the entry states the cost as two presses instead of one.
- **Telex, Simple Telex; no VNI, code tables or macros**: `Windows/App/Settings.cpp` offers Telex, Simple Telex 1 and Simple Telex 2 only; a comment in `Windows/App/Prefs.cpp` says VNI is not in Vikey; `DECISIONS.md` 56 (Simple Telex) and its list of removed features (VNI, code tables other than Unicode, macros).
- **No admin mode found**: `Windows/App/Vikey.manifest` requests `asInvoker`; a search of `Windows/App/*.cpp` for an elevation path found none. (`Windows/PLAN.md`, a planning note, mentions admin mode as a proposal.)
- **Per-app and per-website memory**: `README.md`.
- **Self-updater, crash reports that ask first**: `Windows/App/Updater.cpp`, `Windows/App/CrashReporter.cpp`. An opt-in typing log for tuning is described in `README.md`, off by default.
- **Installer, versions**: `Windows/installer/Vikey.iss`; `DECISIONS.md` 61 ("Shipped in 1.0.33").

### VieLotus-IM

- **MIT, platforms, languages, downloads**: `README.md` (platform table: releases, Microsoft Store, Debian packaging), `docs/ARCHITECTURE.md`.
- **Windows adapter is a TSF text service**: `README.md` ("C++20 engine and TSF shell"), `docs/ARCHITECTURE.md`, and `CrossPlatform/windows-tsf/README.md`, which describes an in-process COM text input processor registered under `vi-VN` that appears in the Windows input list after registration.
- **Telex and VNI, macros**: `CrossPlatform/windows-tsf/src/PrefsStore.h` (`isVni`, `macroEnabled`); `PRIVACY.md` mentions macros.
- **Regression suite and its own results**: `Sources/VieLotusCLI/Resources/test_suite.csv`; `README.md` and `docs/TEST_STATUS.md` report 9,091/9,091 on the primary suite and 362/369 (Swift) or 369/369 (C++) on the no-context replay of the collision words; `Sources/VieLotusCLI/main.swift` shows how each kind of row is run.
- **Network**: `README.md` says typing is processed locally and that the macOS updater contacts GitHub; the Windows adapter was not checked.

### TrioKey

- Everything in the TrioKey column comes from this repository: the [README](../README.md), [docs/decisions/](decisions/) and the source in `crates/`.
