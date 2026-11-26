//! TrioKey for macOS: the logic that needs no macOS call, kept here so it is tested on every system
//! (docs/ARCHITECTURE.md, 12. macOS app). The app itself is in `src/mac/`, built only on macOS.

pub mod diag;
pub mod keymap;
pub mod plan;
pub mod source;
pub mod text;

/// Bundle identifier, fixed: the Accessibility grant and the single-instance check are tied to it.
pub const BUNDLE_ID: &str = "io.github.ved0el.triokey";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
