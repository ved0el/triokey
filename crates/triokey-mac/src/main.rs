//! TrioKey for macOS (docs/ARCHITECTURE.md, 12. macOS app). Everything below runs only on macOS; elsewhere the binary is
//! an empty stub so the workspace still builds and tests on Windows.

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "macos")]
fn main() {
    mac::main()
}

#[cfg(not(target_os = "macos"))]
fn main() {}
