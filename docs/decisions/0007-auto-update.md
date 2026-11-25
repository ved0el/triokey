# 0007: Signed automatic updates, CI and releases

Date: 2026-11-25. Status: accepted; unit-tested where it is pure, **never run**: no request has
been sent, no running exe has been swapped, and neither workflow has executed (the repository
is not pushed yet). Hands-on list: AT10 in `docs/HANDS_ON_TEST.md`.

## What changes for the user

Until now the spec (2.3, 5.8) and the README said TrioKey never opens a network connection.
The owner asked for automatic updates, so that promise is replaced by a narrower one:

- TrioKey connects only to GitHub, only to check for and download a signed update.
- The requests send no typing data and no identifier (not even the running version).
- The check can be turned off (`auto_update = false`, `Tự động cập nhật`). With it off there
  is no network use unless the user presses `Kiểm tra bản mới`. That button checks once and,
  in a normal copy, installs what it finds, like the automatic check; whether a manual check
  should only report is the owner's call.
- A build without a real update key never connects at all. That is the state of the
  repository until the owner generates the key (`docs/RELEASING.md`).

## Trust

- A release is `triokey.exe` plus `triokey.exe.minisig`, a [minisign](https://jedisct1.github.io/minisign/)
  (Ed25519, prehashed) signature. The public key is compiled in from
  `crates/triokey/update-pubkey.txt`.
- The signature's trusted comment is `version=X.Y.Z file=triokey.exe`. minisign signs the
  comment together with the signature, so after verification both fields are authentic. An
  update is installed only if the signature verifies **and** `file` is `triokey.exe` **and**
  `version` is greater than the running version. Same version and older versions are refused,
  so an old signed release cannot be replayed as a downgrade **below the running version**.
- Limit, not closed: the signature carries no date and no expiry. Someone who controls the
  release assets (or TLS) but not the key can serve any genuine release that is newer than
  the running one, for example 0.2.0 to a 0.1.0 copy when 0.3.0 is out, or serve an old
  signature to hide newer releases. Closing this needs signed freshness (a signed release
  time that the updater remembers), which is not built. Until then the answer to a flawed
  release is a higher version, and deleting the flawed assets.
- Before the exe is downloaded the comment is only a hint (it decides whether to download).
  The decision is made again from the same signature after it has been verified, and only
  that verified version is ever shown (notification, tray menu, status line), stored or
  installed, in every mode. A signature file that does not verify produces a status line
  without a version and no notification. Tests: `refuses_whatever_the_signature_does_not_cover`
  edits the comment of a validly signed old release and pairs a signature-shaped file with
  another exe.
- The version in the comment must be exactly three numbers (`update::offered`); `xtask sign`
  writes nothing else, and no other text from the comment reaches the user.
- TLS and the host allow-list are not what the update is trusted on. They keep the requests
  where they belong and stop a network attacker from wasting a download. Someone who controls
  the GitHub repository but not the key cannot ship an update. Someone who has the key and can
  publish a release (or intercept TLS) can.
- Verification uses the `minisign-verify` crate (no dependencies, MIT). Signing uses the
  `minisign` crate in `crates/xtask` only.
- The exe has no Authenticode signature. This scheme protects the update path, not the first
  download: SmartScreen still treats a release as unsigned.

## Network

- WinHTTP through `windows-sys` (`Win32_Networking_WinHttp`): system TLS, certificate store and
  proxy (`WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY`). No HTTP crate.
- Two fixed URLs: `https://github.com/ved0el/triokey/releases/latest/download/triokey.exe.minisig`
  and `.../triokey.exe`. One check fetches each at most once (the signature, then the exe only
  if the signature names a newer version); each fetch is two or three HTTPS requests because
  GitHub redirects `latest/download` to the tagged URL and then to its asset server. A
  network monitor shows about three requests for a plain check and about six for an update. The GitHub API is not used (no JSON to parse, no rate limit, no token).
- WinHTTP's own redirect handling is switched off, with cookies and authentication
  (`WINHTTP_OPTION_DISABLE_FEATURE`; if that call fails the request is not sent). Redirects are
  followed in `update::http_get`, at most 5, and every target must pass `update::target`:
  `https://`, no user part, no port, and a host that is exactly `github.com`,
  `release-assets.githubusercontent.com` or `objects.githubusercontent.com`. The first CDN name
  is what a release download redirected to on 2026-11-25; the second is the name GitHub used
  before. If GitHub moves downloads to a new host, updates stop until a release adds it, and
  that release has to be downloaded by hand.
- Limits: 4 KB for the signature, 20 MB for the exe, counted on the bytes received; timeouts of
  10 s resolve, 10 s connect, 15 s send, 30 s per receive, and 10 minutes for a whole request.
- The only header beyond what WinHTTP adds is `User-Agent: TrioKey-updater`, the same for
  every copy and version.
- Everything runs on one dedicated thread (`update::spawn`). The hook thread and the tray
  thread never touch the network or the downloaded file. The thread wakes 60 s after start and then hourly, and
  checks when `auto_update` is on and the last answered check (`last_update_check` in
  config.txt, Unix seconds) is a day or more old: one automatic check a day, not one per
  start. A check that failed (offline, a 404 because no release exists yet, a signature file
  that cannot be read) is not recorded, so it is tried again an hour later, without limit;
  the public docs say so. The check counts as answered once the signature file has arrived
  and been read, whether or not the exe then verifies, so a bad signature costs one exe
  download a day, not one an hour.
- An elevated copy does not write `last_update_check` to config.txt: that would make it
  rewrite a file under the user-writable `%APPDATA%` on a timer (0006). It keeps the time in
  memory, so it may check once more after each start, then daily.

## Installing

`update::mode` decides from what the process is, not from the config:

| Process | What happens with a newer release |
| --- | --- |
| Not elevated | Downloaded into memory, verified, written to the exe's own folder and swapped without asking. If the new file cannot be created there (read-only folder, the Program Files copy started unelevated), or the running exe is not named `triokey.exe` (`update::own_dir`: the swap works on that name, and a copy called `triokey (1).exe` must not replace its neighbour), a notification and a tray menu item say a new version exists; clicking opens the releases page. |
| Elevated, the exe is `%ProgramFiles%\TrioKey\triokey.exe` (admin mode) | Never by itself. The exe is downloaded into memory and verified, then: notification and tray menu item `Có bản X.Y.Z — bấm để cập nhật`. On a click it is downloaded and verified again, written **into that admin-only folder** and swapped there. While that version waits for the click, the daily check fetches only the signature file. |
| Elevated from any other folder ("Run as administrator") | Nothing is written, renamed or deleted in that folder, because unelevated programs may be able to write it. The exe is downloaded into memory and verified, then: notification only; the click shows the address as text, since an elevated TrioKey does not call `ShellExecute` (0006). |

"Elevated" means the elevated half of a split UAC token (`admin::is_elevated`, 0006). With UAC
off or as the built-in Administrator every copy is "not elevated" here.

- The unelevated copy never touches the admin copy, and the other way round: each copy only
  ever works in the folder of its own exe. With admin mode on, the two copies update
  independently.
- Accepted: the two copies can diverge, and turning admin mode on again from an older normal
  copy copies that older exe over a newer admin copy (`admin::install` copies without
  comparing; the exe has no version resource to compare). The admin copy is then offered the
  update again, on a click. The normal copy does not run while admin mode is on, so it only
  catches up within a day of running again.
- The swap: the bytes are verified in memory. `triokey.new.exe` is then created with no
  sharing, written, flushed to disk (`sync_all`) and closed. Then
  `triokey.exe` is renamed to `triokey.old.exe` (Windows allows renaming a running image) and
  the new file to `triokey.exe`. If the second rename fails the first is undone, with retries
  for a second; if even that fails the status line tells the user to rename
  `triokey.old.exe` back by hand (`Error::Stranded`), and the next install attempt starts by
  doing that itself (`update::repair`). The new version runs from the next start, which also
  deletes `triokey.old.exe`.
- Known gap (marked `shortcut:` in the code): a process killed, or power lost, exactly
  between the two renames leaves `triokey.old.exe` and `triokey.new.exe` but no
  `triokey.exe`. TrioKey then does not start and cannot repair itself; the user has to rename
  `triokey.old.exe`. Replacing the name in one step (`ReplaceFileW`, or a hard link plus a
  replacing rename) would close it, but whether either works on a running image is untested,
  and hard links do not exist on FAT/exFAT. To be tried by hand before changing the code
  (listed under "Chưa ai kiểm" in AT10).
- Known gap (marked `shortcut:` in the code): the handle is closed before the two renames,
  which go by path. In both installing modes the folder belongs to the process's own privilege
  level, so replacing the file in that window gains an attacker nothing they did not already
  have. Rename by handle (`FileRenameInfo`) if a mode is ever added where that is not true.
- One install per run: after it, the running image is `triokey.old.exe` and cannot be
  replaced, so later checks only repeat that a restart is pending.
- `std::env::current_exe()` keeps returning `...\triokey.exe` after the swap. The Run value,
  the admin-mode install (which copies that path) and `is_admin_copy` therefore keep working,
  and admin mode turned on after an update copies the new version.
- Writing `last_update_check` goes through `app::set_last_update_check`, which does not rebuild
  the engine and does not raise the "config.txt could not be saved" warning. An elevated copy
  skips the file write altogether (see Network). `app::SAVE` serialises
  the two threads that now write config.txt.

## The placeholder key

`update-pubkey.txt` in the repository holds only `#` comment lines. `update::key_from` then
returns `None`, `update::spawn` starts no thread, and both controls in the settings window are
greyed out with a line that says why. Tests: `placeholder_key_means_no_request_at_all` (a fetch
function that panics is never called) and `committed_key_file_is_placeholder_or_valid` (a
damaged key file cannot silently disable updates). `xtask verify` refuses the placeholder, so
the release workflow cannot publish a build that would never update.

## Deviations from the brief

- **The download is verified in memory, not on disk.** The brief asked to write the file,
  re-open it with exclusive sharing and verify that. Verifying first means nothing unverified
  is ever written, an elevated copy that must not write can still verify before it announces
  a version, and a read-only folder is found out only after the release is known to be
  genuine. The verified bytes go into a file created with exclusive sharing. Cost: up to
  20 MB of memory for the length of one check.
- **Elevated, but not the admin copy: notify only.** The brief covered the admin copy and the
  normal copy. A third case exists, and installing there would mean an elevated process
  creating and renaming files in a folder unelevated programs may control.
- **The signing key reaches CI through one step's environment, not a temporary file.** A
  file would have to be created, protected and deleted; the environment variable exists only
  for the one step that runs the already built `xtask`, and nothing is written to disk.
- **`xtask sign env <file> --version X.Y.Z`**: the literal word `env` in place of a key path
  reads `TRIOKEY_SIGN_KEY`.
- **CI runs on pushes to `main` and on pull requests**, not on every push, so a pull request
  from a branch of this repository is not built twice.
- **Older `minisign` (0.7) with `rpassword` and `rtoolbox` pinned** in `crates/xtask`. Their
  newer versions use windows-sys 0.61 or getrandom 0.3+, which link through raw-dylib and need
  `dlltool.exe`; the GNU toolchain used here fails with "dlltool.exe: program not found".
  Dependabot is told to leave these (and `windows-sys`) alone. None of them is in the exe.
- **Keys are generated with an empty password** (`generate_encrypted_keypair(Some(""))`), which
  leaves the key unencrypted but writes the checksum; minisign 0.7's
  `generate_unencrypted_keypair` writes a key its own loader then rejects.

## CI and releases

- `ci.yml`: Windows runner, `stable-x86_64-pc-windows-gnu` installed with the preinstalled
  rustup (the same toolchain as `mise.toml`), then `cargo fmt --check`, clippy with
  `-D warnings`, tests, release build, all with `--locked`. `permissions: contents: read`.
  Whether the runner links with the GNU toolchain out of the box is expected but unverified.
- `release.yml`: on a tag `vX.Y.Z`, two jobs. `build` (`contents: read`, no secret) checks
  the tag against the crate version, runs the tests, builds `triokey.exe` and hands it over as
  an artifact: the build scripts and tests of every dependency run here, away from the key.
  `publish` (environment `release`) builds only `xtask`, signs, verifies against the committed
  public key and the expected version, makes the zip and `SHA256SUMS.txt`, and creates the
  release with `gh`. Only `publish` has `contents: write`, plus `id-token` and `attestations`
  for the build provenance attestation, which is best-effort (`continue-on-error`). No cache
  in this workflow.
- `xtask` is built in `publish`, not taken from `build`: a tool from the job that runs
  third-party code must not be executed next to the key. Its own dependencies (`minisign` and
  what that pulls in) handle the key in any case.
- What the split does not do: a dependency that is compiled into `triokey.exe` and is
  malicious still gets signed. The split keeps the key itself and the release permission away
  from build scripts, test binaries and dev-dependencies.
- The signing key is a secret of the GitHub **environment** `release`, not of the repository.
  A repository secret is handed to any workflow run started by anyone with push access,
  including a run of an edited workflow on a branch or tag. The environment is meant to have
  a required reviewer and to be limited to `v*` tags, with a tag ruleset on `v*`
  (`RELEASING.md`). Those settings live in the repository's settings, not in this tree:
  until the owner has made them, the workflow file alone protects nothing.
- Third-party actions are pinned to commit SHAs read from the tags on 2026-11-25:
  `actions/checkout` v7.0.1, `actions/cache` v6.1.0, `actions/attest-build-provenance` v4.2.2,
  `actions/upload-artifact` v7.0.2, `actions/download-artifact` v8.0.2.
  Everything else is `run:` steps with the preinstalled `rustup`, `cargo`, `jq`, `7z` and `gh`.
- `rustfmt.toml` (`max_width = 160`, `use_small_heuristics = "Max"`) matches how the code was
  already written; the whole workspace was formatted once so that the check passes.
- `.gitattributes` forces LF: the word lists are compiled in and compared in tests, and a
  Windows runner would otherwise check them out with CRLF.

## Not done

- No delta updates, no rollback command, no release channels, no update of the admin copy
  from the normal copy.
- No way to rotate the key in place: a release signed with a new key has to be downloaded by
  hand (`RELEASING.md`). A bridge release (signed with the old key, carrying the new one) would
  need `release.yml` to verify against the previous key, which it does not do.
- The update notification uses the classic balloon API; how Windows 11 shows it, and whether
  the click arrives, is untested.
