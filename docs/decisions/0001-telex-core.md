# 0001: Telex core = `vi` 0.8.0 as released

Date: 2026-11-25. Status: accepted.

## Context

Spec section on the Telex core: reuse an existing optimised implementation if it really
works, in this order: released `vi` -> pinned git rev of vi-rs -> vendored and patched copy
-> write our own.

## Test

`crates/triokey-engine/tests/telex_conformance.rs`: 29 cases (the Task 1 table; `tieesng`,
`tieengs`, `hoaf`, `thuys` run in both `ToneStyle::Old` and `New`).
`triokey_engine::telex::compose` wraps `vi::transform_buffer_with_style(&vi::TELEX, ..)`
(`ToneStyle::Old` -> `AccentStyle::Old`, `New` -> `AccentStyle::New`).

Result (`mise x -- cargo test -p triokey-engine --test telex_conformance`):
29 passed, 0 failed, on the first run.

## Decision

Use the released `vi = "0.8.0"` from crates.io unchanged. No pinned rev, no vendoring,
no cases needed fixing.

Note: `vi`'s own `AccentStyle` defaults to `New`; TrioKey's `ToneStyle` defaults to `Old`
(the user's setting), and `compose` always passes the style in explicitly.

Out of scope for `vi` (handled by TrioKey layers in later tasks): giving the raw keys back
for non-Vietnamese words (`windows` -> `uindows` in `vi`), the syllable set and the
English word list.
