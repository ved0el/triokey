# TrioKey architecture

This document explains how TrioKey is built and why, and where each part lives. It describes
the code as it is. Section numbers and titles are stable: source comments refer to them as
"docs/ARCHITECTURE.md, <number> <title>" (for example "docs/ARCHITECTURE.md, 8. Elevated copy").

What has been run on a real machine: an earlier build was hand-tested with the Japanese
Microsoft IME (checklist rows AT1 and AT2 passed). The owner typed ordinary mixed English and
Vietnamese text with the 0.2.0 build (the frequency scoring engine) and found it good enough to
release, with a few words still wrong, and took one hook timing reading (AT7: p99 314 µs for
keys passed through, 5,600 µs for keys that replace text, 502 keys). Remembering of kept swaps
(no swap was kept in that session), the checked file access of the elevated copy, the tray and
app icons and the save at logoff have so far been checked only by `cargo test` and compilation. Chinese and Korean have
never been tried by hand. The other checklist rows (among them hook reinstall, admin mode and the
update swap, and AT11 and AT12 for the new engine and remembering) have no recorded result. Statements below that depend on a real desktop say so.

## 1. Goals and constraints

### 1.1 Goals and hard constraints

- **No keyboard layout, language or text service is registered.** TrioKey never touches
  `HKCU\Keyboard Layout` or the Windows language list. It adds nothing to Windows except, if the
  user asks, a Run value (start with Windows) or a scheduled task (admin mode).
- **Steps aside for Chinese, Japanese and Korean IMEs.** When a CJK IME is composing, every key
  goes to it untouched. When the same layout is in direct input, TrioKey types Telex.
- **No English/Vietnamese toggle.** The user types Telex all the time; each word is decided as
  English or Vietnamese when it ends. A hotkey only switches TrioKey off and on.
- **Small settings surface.** One settings window (on/off, hotkey preset, tone style, keep
  English words, yield to CJK IMEs, start with Windows, admin mode, automatic updates) plus a few
  plain text files (section 7). No macros, no per-app memory, no key recorder.
- **The engine is plain Rust with no OS calls**, so all typing behaviour is testable with
  `cargo test` and the Windows part stays thin.
- **Nothing typed is sent anywhere.** The only typed text written to disk is `learned.txt`:
  words the user swapped with Backspace and kept, as keys and shown text (section 5.5). The only
  network use is the signed update check (section 9), and it can be switched off.

### 1.2 What follows: a low-level hook, not a text service

A TSF text service would have to be registered as a keyboard in the language list, which the
first constraint forbids, and only one TSF input processor is active at a time, so the CJK IME
would have to make room for it. TrioKey therefore:

- reads keys with a low-level keyboard hook (`WH_KEYBOARD_LL`) in its own process,
- asks the engine what the word on screen should become,
- swallows the key and injects the difference (Backspaces plus Unicode characters) with
  `SendInput`.

The costs of this choice are accepted and handled elsewhere: the screen is edited by deleting
and re-inserting (section 4), the IME state of another process must be read from outside
(section 3), the hook can be dropped by Windows (section 4.4), and typing into windows of a
higher integrity level needs an elevated copy (section 8).

### 1.3 Where the code lives

| Path | Role |
|---|---|
| `crates/triokey-engine/src/engine.rs` | `Engine`: whole-word recompute, scoring, line mode, Backspace swap, remembering hook-in |
| `crates/triokey-engine/src/lexicon.rs` | Frequency tables, floors, force files, `Learned` (remembered swaps) |
| `crates/triokey-engine/src/telex.rs` | Telex composition through the `vi` crate, ALL-CAPS fix |
| `crates/triokey-engine/src/syllable.rs` | Vietnamese syllable set built from rules, prefixes, tone-free table keys |
| `crates/triokey-engine/src/token.rs` | Code-cluster starters, mixed-case test |
| `crates/triokey/src/main.rs` | Entry point; `--install-admin` / `--uninstall-admin` |
| `crates/triokey/src/app.rs` | Startup, settings in memory, every read and write of the user data files (`%APPDATA%\triokey`) |
| `crates/triokey/src/hook.rs` | Hook thread, key and mouse callbacks, hotkey, latency rings, reinstall |
| `crates/triokey/src/keymap.rs` | Classifies a key event (letter, boundary, Backspace, word break, possible IME toggle) |
| `crates/triokey/src/inject.rs` | Builds `SendInput` records, the own-event marker |
| `crates/triokey/src/ime.rs` | Pure rules: when to yield to a CJK IME |
| `crates/triokey/src/ime_watch.rs` | Watcher thread: IME state, password flag, focus changes, timer duties |
| `crates/triokey/src/config.rs` | `config.txt` format |
| `crates/triokey/src/tray.rs`, `icon.rs`, `settings_ui.rs` | Tray icon and menu, runtime-drawn tray glyphs, settings window |
| `crates/triokey/src/admin.rs`, `autostart.rs` | Admin mode (Program Files copy, scheduled task), Run value |
| `crates/triokey/src/update.rs` | Signed self-update |
| `crates/triokey/build.rs` | Embeds the app icon and the manifest |
| `crates/xtask` | Release tool: `keygen`, `sign`, `verify` (not shipped) |
| `tools/corpus` | Builds the frequency tables and the web benchmark sets |

## 2. Process and threads

### 2.1 Process

`triokey.exe` is one background process with a tray icon. `main` first looks at the first
argument (`--install-admin`, `--uninstall-admin`: the elevated helper of section 8), before the
single-instance check. A normal start takes the named mutex `Local\triokey-single-instance`;
if another copy (normal or elevated) holds it, the new copy retries for about 3 s and then exits.
The retry lets a copy that hands over to the admin copy quit first.

`app::run` loads the settings, the force files and `learned.txt`, builds the `Lexicon`, starts
the threads below and then runs the tray message loop. Leaving that loop ends the process, which
removes the hooks.

### 2.2 Who owns what

| Thread | Owns | Started in |
|---|---|---|
| Tray / UI (main thread) | Tray icon and menu, settings window, admin-mode switch (it blocks here while the elevated helper runs, up to 2 minutes), final save of `learned.txt` | `app::run`, `tray::run` |
| Hook | The `Engine` (thread-local), both low-level hooks, the hotkey state, the latency rings | `hook::spawn` |
| Watcher | Cached IME state, foreground layout language, password flag, word-break flag; hook liveness check; periodic save of `learned.txt` | `ime_watch::spawn` |
| Updater | All network and update file work; exists only when a real update key is compiled in | `update::spawn` |

