use crate::ToneStyle;
use vi::processor::AccentStyle;

/// Classic Telex result of the whole key sequence (no English/Vietnamese logic).
pub fn compose(keys: &[char], style: ToneStyle) -> String {
    let accent = match style {
        ToneStyle::Old => AccentStyle::Old,
        ToneStyle::New => AccentStyle::New,
    };
    let mut out = String::new();
    vi::transform_buffer_with_style(&vi::TELEX, accent, keys.iter().copied(), &mut out);
    out
}
