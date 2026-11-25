//! Release tooling for TrioKey's signed updates (docs/RELEASING.md). Not shipped.
//!
//! The signature format is minisign's; the trusted comment `version=X.Y.Z file=triokey.exe` is
//! what the updater in crates/triokey/src/update.rs reads after verifying it.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use minisign::{KeyPair, PublicKey, SecretKeyBox, SignatureBox};

const USAGE: &str = "usage:
  cargo run -p xtask -- keygen <secret-out-path> <public-out-path>
  cargo run -p xtask -- sign <secret-key-path | env> <file> --version X.Y.Z
  cargo run -p xtask -- verify <public-key-path> <file> [--version X.Y.Z]

keygen  writes a new unencrypted key pair; never overwrites, never puts the secret in this repository
sign    writes <file>.minisig; `env` reads the secret key text from the variable TRIOKEY_SIGN_KEY
verify  checks <file> against <file>.minisig and, with --version, the signed version";
const KEY_ENV: &str = "TRIOKEY_SIGN_KEY";

type Res<T> = Result<T, String>;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let done = match args[..] {
        ["keygen", secret, public] => keygen(Path::new(secret), Path::new(public), &repo()),
        ["sign", key, file, "--version", version] => secret_text(key).and_then(|k| sign(&k, Path::new(file), version)),
        ["verify", public, file] => verify(Path::new(public), Path::new(file), None),
        ["verify", public, file, "--version", version] => verify(Path::new(public), Path::new(file), Some(version)),
        _ => Err(USAGE.into()),
    };
    match done {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(text) => {
            eprintln!("{text}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root: two levels above this crate.
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn err<E: std::fmt::Display>(what: &str) -> impl Fn(E) -> String + '_ {
    move |e| format!("{what}: {e}")
}

/// `file` with `.minisig` appended.
fn sig_path(file: &Path) -> PathBuf {
    let mut p = file.as_os_str().to_owned();
    p.push(".minisig");
    p.into()
}

fn trusted_comment(file: &Path, version: &str) -> Res<String> {
    let numbers: Vec<&str> = version.split('.').collect();
    if numbers.len() != 3 || numbers.iter().any(|n| n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit())) {
        return Err(format!("version must be X.Y.Z (numbers only), got {version:?}"));
    }
    let name = file.file_name().and_then(|n| n.to_str()).ok_or("the file has no usable name")?;
    Ok(format!("version={version} file={name}"))
}

fn keygen(secret: &Path, public: &Path, repo: &Path) -> Res<String> {
    let folder = |p: &Path| {
        let parent = p.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
        fs::canonicalize(parent).map_err(err(&format!("the folder for {} must exist", p.display())))
    };
    let repo = fs::canonicalize(repo).map_err(err("workspace root"))?;
    if folder(secret)?.starts_with(&repo) {
        return Err("refusing to write the secret key inside the repository: choose a path outside it".into());
    }
    folder(public)?;
    for p in [secret, public] {
        if p.exists() {
            return Err(format!("{} already exists: not overwritten", p.display()));
        }
    }
    // an empty password: the checksum is written and the key stays unencrypted
    let kp = KeyPair::generate_encrypted_keypair(Some(String::new())).map_err(err("key generation"))?;
    let secret_text = kp.sk.to_box(Some("TrioKey update signing key (SECRET, unencrypted)")).map_err(err("secret key"))?.into_string();
    let public_text = kp.pk.to_box().map_err(err("public key"))?.into_string();
    // create_new: a file that appeared since the check above is not overwritten either
    let write = |p: &Path, text: &str| {
        OpenOptions::new().write(true).create_new(true).open(p).and_then(|mut f| f.write_all(text.as_bytes())).map_err(err(&p.display().to_string()))
    };
    write(secret, &secret_text)?;
    write(public, &public_text)?;
    Ok(format!("secret key: {}\npublic key: {}\nNext steps: docs/RELEASING.md", secret.display(), public.display()))
}

fn secret_text(key: &str) -> Res<String> {
    if key == "env" {
        return std::env::var(KEY_ENV).ok().filter(|k| !k.trim().is_empty()).ok_or(format!("{KEY_ENV} is not set or empty"));
    }
    fs::read_to_string(key).map_err(err(key))
}

fn sign(secret_text: &str, file: &Path, version: &str) -> Res<String> {
    let comment = trusted_comment(file, version)?;
    // an empty password: the key was written unencrypted
    let sk = SecretKeyBox::from_string(secret_text.trim()).and_then(|b| b.into_secret_key(Some(String::new()))).map_err(|_| "the secret key cannot be read")?;
    let data = File::open(file).map_err(err(&file.display().to_string()))?;
    let sig = minisign::sign(None, &sk, data, Some(&comment), Some("signature from the TrioKey update key")).map_err(err("signing"))?;
    let out = sig_path(file);
    fs::write(&out, sig.into_string()).map_err(err(&out.display().to_string()))?;
    Ok(format!("{}: {comment}", out.display()))
}

/// Same rule as `key_from` in the updater: `#` and `untrusted comment:` lines are skipped and
/// exactly one key line must remain.
fn public_key(text: &str) -> Res<PublicKey> {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("untrusted comment:")).collect();
    match lines[..] {
        [] => Err("the public key file is still the placeholder: this build would never update".into()),
        [line] => PublicKey::from_base64(line).map_err(err("public key")),
        _ => Err("the public key file must hold exactly one key line".into()),
    }
}