Threads talk through atomics and posted messages. The hook thread reads the settings (an
`RwLock` copy) and the watcher's atomics; apart from short locks it never waits for another
thread, except in the first-key wait (section 3.3). Settings changes reach the engine by rebuilding it on the hook
thread (`hook::reconfigure` posts a message; the engine has no setter), so every change in the
settings window drops the current word, the line mode and any pending swap. The remembered-word map
is shared by `Arc<Mutex<…>>`; the lock is held for one lookup, insert or render.

### 2.3 What may run inside the hook callback

Allowed: classify the key, read cached state, run the engine, call `SendInput` once (the
inserted text must come before any later key, so it is sent from the callback itself), record
the callback time. Not allowed: file access, network, waiting on locks held for long, asking the
IME directly. The one exception is the bounded first-key wait in section 3.3.

Order inside `kb_proc` (and `on_key`): own injected events pass at once; keys injected by a
lower-integrity process pass and reset the word; then the hotkey; key-up events return here;
`VK_PACKET` resets the word and passes; the key is classified (a possible IME toggle or a word
break also asks the watcher for a recheck); the watcher's word-break flag is consumed (it resets the word when set); then the
on/off and password checks; then the first-key wait and the yield check; then the engine. The
engine's `RefCell` borrow is released before `SendInput`, because `SendInput` re-enters the hook
with the marked events.

The character a key types comes from `ToUnicodeEx` on the foreground thread's keyboard layout
(`hook::to_char`), with a modifier state built from `GetAsyncKeyState`; the Caps Lock toggle is
read with `GetKeyState` on the hook thread, which is marked as a shortcut. `keymap::classify`
passes only ASCII letters to `Engine::key`; a letter from another layout (`ú`) or a control
character resets the word.

### 2.4 Latency budget and measurement

The budget is a p99 below 1 ms for keys that pass through, checked by hand in checklist row AT7.
The code only measures: two rings of the last 1024 callback times (keys that called `SendInput`,
time included, and keys that did not), shown at the bottom of the settings window. A key that
waited for a fresh IME read (section 3.3) is not recorded.

### 2.5 Warm-up

`hook::spawn` types one throwaway word (`a`, then a space) through a throwaway engine before
installing the hook, which builds the syllable tables, and calls `Lexicon::rates` once, which
parses the four data files into the frequency tables (about 70 ms in a release build). The word
`a` alone never reaches `rates`. After both, the first word typed costs tens of microseconds
(measured: `tieesng ` about 42 µs after the full warm-up, about 70 ms with the word alone).

## 3. IME state and yielding

### 3.1 Reading the state from another process

The IME belongs to the foreground application, not to TrioKey. The watcher thread
(`ime_watch.rs`) reads it and caches the result; the key path only reads the cache.

One read (`refresh_once`):

1. Foreground window, its thread, and that thread's focused window (`GetGUIThreadInfo`).
2. Password flag: the focused control's class contains `edit` and has `ES_PASSWORD`.
3. Layout language: the low 10 bits of `GetKeyboardLayout(thread)`.
4. If "yield to CJK IMEs" is off or the language is not Japanese, Chinese or Korean: type.
5. Otherwise `ImmGetDefaultIMEWnd(focus)` and `SendMessageTimeoutW(WM_IME_CONTROL,
   IMC_GETOPENSTATUS)` with `SMTO_ABORTIFHUNG` and a 50 ms timeout; for Chinese and Korean with
   the IME open, also `IMC_GETCONVERSIONMODE`.
6. `ime::decide` turns that into Yield or Type; the result is kept per top-level window (the
   cache is cleared past 256 windows).

Reads happen on foreground and focus WinEvents, on a 100 ms timer, and 15 ms after the hook
reports a key or click that may have toggled the IME (the application handles the toggle key
after the hook sees it, so an immediate read would return the old state). A change of focus or
of the yield state drops the word being typed. The tray icon is redrawn when the yield state
flips, or when the layout language changes while yielding (the tooltip names the language).

### 3.2 Rule per language

The rule is `ime::decide` in `crates/triokey/src/ime.rs`.

| Layout | Yield when | Failed read |
|---|---|---|
| Japanese | open status is 1 (conversion mode not used) | last known state of that window, else yield |
| Chinese, Korean | open status is 1 and the conversion mode has `IME_CMODE_NATIVE` | same; an open IME whose conversion mode cannot be read counts as a failed read |
| Any other | never; no read is made | n/a |

Before the first read the state is Yield. A window whose IME cannot be read at all therefore
behaves as if the IME were composing: TrioKey steps aside there.

### 3.3 The first-key wait

A key that types nothing (F-keys, Kanji, IME keys), a shortcut, a cursor key or a mouse click
marks the moment and asks the watcher for a recheck. The next letter, boundary or Backspace may
then wait for that recheck, at most `FIRST_KEY_WAIT_MS` = 30 ms (`ime.rs`), with
`park_timeout`; the watcher unparks the hook thread when the read is done. Past the limit the
cached value is used. `park_timeout` can overshoot by a timer tick; this is marked as a shortcut in
`hook.rs`. Correct state is preferred over latency here.

### 3.4 What is tested

`ime::decide`, `is_cjk` and the password test have unit tests. By hand, only Japanese with
Microsoft IME on an earlier build (AT1, AT2). The 20-out-of-20 first-key test (AT3) has no
recorded result. Chinese and Korean (checklist section AT2-CK) have never been run.

## 4. Text injection

### 4.1 SendInput and the own-event marker

An engine `Edit` (delete N characters, insert a string) becomes N Backspace down/up pairs
followed by one `KEYEVENTF_UNICODE` down/up pair per UTF-16 unit (`inject::build_inputs`). The
original key is swallowed. Output is precomposed Unicode, so one character is one Backspace.

Every injected event carries `dwExtraInfo = TRIOKEY_MARK` (`0x45564A31`, a fixed value that must
not change). The hook passes marked events through before touching any state.

Enter and Tab sent as Unicode characters do not act as Enter or Tab in most applications, so
when a word is replaced at an Enter or Tab, the word is sent as text and the key itself as a
real virtual-key press (`hook::split_boundary`). The Alt+Z hotkey, which is swallowed, is
followed by an unassigned virtual key (`0xE8`) so the application does not open its menu bar
when Alt is released.

### 4.2 Keys from other software

- `VK_PACKET` (Unicode typed in by other software) resets the word and passes.
- Keys flagged `LLKHF_LOWER_IL_INJECTED` (injected by a process of lower integrity) pass
  untouched and reset the word, in every mode. An elevated TrioKey must never turn them into
  text that reaches admin windows. Cost: while TrioKey runs elevated, tools such as on-screen
  keyboards or AutoHotkey running unelevated get no Vietnamese.
