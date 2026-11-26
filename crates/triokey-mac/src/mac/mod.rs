//! The macOS app (docs/ARCHITECTURE.md, 12. macOS app). Everything runs on the main thread: AppKit, the event tap's run
//! loop source, every TIS call, the engine and the files; only the version check's curl runs on a thread of its own.

pub mod app;
mod settings;
mod tap;
mod tis;
mod ui;

pub fn main() {
    let arg = |name: &str| std::env::args().skip(1).any(|a| a == name);
    let code = match app::run(arg("--self-test"), arg(triokey_mac::RESTARTED)) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("TrioKey: {e}");
            1
        }
    };
    std::process::exit(code)
}