fn verify(public: &Path, file: &Path, version: Option<&str>) -> Res<String> {
    let pk = public_key(&fs::read_to_string(public).map_err(err(&public.display().to_string()))?)?;
    let sig = SignatureBox::from_file(sig_path(file)).map_err(err("signature file"))?;
    let data = File::open(file).map_err(err(&file.display().to_string()))?;
    // quiet, no copy of the data to stdout, no legacy (non-prehashed) signatures
    minisign::verify(&pk, &sig, data, true, false, false).map_err(err("NOT VERIFIED"))?;
    let comment = sig.trusted_comment().map_err(err("trusted comment"))?;
    if let Some(v) = version {
        let want = trusted_comment(file, v)?;
        if comment != want {
            return Err(format!("NOT VERIFIED: signed as {comment:?}, expected {want:?}"));
        }
    }
    Ok(format!("verified: {comment}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("triokey-xtask-{name}-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn keygen_sign_verify() {
        let dir = temp("roundtrip");
        let (secret, public, exe) = (dir.join("triokey-update.key"), dir.join("update-pubkey.txt"), dir.join("triokey.exe"));
        keygen(&secret, &public, &repo()).unwrap();
        fs::write(&exe, b"MZ not really").unwrap();
        let key = fs::read_to_string(&secret).unwrap();
        // GitHub may hand the secret back with other line endings and a trailing newline
        sign(&format!("{}\r\n", key.replace('\n', "\r\n")), &exe, "1.2.3").unwrap();
        assert_eq!(verify(&public, &exe, None).unwrap(), "verified: version=1.2.3 file=triokey.exe");
        verify(&public, &exe, Some("1.2.3")).unwrap();
        assert!(verify(&public, &exe, Some("1.2.4")).is_err());
        // a second key does not verify the first one's signature
        let (secret2, public2) = (dir.join("b.key"), dir.join("b.pub"));
        keygen(&secret2, &public2, &repo()).unwrap();
        assert!(verify(&public2, &exe, None).is_err());
        // one changed byte
        fs::write(&exe, b"MZ not really!").unwrap();
        assert!(verify(&public, &exe, None).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn keygen_never_overwrites_and_keeps_the_secret_out_of_the_repo() {
        let dir = temp("keygen");
        let (secret, public) = (dir.join("a.key"), dir.join("a.pub"));
        keygen(&secret, &public, &repo()).unwrap();
        let before = fs::read(&secret).unwrap();
        assert!(keygen(&secret, &dir.join("other.pub"), &repo()).is_err());
        assert!(keygen(&dir.join("other.key"), &public, &repo()).is_err());
        assert_eq!(fs::read(&secret).unwrap(), before);
        assert!(!dir.join("other.key").exists() && !dir.join("other.pub").exists());
        // `dir` stands in for the repository: nothing is written below it
        let sub = dir.join("sub");
        fs::create_dir(&sub).unwrap();
        assert!(keygen(&sub.join("in-repo.key"), &dir.join("c.pub"), &dir).is_err());
        assert!(keygen(&dir.join("sub/../in-repo.key"), &dir.join("c.pub"), &dir).is_err());
        assert!(!sub.join("in-repo.key").exists() && !dir.join("in-repo.key").exists() && !dir.join("c.pub").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn placeholder_key_and_bad_input_are_refused() {
        assert!(public_key("# PLACEHOLDER\n\n# nothing here\n").is_err());
        assert!(public_key("").is_err());
        assert!(public_key("not base64 at all").is_err());
        let exe = Path::new("triokey.exe");
        assert_eq!(trusted_comment(exe, "0.1.0").unwrap(), "version=0.1.0 file=triokey.exe");
        for bad in ["", "1.2", "v1.2.3", "1.2.3-rc1", "1.2.3 file=x", "1..3"] {
            assert!(trusted_comment(exe, bad).is_err(), "{bad}");
        }
        assert!(sign("not a key", exe, "0.1.0").is_err());
    }
}