- Other injected keys are treated like real ones. Two Vietnamese input methods running at once
  will therefore edit each other's output.
- Ctrl+Alt with a character (AltGr) is a character, but the word is reset and the key passes:
  injecting Backspace while Ctrl and Alt are held is not safe.

### 4.3 Password fields

When the watcher has flagged the focused control as a Win32 password edit, every key passes
untouched. Password fields in browsers and Electron apps cannot be detected. There, a run of
letters can change only when it composes to Vietnamese and the scoring prefers it; the rest
(digits, `_`, code clusters, mixed case) is left alone. Example from the engine: `mas!x9` gives
`má!x9`. The tray icon and the hotkey are the way out.

### 4.4 Hook loss and reinstall

Windows removes a low-level hook that does not answer in time. Both hooks store the time of the
last event they saw (`LAST_EVENT`). On every 100 ms tick the watcher compares it with
`GetLastInputInfo`; if input is more than `DEAD_AFTER_MS` = 1000 ms newer than the last
callback, it posts a reinstall to the hook thread (uninstall, install), once per burst. Tick
wrap-around is handled (`hook::looks_dead` tests).

## 5. Engine

### 5.1 Model: whole-word recompute

The engine keeps the raw keys of the word being typed. After every key it recomputes what the
whole key sequence should look like, compares it with what is on screen, and returns the
smallest edit. There is no other incremental state that could drift from the screen.

API (`triokey_engine`):

- `Engine::key(ch)`: an ASCII letter. Anything else resets the word and passes.
- `Engine::boundary(ch)`: any printable non-letter, including space, Tab and `'\n'`. Commits the
  word. A `Replace` at a boundary ends with that character.
- `Engine::backspace()`: section 5.6.
- `Engine::reset()`: the cursor moved (click, arrow keys, shortcut, focus change, IME change,
  on/off): forget the word, the line and any pending swap.
- `Outcome::Pass` (let the key through) or `Outcome::Replace(Edit { delete, insert })`.
- `Config { tone_style, auto_english }`. With `auto_english` off the engine is plain Telex: no
  scoring, no code mode, no remembered words.

While typing, the screen shows the composed Telex text until it can no longer become Vietnamese
(not a loose syllable prefix, which ignores marks and tones still to come, and not a prefix of a
common abbreviation with `đ`); from then on it shows the raw reading of section 5.3, step 1
(`raw`, sticky for the word; `verr` shows `ver`).
The final choice between readings is made at the boundary (section 5.3).

Repeated keys that cancel a mark (`verr`, `xooo`) are tracked per key (`cancel`), which gives
two raw readings: the keys as typed ("double", `mixx`) and the keys without the cancelling second
press ("single", `mix`).

### 5.2 Code mode

A cluster is the text between two whitespace characters. With `auto_english` on, a cluster turns
into code, and from then on every word in it is committed exactly as typed, with no Backspace
alternative and no effect on the line mode, when:

