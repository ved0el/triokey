/// A cluster starting with this character is code from its first key (`-las`, `/usr/bin`).
pub fn starts_code(first: char) -> bool {
    matches!(first, '-' | '/' | '\\' | '.' | '~' | '$' | '@' | '#')
}

/// An uppercase letter after the first one in a word that also has lowercase (`iPhone`).
/// `Việt`, `VIỆT`, `viet` are not mixed.
pub fn is_mixed_case(word: &str) -> bool {
    word.chars().skip(1).any(char::is_uppercase) && word.chars().any(char::is_lowercase)
}
