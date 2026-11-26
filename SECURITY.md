# Security policy

## Supported versions

Only the latest release on GitHub receives fixes. Please check that a problem still exists in
the latest release before reporting it.

## Reporting a vulnerability

Please do not open a public issue. Report it privately through GitHub's private vulnerability
reporting on this repository: the **Security** tab, then **Report a vulnerability**
(https://github.com/ved0el/triokey/security/advisories/new).

Include what you found, the TrioKey version, the Windows or macOS version, and the steps to
reproduce. For the macOS build, the text of its `Chẩn đoán` menu item helps; it holds no typed text.
A proof of concept helps but is not required.

## The macOS build

The macOS build is experimental: it has never run on a real Mac and is not a release (it is a
workflow artifact of the `macos` branch). What to know before running it:

- **Not signed with a Developer ID and not notarized.** The app carries only an ad-hoc signature,
  which proves nothing about who built it. Apple has not scanned it, macOS blocks the first open,
  and you allow it with Open Anyway. Download it only from this repository's own workflow runs.
  Each build has a different signature, so macOS treats every new build as a new app and the
  Accessibility permission must be removed and granted again (quit TrioKey first).
- **It needs the Accessibility permission.** That lets it see every key you press while it runs
  and post keys of its own, as every input method of this kind must. It keeps no log, writes no
  files and has no network code; the only thing it outputs besides keys is the diagnostics text it
  copies to the clipboard when you ask. Removing the permission while TrioKey runs is reported to
  freeze the keyboard and mouse until a restart: quit TrioKey first.
- **No updater.** It never checks for or installs updates; you replace the app by hand.

## Scope

In scope:

- **The elevated copy** (admin mode): anything that lets an unelevated program make an elevated
  TrioKey run code, write or read files it should not, or turn injected keys into text.
- **The update path**: anything that gets an installed TrioKey to accept or run a file that was
  not signed with the project's update key, to downgrade, or to send data beyond the two fixed
  requests to GitHub.
- **The keyboard hook** (or, on macOS, the event tap): typed text leaking to disk (beyond the remembered words in
  `learned.txt`, which are stored by design), to the network or to another process, or input that
  makes TrioKey type something other than what was typed.

Out of scope: problems that need the update signing key or push access to this repository,
attacks that already require administrator rights, and words that convert wrongly (please open a
normal issue for those). Known limits are listed in `docs/ARCHITECTURE.md` (sections 8.5 and
9.5, and for macOS 12.12).

## What to expect

TrioKey is a small project with no guaranteed response time. Reports are acknowledged as soon as
possible. Confirmed problems are fixed in a new release, and you are credited in the release notes if you
want to be.