- its first character is one of `- / \ . ~ $ @ #` (`-las`, `/usr/bin`),
- it contains a digit or `_` (`has123`, `my_var`),
- a letter follows `.`, `:` or `\` (`obj.dir`),
- a word has an uppercase letter after its first letter and also lowercase letters (`iPhone`),
- a `/` or `-` follows a committed word that is ASCII and not a Vietnamese syllable
  (`src/main` is code; `bật/tắt` and `ca-nô` are not).

Code mode ends at the next whitespace.

### 5.3 Scoring model

Rates are uses per million words. `Lexicon::rates(w)` returns (Vietnamese rate, English rate)
for any word, looked up by a tone-free key: toneless lowercase plus a tone digit, so `hòa` and
`hoà` share one entry and their counts add up (section 6.1 for the tables).

**Line mode.** `mode` is the log-odds that the current line is Vietnamese (possibly with English
terms) rather than English. It starts at `PRIOR` on a fresh line, so a fresh line leans
Vietnamese (`p = 1/(1+e^-1) ≈ 0.73`). After each committed word of two or more characters that is
not code:

```
lean(word) = clamp(ln((vi + EPS) / (en + EPS)), -CAP, CAP)
mode       = DECAY * mode + lean(word)
```

The mode returns to `PRIOR` at Enter (`'\n'` or `'\r'`) and on every `reset()`. A Backspace into
the previous word keeps the mode. A Backspace swap sets the mode as if the swapped word had been
committed (`this ⌫ songs ` gives `thí sóng `). The engine also remembers whether the previous
word on the line was ALL CAPS Vietnamese (no lowercase letter and a positive lean), for rule
"ALL CAPS" below.

**Readings at commit** (`Engine::scored`), in order:

1. **Raw reading `r`.** With no cancelled mark, or when the keys are in `force_en.txt`: the keys
   as typed. Otherwise compare the double and single readings: both known in a table, the double
   one (as typed; plain `of` already gives `of`, so `off` stays `off`), with the single one as the
   Backspace alternative if close; only one known, that one (`mixx` gives `mix`); neither known,
   the single one if it composes to a Vietnamese syllable, else as typed (`passwd`,
   `Terraform`).
2. **Rebuilt after Backspace.** If a Backspace left the word as raw characters (`literal`,
   section 5.6) and `r` is a known word, commit `r` as is (`boss⌫ ` gives `bos`; `hocc⌫j ` still
   gives `học` because `hocj` is no word). If the composed text `v` equals `r`, commit it.
3. **ASCII composition.** If `v` has no Vietnamese letter (a `z` removed the tone: `toansz` gives
   `toan`): `v` when it is a syllable and `r` is no known word, else `r`.
4. **Vietnamese candidate.** `rate_v` is the table rate of `v`, floored at `SYL_FLOOR` (or
   `MARKED_FLOOR` for two or more marks) if `v` is a valid syllable. If `v` is not a syllable it
   must be an abbreviation (`lexicon::is_abbr`: two or more letters, at least one consonant, `đ`
   counted, not a syllable, rate at least `ABBR_MIN`; `ddc` gives `đc`), else `r` is committed.
   Lone letters and vowel fragments (`dd`, `aa`, `aw` alone) are not abbreviations.
5. **Raw candidate.** `p = sigmoid(mode)`; `rate_r = p * vi(r) + (1 - p) * en(r)`, or
   `RAW_FLOOR` if `r` is in no table. `score_v = p * rate_v`.
6. **Decision**, first match wins:
   - typed keys (lowercased) in `force_vi.txt`: `v`; `r` is always the Backspace alternative.
   - typed keys in `force_en.txt`: `r`; `v` is always the alternative.
   - **ALL CAPS:** every key uppercase, the previous word was not ALL CAPS Vietnamese, and
     either `r` is a known word or the table rate of `v` (before
     the floors of step 4) is below `CAPS_MIN`: `r`, with `v` always the alternative (`OS`, `USA`
     stay). When the previous word on the line was ALL CAPS Vietnamese, this rule is skipped and
     the word is scored normally (`CA SIX` gives `CA SĨ`); an English acronym in between ends
     this (`CA USB SIX` stays as typed).
   - `score_v > rate_r`: `v`. `r` is the alternative if it is close (below) or if its English rate
     is at least `ARM_EN`: however Vietnamese the line, a common English word stays one Backspace
     away (`tôi đã nói … sets ⌫` gives `sets`).
   - otherwise `r`, with `v` as the alternative if close, else the alternative from step 1.

**Lone capitals.** Two capitals are an acronym wherever they stand: `Mac OS `, `cài OS mới `,
`team US đang online ` keep them. A one-letter word is typed with Shift on the letter only
(`As`, not `AS`); mixed case like that is scored normally, so `chaau As ` gives `châu Á `, while
`As ` alone on a fresh line stays `As` (English `as` is far commoner) and one Backspace gives `Á`.
Inside a shouted line the skip above applies: `VIEEJT NAM CHAAU AS ` gives `VIỆT NAM CHÂU Á `.

**Arming the Backspace swap.** The losing reading becomes the alternative only when the winner
is at most `ARM_RATIO` times likelier (`pick`). A near-certain word leaves Backspace alone:
`tooi ⌫` deletes only the space. Forced words and ALL CAPS words are always armed; `ARM_EN`
arms common English words that lost.

**Constants**

| Constant | Value | Meaning | Where |
|---|---|---|---|
| `PRIOR` | 1.0 | line mode at the start of a line | `engine.rs` |
| `DECAY` | 0.6 | weight of the previous mode per word | `engine.rs` |
| `CAP` | 3.0 | most one word can move the mode | `engine.rs` |
| `EPS` | 0.5 | smoothing of the per-word log-odds | `engine.rs` |
| `SYL_FLOOR` | 0.1 | rate of a valid syllable missing from the table (one mark or none) | `engine.rs` |
| `MARKED_FLOOR` | 0.2 | same, two or more marks | `engine.rs` |
| `RAW_FLOOR` | 0.05 | rate of raw keys that are in no table | `engine.rs` |
| `CAPS_MIN` | 50 | an ALL CAPS word composes only when the Vietnamese word is this common | `engine.rs` |
| `ARM_RATIO` | 100 | the loser is armed when the winner is at most this many times likelier | `engine.rs` |
| `ARM_EN` | 10 | raw keys this common in English are always armed against a Vietnamese win | `engine.rs` |
| `ABBR_MIN` | 3 | a composed non-syllable this common is an abbreviation | `lexicon.rs` |
| `EN_WORD_EN` | 0.5 | English rate floor for every word in `en_words.txt` | `lexicon.rs` |
| `TECH_EN`, `TECH_VI` | 5, 5 | English and Vietnamese rate floors for `en_tech.txt` | `lexicon.rs` |
| `LEARNED_MAX` | 5000 | most remembered entries | `lexicon.rs` |
| `LEARNED_LEN` | 32 | longest keys or text of one entry, in characters | `lexicon.rs` |

The scoring constants were tuned on the dev split of the typing benchmark (section 11.2).

### 5.4 Telex layer and the ALL-CAPS fix

Composition is the `vi` crate (0.8.0, MIT) with its Telex rules and `AccentStyle::Old` or `New`
(`hòa` / `hoà`), called on the whole key sequence (`telex::compose`). The `vi` crate misplaces
the tone in ALL CAPS `uyê` (`TRUYEENF`), so a sequence of two or more keys that are all uppercase
is composed in lower case and uppercased (`TRUYỀN`).

Whether a string is a Vietnamese syllable comes from `syllable.rs`, a set generated from rules
(initials × rhymes × tones, stop finals only with sắc or nặng, the c/k/q/g/gh/ng/ngh spelling
rules), not from a word list.

### 5.5 Remembering kept swaps

**When a swap is stored.** Right after a commit, one Backspace swaps the word to its alternative
(section 5.6). If the very next event is a space, Tab or Enter, the swap is kept
(`Engine::keep_swap`): `keys → shown text`, both lowercase, goes into the shared `Learned` map.
A letter, punctuation, another Backspace or any reset in between stores nothing, so `test ⌫s`
(screen `téts`) or `this ⌫,` (screen `thí,`) teach nothing; a swap fixed right before punctuation has to be fixed again next time.
Swaps of words in the force files are not stored. A clone of the engine that swaps (a probe)
stores nothing, because storing happens only at the next whitespace.

**When an entry applies** (`Engine::commit_word`). Order: force files, then remembered entries,
then scoring. An entry is ignored in code clusters, for keys rebuilt after a Backspace and with
`auto_english` off. It applies only when its text equals one of the readings in play (as typed,
single, composed), compared in lower case, and differs from the scored word; the reading is then
written in the case of the keys (a remembered `this → thí` gives `Thí` for `This` and `THÍ` for
`THIS`).

**When a remembered reading yields.** If scoring picked the composed Vietnamese word and the line
already leans more Vietnamese than a fresh line (`mode > PRIOR`), the entry does not replace the
Vietnamese word: it only becomes the Backspace alternative, and keeping that swap leaves the
entry as it was (`đi tooi` with `tooi → tooi` remembered gives `đi tôi`). When the line does not
lean more Vietnamese than a fresh line (`mode ≤ PRIOR`, which includes the start of a line, English
lines and lines leaning only mildly Vietnamese), or the scored word is not the composed Vietnamese
one, the entry wins. So a remembered English
reading at the start of a line beats a Vietnamese word there (with `sex → sex` remembered, a
line-initial `sex` stays `sex`, though it would otherwise give `sẽ`); mid-line on a Vietnamese
line it does not.

**Undoing an entry.** When an entry applied, the scored word is the Backspace alternative.
Swapping back and keeping it forgets the entry if a fresh line would pick that word anyway, and
otherwise replaces the entry with it.

**Limits.** At most `LEARNED_MAX` = 5000 entries; once full, new keys are refused silently and
existing ones can still change. Keys must be 1 to 32 ASCII letters and the text 1 to 32
alphabetic characters. Entries never age. An entry saved under the other tone style never
applies; it stays until the same keys are swapped and kept again, or the line is deleted from the
file. An entry for a word later added to a force file never applies and is never changed by
typing; only editing the file removes it.

**File format** (`learned.txt`, section 7): two `#` header lines, then one `keys<TAB>text` line
per entry, sorted by keys. On reading, a leading BOM is stripped, any whitespace separates the two
fields, lines starting with `#` (after leading whitespace) and malformed lines
are skipped, both fields are lowercased and checked as above, and entries past 5000 are dropped.

