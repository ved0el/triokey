//! The macOS app (docs/ARCHITECTURE.md, 12. macOS app). Everything runs on the main thread: AppKit, the event tap's run
//! loop source, every TIS call and the engine.

pub mod app;
mod tap;
mod tis;
mod ui;

pub fn main() {
    let self_test = std::env::args().skip(1).any(|a| a == "--self-test");
    let code = match app::run(self_test) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("TrioKey: {e}");
            1
        }
    };
    std::process::exit(code)
}
