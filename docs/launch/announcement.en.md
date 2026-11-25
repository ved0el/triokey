# Launch post (English)

## Taglines

1. English, Vietnamese and Japanese on one keyboard, with no Vietnamese toggle.
2. A Vietnamese input method that steps aside when your CJK IME is on.
3. Type `windows` and get `windows`; type `tieesng` and get `tiếng`. No mode to switch.

## Post

I type English, Vietnamese and Japanese every day on Windows. Going from Vietnamese to Japanese meant turning hiragana on and also turning Vietnamese mode off, then both again on the way back.

So I wrote TrioKey, an open-source Vietnamese input method (Telex) in Rust.

- It reads whether the CJK IME is open in the current window and steps aside by itself. With the IME closed, it types Vietnamese, even under the Japanese layout.
- There is no Vietnamese/English mode. Each word is decided on its own: a valid Vietnamese syllable becomes Vietnamese, anything else stays as typed. One Backspace flips a word that has two readings.
- It adds no keyboard layout and by default registers nothing with Windows. It is a tray app: text config, no key logging. It connects only to GitHub, to check for and download signed updates; it sends no typing data or identifiers, and with the check switched off it uses no network at all.

VKey, Vikey and VieLotus-IM already do parts of this (automatic English, stepping aside on a CJK layout), and I learned from them. TrioKey differs by reading the IME's on/off state, not the layout. Details: docs/COMPARISON.md.

It is a pre-release: no binaries (build from source), tried on Windows 11 only, Telex only, Vietnamese interface. Only an earlier build was hand-tested, with Japanese; the current tray, settings and admin mode have not run on a real machine, and Chinese and Korean have only a unit-tested rule. Detection scores 92.0% on VieLotus-IM's public suite, typed without context; one known miss is a doubled s/f/r/x/j in a word it does not know (`passwd` → `paswd`).

If you type three languages a day, tell me which words it gets wrong.

https://github.com/ved0el/triokey