### 5.6 Backspace handling

- **Right after a commit with an armed alternative:** one Backspace deletes the word and its
  boundary character and inserts the alternative (`this ⌫` gives `thí`, with the space gone).
  The line mode follows the swap; a following whitespace keeps it (section 5.5).
- **Into the previous word** (nothing of the current word on screen): the key passes and the word
  state resets, but the line mode and previous-word flag are kept.
- **Inside a word:** one character is removed from the shown text and the raw keys are re-derived
  from what is left (`derive_keys`, tone key last; `tiế` gives `tiees`), so `tieesng⌫⌫ng ` gives
  `tiếng `. If the shown text was raw keys or code, or the derived keys do not compose back to
  the shown text, the remaining characters themselves become the keys and, unless the word is
  code, it stays raw and `literal` (step 2 of section 5.3).

## 6. Data and corpus tools

### 6.1 Engine data

Files in `crates/triokey-engine/data/`, compiled into the exe with `include_str!`:

| File | Content | Source and licence |
|---|---|---|
| `freq_vi.tsv` | Lowercase token counts (`token<TAB>count`) under a `# total: N` header; count ≥ 5 | FineWeb-2, config `vie_Latn` (Hugging Face), ODC-By 1.0; counts only, no text |
| `freq_en.tsv` | Same, count ≥ 3 | FineWeb, config `sample-10BT`, ODC-By 1.0; counts only |
| `en_words.txt` | English dictionary words, one per line; each gets an English rate of at least 0.5 | SCOWL via the VieLotus-IM `scowl-en.txt`, plus VieLotus-IM English lexicon words; SCOWL licence in `SCOWL-Copyright.txt`, VieLotus-IM MIT. No generator for this file is in the repository |
| `en_tech.txt` | Developer and software tokens; each gets both rates of at least 5 | Written for TrioKey, MIT |
| `SCOWL-Copyright.txt` | SCOWL licence, stored byte for byte | SCOWL |

The tables are parsed once per process into one `HashMap` on the first `rates` or
`is_abbr_prefix` call (`OnceLock`); the latter is also called while a word is being typed. Abbreviation prefixes are every prefix of a `freq_vi` word that contains `đ` and is
an abbreviation (section 5.3, step 4). Full attributions: `NOTICE.md`.

### 6.2 Test data

Files in `crates/triokey-engine/tests/data/`:

| Path | Content | Source and licence |
|---|---|---|
| `vielotus_test_suite.csv` | 9,084 single-word cases with expected output | VieLotus-IM test suite, copied unchanged, MIT (`VieLotus-LICENSE.txt`) |
| `vi_attested.txt` | Attested Vietnamese syllables; read by `tools/corpus/fetch.py` to pick English words for the mixed web set | VieLotus-IM syllable list plus `tý`, MIT |
| `mixed/*.txt` | Five sets of 320 lines, one per register (see `mixed/README.md`) | Written for this project, invented names, MIT |
| `final/*.txt` | Five blind sets in the same registers, written by people who never saw the engine (see `final/README.md`) | Written for this project, MIT |

`.gitattributes` forces LF everywhere except the three files copied byte for byte.

### 6.3 Corpus tools

`tools/corpus/` holds two standard-library Python scripts, run by hand; output goes to `.corpus/` at the repository root.

- `fetch.py` samples rows of FineWeb-2 (`vie_Latn`, default 16,000) and FineWeb (`sample-10BT`,
  default 10,000) through the Hugging Face datasets-server rows API, evenly spread over each
  split, NFC-normalised, resumable. Each document goes to exactly one side, fixed by
  `sha1(id) % 5`: one in five documents is **benchmark** text and is never counted, the rest are
  **counts** documents. The split is checked (`check_split`). From benchmark documents it writes
  `.corpus/bench/web_vi.txt`, `web_en.txt` and `web_vi_mixed.txt` (5 to 40 tokens per line,
  at least 70% letters; at most 5 lines per document in `web_vi` and `web_en`; mixed lines
  contain an English dictionary word of three or more letters that is no attested syllable). From counts documents it writes
  `.corpus/counts/{vi,en}_tokens.tsv`.
- `build_freq.py` turns the counts into `data/freq_vi.tsv` and `data/freq_en.tsv`: lowercase
  tokens of ASCII or Vietnamese letters only, at most 24 characters, with at least the minimum count (5 for Vietnamese, 3 for English), with
  the source, licence and document count in the header.

**Never committed:** anything under `.corpus/` (raw rows, benchmark lines, token counts),
benchmark dumps, and dataset text of any kind. Only the derived counts in `data/` are tracked.
`.gitignore` also excludes `*.key`, `*.minisig` and `minisign.pub`.

## 7. Files on the user's machine

All user files live in `%APPDATA%\triokey`, found with `SHGetKnownFolderPath(FOLDERID_RoamingAppData)`.
Each file is optional; TrioKey runs on defaults without any of them.

| File | Format | Read | Written |
|---|---|---|---|
| `config.txt` | `key = value` lines: `enabled`, `tone_new`, `auto_english`, `yield_cjk`, `hotkey` (`ctrl_shift`, `alt_z`, `ctrl_space`, `none`), `run_at_startup`, `run_as_admin`, `auto_update`, `last_update_check` (Unix seconds). Unknown keys, bad values and a BOM are tolerated | once at start | created with defaults when missing (not by an elevated copy); rewritten on every settings change (tray thread) and after every answered update check (updater thread, unelevated only) |
| `force_vi.txt`, `force_en.txt` | one word per line as typed keys (`this`, not `thí`), `#` comments, case ignored; a word in both counts as English | once at start; editing needs a restart | never |
| `learned.txt` | section 5.5 | once at start (at most 256 KB, UTF-8) | created with the header when missing (not by an elevated copy); about 2 s after the last change (watcher tick), at exit and at logoff/shutdown (`WM_ENDSESSION`) |

Rules:

