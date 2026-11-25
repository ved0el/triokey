# Releasing TrioKey

For the repository owner. Design and reasons: [decisions/0007-auto-update.md](decisions/0007-auto-update.md).
Commands are shown without the `mise x --` prefix; add it if you use mise.

Every installed TrioKey accepts an update only if it is signed with the project's update key.
Whoever holds the secret key and can publish a release can run code on every user's machine.
Treat it accordingly.

## Once: the update key

1. Delete the placeholder (the tool never overwrites a file), then generate the pair. The
   secret path must be outside the repository; the tool refuses otherwise.

   ```sh
   rm crates/triokey/update-pubkey.txt
   cargo run -p xtask -- keygen <secret-out-path> crates/triokey/update-pubkey.txt
   ```

2. Keep the secret key file somewhere that is not this repository and not a synced project
   folder: a password manager entry, plus one offline copy (it is two lines of text). It is
   **not encrypted**. If it is lost, see "Changing the key". `.gitignore` lists `*.key` as a
   last line of defence only.
3. Give it to the release workflow as a secret of a protected **environment**, not as a
   repository secret: a repository secret can be read by any workflow run that anyone with
   push access starts, for example from a branch with an edited workflow file.

   In the repository settings, before storing the key:
   - Settings → Environments → New environment `release`.
   - "Required reviewers": yourself. Every release then waits for your approval, and so
     does any other run that asks for this environment.
   - "Deployment branches and tags": selected tags only, pattern `v*`.
   - Settings → Rules → Rulesets → new tag ruleset for `v*`: restrict creations, updates and
     deletions to yourself (bypass list).

   Then store the key. Paste nothing into a terminal history:

   ```sh
   gh secret set TRIOKEY_SIGN_KEY --env release --repo ved0el/triokey < <secret-out-path>
   ```

   In PowerShell: `Get-Content -Raw <secret-out-path> | gh secret set TRIOKEY_SIGN_KEY --env release --repo ved0el/triokey`.
   If the key was ever stored as a repository secret, delete that one
   (`gh secret delete TRIOKEY_SIGN_KEY --repo ved0el/triokey`).
4. Commit `crates/triokey/update-pubkey.txt` (the public key). `cargo test --workspace` checks
   that the file holds exactly one valid key. From this commit on, builds check for updates.
5. Uncomment the CI badge at the top of `README.md` once the first CI run has finished.

## Each release

1. Set the new version in `crates/triokey/Cargo.toml` (`X.Y.Z`, numbers only), run
   `cargo build -p triokey` so that `Cargo.lock` follows (CI builds with `--locked`), and
   update `CHANGELOG.md`.
2. Commit, push, and wait for the CI run on `main` to pass.
3. Tag and push the tag. The tag must be `v` plus exactly the crate version.

   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. The `Release` workflow then, in a first job without the key:
   - stops if the tag and the crate version differ;
   - runs the tests and builds `triokey.exe` (GNU toolchain, no cache).

   The second job waits for your approval of the `release` environment (Actions → the run →
   "Review deployments"). Check that the tag is the commit you mean to release, then approve.
   It then:
   - stops if `TRIOKEY_SIGN_KEY` is not set in the environment;
   - builds the signing tool `xtask` and nothing else;
   - signs the exe from the first job: `triokey.exe.minisig` with the trusted comment `version=X.Y.Z file=triokey.exe`;
   - verifies that signature against the committed public key, which also fails if the key
     file is still the placeholder;
   - packs `triokey-X.Y.Z-windows-x64.zip` (exe, signature, licence, notices, both READMEs)
     and writes `SHA256SUMS.txt`;
   - records a build provenance attestation (best-effort; check with
     `gh attestation verify triokey.exe --repo ved0el/triokey`);
   - creates the GitHub release with the four files.
5. Look at the release page:
   - The assets `triokey.exe` and `triokey.exe.minisig` must be there **under exactly these
     names**. Installed copies fetch `releases/latest/download/triokey.exe.minisig` and
     `.../triokey.exe`.
   - The release must be neither a draft nor a pre-release: `latest` skips both, and the
     updater does not understand pre-release ordering beyond "older than the release".
6. Run AT10 in [HANDS_ON_TEST.md](HANDS_ON_TEST.md) for the first release, and whenever
   `update.rs` changed.

To sign or check by hand (for example to test before the first tag):

```sh
cargo run -p xtask -- sign <secret-key-path> <path>/triokey.exe --version X.Y.Z
cargo run -p xtask -- verify crates/triokey/update-pubkey.txt <path>/triokey.exe --version X.Y.Z
```

## If a release goes wrong

- **The workflow failed before the release was created:** fix the cause, delete the tag
  (`git push origin :vX.Y.Z`, `git tag -d vX.Y.Z`) and tag again.
- **A bad version is out:** publish a higher version. Installed copies never go back to a
  lower version, so deleting the bad release only stops new installs of it. Copies that
  already installed it wait for the next higher one.
- **Never** re-sign a different exe under a version that was already published.

## Changing the key

An installed TrioKey trusts exactly one key, the one compiled into it. It rejects a release
signed with any other key and stays on its version; the settings window then shows
`chữ ký không hợp lệ, không cài`.

So changing the key always means a manual download for every user, whatever the reason:

1. Generate a new pair as in "Once: the update key" and set the secret `TRIOKEY_SIGN_KEY` of
   the `release` environment to the new secret key.
2. Commit the new public key and release a new version as usual. The workflow signs with the
   new key and verifies against the new public key.
3. Say at the top of the release notes, and in the README, that this version must be
   downloaded and installed by hand, and why. Installed copies will not take it by themselves.

- **Old key lost:** nothing else to do.
- **Old key stolen:** do the same at once. A thief still needs a way to publish a release in
  this repository (or to intercept TLS) to use it, so also check who can push tags and
  create releases. Say plainly in the release notes that the update requires a manual download.

## Maintenance notes

- Actions are pinned to commit SHAs; Dependabot (`.github/dependabot.yml`) proposes updates
  weekly. `windows-sys`, `minisign`, `getrandom`, `rpassword` and `rtoolbox` are excluded on
  purpose: their newer versions do not link with the GNU toolchain used here (0007).
- If GitHub starts serving release downloads from a host that is not in `update::HOSTS`,
  installed copies report `không tải được từ GitHub` and stop updating. The release that adds
  the host has to be downloaded by hand.
