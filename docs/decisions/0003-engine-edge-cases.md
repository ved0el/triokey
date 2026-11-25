# 0003: Typing engine edge cases left as they are

Date: 2026-11-25. Status: accepted (open for the user to revisit).

## `docs/las` does not become code

Spec 5.4 rule 1 gives `docs/las` as an example of a cluster turning into code, but the same
section defines "clearly English" as "not a Vietnamese syllable". `docs` composes to the
syllable `dóc` and is not in `ambiguous_defaults.txt`, so the engine commits `dóc`, the word
is not clearly English, and `docs/las ` gives `dóc/lá `. The engine follows the definition.

Rejected: treating any word on the English list as clearly English at `/` and `-`. It would
fix `docs/las`, but common Vietnamese pairs whose first half is also an English word would
then lose their marks on the right side (`to/nhor` -> `to/nhor` instead of `to/nhỏ`). The
other suggestion, counting a rule-4 word committed as English, does not help here: `docs`
is committed as Vietnamese. Workaround today: type the path with a code start (`./docs/las`).

## Undo after commit even when nothing changed on screen

Undo is offered whenever exactly one of the single-press and double-press forms is on the
English list, also when the commit left the screen as typed: `err ⌫` -> `er`,
`usser ⌫` -> `usser`. This matches the plan's `mixx ⌫` -> `mixx` case and keeps the rule
simple (the backspace always swaps readings). Cost: one extra keystroke under AT4 for
someone deleting the space after `err`, `terrible`, `sorry`. Revisit if AT4 counts show it.