- A `config.txt` that exists but cannot be read (locked, UTF-16) is never overwritten; defaults
  apply and the settings window says changes last until exit. A failed write is reported too.
- A `learned.txt` that cannot be used (not UTF-8, over 256 KB, locked, refused by the elevated
  checks) leaves the words in memory only and the file untouched.
- The on/off toggle from the hotkey or a tray click is not saved; the checkbox in the settings
  window is (`enabled`).
- A crash or kill loses up to about 2 s of remembered words; a crash in the middle of a write
  can leave the file cut short (no temp-file rename, so a linked `config.txt` stays a link).
- First start only: if `%APPDATA%\triokey` does not exist but `%APPDATA%\evj` does (an older
  folder name), the three files `config.txt`, `force_vi.txt`, `force_en.txt` are copied (regular
  files only, `run_as_admin` reset). Never done by an elevated copy.

Outside that folder TrioKey writes only: the Run value `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TrioKey`
(start with Windows, normal mode), the scheduled task `TrioKey` and `%ProgramFiles%\TrioKey\`
(admin mode, section 8), and `triokey.new.exe` / `triokey.old.exe` next to the running exe during
an update (section 9).

## 8. Elevated copy

### 8.1 Why it exists, and what "elevated" means

Windows does not let a process read or inject keys into windows of a higher integrity level, so
typing into admin windows needs an elevated TrioKey. "Elevated" (`admin::is_elevated`) means the
process holds the elevated half of a split UAC token (`TokenElevationTypeFull`): the admin-mode
copy, or any copy started with "Run as administrator". Only then do unelevated programs of the
same user exist, and they control HKCU and `%APPDATA%`. With UAC off or as the built-in
Administrator the token is not split, every program of the user is equally privileged, and
TrioKey behaves like a normal copy. If the token type cannot be read, an administrator
(`IsUserAnAdmin`) counts as elevated. Whether the admin part of section 8.2 runs in-process
depends on admin rights (`admin::is_admin`, `IsUserAnAdmin`), not on `is_elevated`, so a copy
with an unsplit admin token also needs no prompt.

### 8.2 Admin mode install

The settings checkbox "Chạy quyền admin" runs, from the normal copy:

1. `app::create_learned_file` (an elevated copy creates no files, so `learned.txt` must exist).
2. `admin::run_elevated`: if the process already has admin rights, `install()` runs in-process;
   otherwise `ShellExecuteExW` with the `runas` verb starts `triokey.exe --install-admin` (one UAC
   prompt) and waits up to 2 minutes for its exit code.
3. The helper takes no path from the environment, the config or its arguments. It copies its own
   exe to `%ProgramFiles%\TrioKey\triokey.exe` (Program Files from the known-folder API), writes
   the task XML into that admin-only folder (UTF-16 with BOM; a file in `%TEMP%` could be swapped
   before `schtasks` reads it), runs `System32\schtasks.exe` by full path (`/Create /XML /F`) and
   deletes the XML. The task: logon trigger and principal for the account that elevated
   (`NameSamCompatible`), `InteractiveToken`, `HighestAvailable`, no time limit, priority 4 (the
   default 7 is below normal), runs on battery, `IgnoreNew`.
4. Back in the unelevated parent: the Run value is removed (the task replaces it), `run_as_admin`
   is saved, and the user may start the admin copy at once (`schtasks /Run`; the normal copy then
   quits).

Turning admin mode off deletes the task and the folder. A running admin copy locks its exe, so
its files are renamed and queued for deletion at the next reboot (`MOVEFILE_DELAY_UNTIL_REBOOT`).
Turned off from the normal copy, the Run value is restored if start with Windows was on; turned
off from the admin copy, the user has to start the normal copy and tick it again.
Exit codes: `OK` 0, `FAILED` 1, `OK_AFTER_REBOOT` 2, `COPY_FAILED` 3, `TASK_FAILED` 4,
`OK_FOLDER_LEFT` 5. UAC "No" changes nothing.

### 8.3 What an elevated TrioKey must never do

- **Call `ShellExecute`.** Verbs are resolved through `HKCU\Software\Classes`, where an
  unelevated program can register its own command. Elevated, "open config folder" and "open the
  releases page" show the path or address as text, and the admin part runs in-process instead of
  through `runas`.
- **Create files** under `%APPDATA%`, or write anything there except an existing file through the
  checked open (8.4). It does not write `last_update_check` (kept in memory) and does not migrate
  the old folder.
- **Turn keys injected from a lower integrity level into text** (section 4.2).
- **Install an update by itself**, or write, rename or delete anything in a folder that is not
  the admin-only Program Files folder (section 9.3).

Every copy allows the `TaskbarCreated` message through UIPI; an elevated copy needs it, or its
tray icon would not come back after an Explorer restart.

### 8.4 Checked open and write

`app::open_checked` is used for every elevated write (`config.txt`, `learned.txt`) and for the elevated read of
`learned.txt`:

1. The folder is opened itself (`FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT`),
   shared for read and write but not delete, so it cannot be renamed or replaced while held; it
   must be a directory and no reparse point (`hold_dir`).
2. The folder's final path must equal the requested folder (`\\?\` stripped, trailing `\`
   trimmed, ASCII case ignored), which refuses a junction or link anywhere higher up the path.
3. The file is opened without following a link and without truncating; on the handle it must be
   a regular file, no reparse point, with exactly one name (no hard link), whose final path is
   `<folder>\<name>`.
4. A write truncates through the handle and writes while the folder handle is still held.

Refusals mean memory only. Tests in `app.rs` cover a missing folder or file, a folder swapped for
a junction, symbolic links where they can be made, hard links both ways, a junction higher up the
path, and the real path in other case with a trailing backslash.

### 8.5 What remains open

- The AppData location is an HKCU setting: an unelevated program can point it at another real
  folder named `triokey` containing these files (marked as a shortcut in `app.rs`). Only a
  per-machine data folder would close this.
- `config.txt` and the force files are read without the checks and follow links. Their content
  is parsed as data; `config.txt` is written back only from the known fields, and the force files
  are never written.
- If any part of the profile path goes through a junction, the elevated copy keeps settings and
  remembered words in memory only.
- The helper copies the running exe from wherever the normal copy lives. A program that can
  write that exe before the UAC prompt already controls what gets elevated, so admin mode should
  be turned on from a folder other programs cannot write.
- Elevation as another account (over the shoulder) registers the task for that account.

## 9. Updates

### 9.1 Trust model

Trust comes from the minisign (Ed25519, prehashed) signature alone, checked with the public key
compiled in from `crates/triokey/update-pubkey.txt` (`minisign-verify`). TLS and the host list
only keep the requests where they belong. A build whose key file holds no key line starts no
updater thread and never connects.

A release is `triokey.exe` plus `triokey.exe.minisig`, whose trusted comment is
`version=X.Y.Z file=triokey.exe`. An update is accepted only if the signature over the exe and the
comment verifies, `file` is `triokey.exe`, `version` is exactly three numbers, and it is greater
than the running version: no downgrade, no reinstall of the same version. Before verification the
comment only decides whether to download the exe; the version is shown, stored or acted on only
after verification, and the decision is made again from the verified comment.

### 9.2 Network

- WinHTTP (system TLS, certificate store and proxy), `User-Agent: TrioKey-updater`, no other
  header, no identifier, no version. Cookies, authentication and WinHTTP's own redirects are off
  (the request is not sent if that option fails).
- Two fixed URLs under `https://github.com/ved0el/triokey/releases/latest/download/`: the
  signature, then the exe only if the signature names a newer version. Redirects are followed by
  hand, at most 5 requests per file (the first plus 4 redirects), and every target must be `https://`, default port, no user part, host exactly
  `github.com`, `release-assets.githubusercontent.com` or `objects.githubusercontent.com`.
