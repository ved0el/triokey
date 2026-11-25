//! TrioKey engine: pure Rust, no OS calls.

mod engine;
pub mod lexicon;
pub mod syllable;
pub mod telex;
pub mod token;

pub use engine::{derive_keys, Config, Edit, Engine, Outcome};

/// Tone mark placement for `oa`, `oe`, `uy`: Old = `hòa`, New = `hoà`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToneStyle {
    #[default]
    Old,
    New,
}

/// Which language a word is read as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Vi,
    En,
}
