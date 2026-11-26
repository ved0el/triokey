use crate::ToneStyle;
use vi::processor::AccentStyle;

/// Classic Telex result of the whole key sequence (no English/Vietnamese logic).
pub fn compose(keys: &[char], style: ToneStyle) -> String {
    // The vi crate misplaces the tone in ALL CAPS `uyê` (`TRUYEENF` -> `TRUỲÊN`): compose lower case.
    if keys.len() > 1 && keys.iter().all(char::is_ascii_uppercase) {
        let lower: Vec<char> = keys.iter().map(char::to_ascii_lowercase).collect();
        return compose(&lower, style).to_uppercase();
    }
    let accent = match style {
        ToneStyle::Old => AccentStyle::Old,
        ToneStyle::New => AccentStyle::New,
    };
    let mut out = String::new();
    vi::transform_buffer_with_style(&vi::TELEX, accent, keys.iter().copied(), &mut out);
    out
}