- Limits: 4 KB signature, 20 MB exe, counted on bytes received; WinHTTP timeouts 10 s resolve,
  10 s connect, 15 s send, 30 s receive, and 10 minutes for a whole request.
- **Cadence:** the thread wakes 60 s after start and then hourly. With `auto_update` on it checks
  when the last answered check is a day or more away (either direction, so a clock set back also
  triggers). A check counts as answered once the signature file was downloaded and parsed (a minisign
  signature with a well-formed trusted comment); a failed one is retried
  at the next wake. "Kiểm tra bản mới" checks at once whatever `auto_update` says.

### 9.3 Install versus notify

`update::mode` decides from what the process is, not from the settings:

| Process | With a newer verified release |
|---|---|
| Not elevated | Installs into its own folder without asking. If the exe is not named `triokey.exe` or the new file cannot be created there: notify, and the click opens the releases page |
| Elevated, running as `%ProgramFiles%\TrioKey\triokey.exe` | Notifies "Có bản X.Y.Z — bấm để cập nhật"; installs there only on a click, downloading and verifying again. While that version waits, later automatic checks fetch only the signature |
| Elevated from any other folder | Notifies only; writes nothing; the click shows the address as text |

The exe is always downloaded into memory and verified before anything is written.

### 9.4 Swap and repair

`triokey.new.exe` is created with no sharing, written, flushed to disk and closed. Then
`triokey.exe` is renamed to `triokey.old.exe` (Windows allows renaming a running image) and the
new file takes the name. If the second rename fails, the first is undone with retries for about a
second; if that fails too, the status line asks the user to rename `triokey.old.exe` back, and
the next install attempt does it itself (`repair`). One install per run; the new version runs
from the next start, which deletes `triokey.old.exe` and any leftover `triokey.new.exe` (only in
a build with an update key, and not in an elevated copy outside Program Files).

### 9.5 What is not protected against

- Anyone holding the signing key, or able to push a `v*` tag (section 10.3), can ship an update
  to every installed copy. A malicious dependency compiled into the exe is signed like the rest.
- The signature carries no date: whoever controls the release assets or TLS, without the key, can
  serve any genuine release newer than the running one, or an older signature that hides newer
  releases.
- A process killed exactly between the two renames leaves no `triokey.exe`; TrioKey cannot start
  to repair that (marked as a shortcut in `update.rs`). The renames go by path after the handle is
  closed, acceptable only because both installing modes write a folder of their own privilege
  level.
- If GitHub moves downloads to a new host, updates stop until a release that allows it is
  downloaded by hand. The key cannot be rotated in place.
- The exe has no Authenticode signature; this protects the update path, not the first download.

## 10. Build and release

### 10.1 Resources written by build.rs

The only supported toolchain is `stable-x86_64-pc-windows-gnu` (pinned in `mise.toml`; run
cargo as `mise x -- cargo …`). It has no resource compiler, so `crates/triokey/build.rs` writes
one x86-64 COFF object with a `.rsrc` section itself and links it into the binary:

- every frame of `assets/brand/triokey.ico`, in file order, as `RT_ICON` resources plus one
  `RT_GROUP_ICON`. The `.ico` is committed as is.
- the application manifest from `embed-manifest`: Windows 10/11, Common Controls 6, per-monitor
  DPI awareness v2, `asInvoker` (admin mode elevates on demand, never at start).

Icon and manifest must be in one object: two objects with their own `.rsrc` would not merge into
one resource tree. `build.rs` asserts the GNU x86-64 target. The tray glyphs are not icon files:
`icon.rs` draws V (red) and E (blue) at the exact size Windows asks for, in a deep shade on a
light taskbar and a light shade on a dark one, and the tray redraws them when the theme changes.

### 10.2 CI

