# Security policy

## Supported versions

Only the latest release on GitHub receives fixes. Please check that a problem still exists in
the latest release before reporting it.

## Reporting a vulnerability

Please do not open a public issue. Report it privately through GitHub's private vulnerability
reporting on this repository: the **Security** tab, then **Report a vulnerability**
(https://github.com/ved0el/triokey/security/advisories/new).

Include what you found, the TrioKey version, the Windows version, and the steps to reproduce.
A proof of concept helps but is not required.

## Scope

In scope:

- **The elevated copy** (admin mode): anything that lets an unelevated program make an elevated
  TrioKey run code, write or read files it should not, or turn injected keys into text.
- **The update path**: anything that gets an installed TrioKey to accept or run a file that was
  not signed with the project's update key, to downgrade, or to send data beyond the two fixed
  requests to GitHub.
- **The keyboard hook**: typed text leaking to disk (beyond the remembered words in
  `learned.txt`, which are stored by design), to the network or to another process, or input that
  makes TrioKey type something other than what was typed.

Out of scope: problems that need the update signing key or push access to this repository,
attacks that already require administrator rights, and words that convert wrongly (please open a
normal issue for those). Known limits are listed in `docs/ARCHITECTURE.md` (sections 8.5 and
9.5).

## What to expect

TrioKey is a small project with no guaranteed response time. Reports are acknowledged as soon as
possible. Confirmed problems are fixed in a new release, and you are credited in the release notes if you
want to be.
