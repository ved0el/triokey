//! Screen simulator shared by integration tests (`mod sim;`).
#![allow(dead_code)]

use triokey_engine::{Edit, Engine, Outcome};

/// Types `s` key by key and returns the screen. Letters go to `key`, `⌫` to `backspace`,
/// anything else to `boundary`. `Pass` lets the key through (a backspace deletes one char).
pub fn type_str(e: &mut Engine, s: &str) -> String {
    let mut screen = String::new();
    for ch in s.chars() {
        let out = match ch {
            '⌫' => e.backspace(),
            c if c.is_ascii_alphabetic() => e.key(c),
            c => e.boundary(c),
        };
        match out {
            Outcome::Pass if ch == '⌫' => {
                screen.pop();
            }
            Outcome::Pass => screen.push(ch),
            Outcome::Replace(Edit { delete, insert }) => {
                for _ in 0..delete {
                    screen.pop();
                }
                screen.push_str(&insert);
            }
        }
    }
    screen
}