`.github/workflows/ci.yml`, on pushes to `main` and on pull requests, on a Windows runner with the
GNU toolchain: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --
-D warnings`, `cargo test --workspace --locked`, `cargo build -p triokey --release --locked`. Third-party actions are pinned to commit
SHAs.

### 10.3 Release flow

`.github/workflows/release.yml` runs when a tag `vX.Y.Z` is pushed. There is no manual approval.

- `build` (read-only, no secret): the tagged commit must be on `origin/main`, the tag must be `v`
  plus the version of the `triokey` crate; then tests and the release build. All build scripts
  and tests of the shipped exe run here, away from the key. The publish job compiles only `xtask`
  and its dependencies, before the step that receives the key.
- `publish` (environment `release`, which holds `TRIOKEY_SIGN_KEY`): builds `xtask` itself (a
  binary from the build job never runs next to the key), signs with the key in one step's
  environment only, verifies the signature against the committed public key and the expected
  version (which also refuses a build with no key), packages a zip and `SHA256SUMS.txt`, attests
  build provenance (best effort), and publishes the release with the two bare asset names the
  updater fetches. The release must not be a draft or pre-release.

Trade-off: a release needs no second person, so whoever can push a `v*` tag can publish a signed
update to every installed copy. The "on main" check catches a tag on the wrong commit, not an
edited workflow under a tag; the protection is who has push access. Steps and recovery:
[RELEASING.md](RELEASING.md).

## 11. Testing and measurement

### 11.1 Unit and integration tests

`cargo test --workspace --locked` runs everything that gates CI, among them:

- engine: `tests/engine.rs` (typed keys → screen through `tests/sim.rs`, including the
  scoring acceptance cases and remembering), `telex_conformance.rs`, `syllable.rs`,
  `lexicon_token.rs`, unit tests in `lexicon.rs`, and the non-ignored helper tests in
  `realtext.rs` and `vielotus.rs` (tone-position equivalence, line planning, cost and armed
  counting, CSV quoting);
- app: pure parts of each module (tray tooltip, icon geometry and WCAG contrast of the tray
  colours, settings window DPI scaling, `ime::decide`, key classification, hotkey state machine,
  Enter/Tab split, hook-liveness test, latency ring, `SendInput` records, config round trip,
  password class test, task XML, Run value, the elevated checked writes and reads on real
  temporary folders with junctions and links, version comparison, trusted comment, host list,
  placeholder key, signature refusal cases, the swap);
- `xtask`: key generation, sign and verify round trip, refusals.

### 11.2 Typing benchmark

`crates/triokey-engine/tests/realtext.rs` measures and does not gate:
`cargo test -p triokey-engine --release --test realtext -- --ignored --nocapture`.

**What it simulates.** Each line is a message as its author wants it on screen. A simulated
typist types it into a fresh engine: English words letter by letter, Vietnamese words in Telex
with a deterministic per-word choice of tone-key position (end of word, after the toned letter,
after the vowel group) and of `uow` or `uwow`. Every cluster on the final screen is compared with
the wanted one; old and new tone placement count as equal.

**Columns.** `lines`, `skip` (empty or untypable lines, or lines with characters outside ASCII
and Vietnamese letters),
`clusters`, `wrong`, `wrong%`, the error classes `en_mangled` (English got marks), `en_altered`
(English changed but stayed ASCII), `vi_raw` (Vietnamese left as ASCII), `vi_wrong`, `line_shift`
(cluster count changed), `flip` (wrong clusters that one Backspace after the space would fix),
`cost/100` (keystrokes spent fixing per 100 clusters: 2 for a flip, else retyping), `armed` and
`armed/100` (correct clusters where one Backspace after the space would swap the word instead of
deleting the space), `vi_lost/100`, `objective` (`cost/100 + 0.05 × armed/100`), and `repeat`
(wrong clusters whose lowercased keys and expected text were already wrong and flip-fixable
earlier in the same set). With `REALTEXT_LEARN=1` each set shares one remembered map, the typist
fixes every flip-fixable cluster at once, and a `harm` column counts clusters that are wrong only
because of a remembered entry.

**Sets and splits.** `REALTEXT_SETS` (default `mixed,web`): `mixed` = `tests/data/mixed/`;
`web` = `.corpus/bench` (or `$TRIOKEY_CORPUS`), present only after running `tools/corpus`;
`final` = `tests/data/final/`. Lines of `mixed` and `web` are split by FNV-1a of the line: one in
five is held out (`REALTEXT_SPLIT=dev|heldout|all`, default `dev`). `final` is always run whole.
Pooled rows: `own` (all `mixed` sets) and `final`. `REALTEXT_DUMP` writes failures as TSV (a `dev`
run only, never for `final`); `REALTEXT_JSON` writes a summary.

**The rule.** Tuning looks only at dev failures. Held-out and blind (`final`) runs are read as
totals only; nobody reads their failures. The report prints its "top wrong" lists only for a
`dev` run and never for a `final` set, so the stdout of a held-out or `final` run holds the table
alone, and `REALTEXT_DUMP` is ignored on anything but a `dev` run.

**Result.** On the blind sets (10,274 words written by people who never saw the engine), going
from the previous engine to the frequency engine: wrong words 0.54% → 0.16%, keystrokes spent
fixing per 100 words 2.74 → 0.59. With remembering on, wrong words went from 16 to 12 and no word
had to be fixed twice.

### 11.3 VieLotus suite

`crates/triokey-engine/tests/vielotus.rs`, run with `cargo test -p triokey-engine --release --test vielotus -- --ignored --nocapture` types each of
the 9,084 single-word cases of the VieLotus-IM suite into a fresh engine and reports passes per
category: 8,524 of 9,084 (93.8%) today, 8,361 with the previous engine. It is a guardrail of hard
single words without context, not a measure of real typing; some of its expectations (for example
a lone `aaf` giving `ầ`) conflict with choices made for real text. `VIELOTUS_DUMP` writes all
failures.

### 11.4 Hands-on checklist

[HANDS_ON_TEST.md](HANDS_ON_TEST.md) lists what only a real desktop can show: IME yielding and
the first key after a toggle (AT1 to AT3, AT2-CK for Chinese and Korean), autocomplete fields
(AT5), the language list left unchanged (AT6), hook latency (AT7), Win32 password fields (AT8),
tray, settings window, autostart, admin mode (AT9), updates (AT10), mixed English and Vietnamese
typing with the frequency engine (AT11) and remembering kept swaps in `learned.txt` (AT12).

## 12. Known limits and non-goals

**Not built, on purpose:**

- Macros or abbreviation expansion, per-application memory of on/off, a UI for the word files.
- Rewriting earlier words: a word is final once the next word starts; only the word just
  committed can be swapped with Backspace.
- VNI, VIQR and legacy code pages: Telex only.
- Special handling for terminals (including ssh sessions) and for fields with autocomplete
  (browser address bar, Excel): they get the same Backspace-and-insert edits as everything else,
  and an inserted suggestion after the cursor can make the Backspace count wrong.
- Detecting password fields outside Win32 edit controls.
- Registering a text service or keyboard layout (section 1.2). A macOS or Linux shell.

**Known behaviour of the engine** (each fixable with one Backspace unless noted):

- The first word of an English line can come out Vietnamese because a fresh line leans
  Vietnamese: `There` gives `Thể`, `These` `Thế`, `Did` `Đi`.
- A common English word after Vietnamese words can come out Vietnamese; `ARM_EN` keeps it one
  Backspace away.
- `w` typed alone stays `w` (one Backspace gives `ư`). `dd`, `aa`, `aw` typed alone stay as
  typed and Backspace does not swap them; `đ` alone needs another way to type it.
- A remembered English reading wins at the start of a line, also over a common Vietnamese word.
  A remembered entry applies in any case, ALL CAPS included. A swap fixed right before
  punctuation is not remembered.

**Known gaps in the code:**

- The first-key wait may overshoot 30 ms by a timer tick (section 3.3).
- A crash between the two update renames leaves no `triokey.exe` (section 9.5).
- The AppData redirect for the elevated copy (section 8.5).
