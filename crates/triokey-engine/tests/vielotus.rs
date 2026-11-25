//! Regression report against the VieLotus-IM test suite (MIT, see NOTICE.md).
//! Measures, does not gate: run with `-- --ignored --nocapture`.
mod sim;

use std::collections::BTreeMap;

use sim::type_str;
use triokey_engine::lexicon::Lexicon;
use triokey_engine::{Config, Engine};

const SUITE: &str = include_str!("data/vielotus_test_suite.csv");

/// Splits one CSV line (RFC 4180 quoting, no embedded newlines in this file).
fn fields(line: &str) -> Vec<String> {
    let (mut out, mut cur, mut quoted) = (Vec::new(), String::new(), false);
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                chars.next();
                cur.push('"');
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    out.push(cur);
    out
}

#[test]
fn csv_fields_handle_quotes() {
    assert_eq!(fields(r#"a,"b,""c""",d"#), ["a", "b,\"c\"", "d"]);
}

#[test]
#[ignore]
fn vielotus_report() {
    let lexicon = Lexicon::builtin();
    let mut by_cat: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut fails = Vec::new();
    let mut samples: BTreeMap<String, Vec<String>> = BTreeMap::new();
    // Review aid: VIELOTUS_DUMP=<file> writes every failure, one per line.
    let mut dump = String::new();
    for line in SUITE.lines().skip(1) {
        let f = fields(line);
        let (input, expected, category) = (&f[1], &f[4], &f[6]);
        if input.chars().any(char::is_whitespace) {
            continue;
        }
        let mut e = Engine::new(Config::default(), lexicon.clone());
        let got = type_str(&mut e, &format!("{input} "));
        // VieLotus writes "either is fine" as "a / b".
        let ok = expected.split(" / ").any(|alt| got == format!("{alt} "));
        let entry = by_cat.entry(category.clone()).or_default();
        entry.1 += 1;
        if ok {
            entry.0 += 1;
        } else {
            let msg = format!("{category}: {input:?} -> {got:?}, expected {expected:?} ");
            dump.push_str(&msg);
            dump.push('\n');
            let s = samples.entry(category.clone()).or_default();
            if s.len() < 3 {
                s.push(msg.clone());
            }
            if fails.len() < 30 {
                fails.push(msg);
            }
        }
    }
    if let Ok(path) = std::env::var("VIELOTUS_DUMP") {
        std::fs::write(path, &dump).unwrap();
    }
    let (pass, total) = by_cat.values().fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    println!("TOTAL {pass}/{total} ({:.1}%)", 100.0 * pass as f64 / total as f64);
    for (cat, (p, t)) in &by_cat {
        println!("{cat:45} {p:5}/{t:<5} {:5.1}%", 100.0 * *p as f64 / *t as f64);
    }
    println!("--- up to 3 failures per category ---");
    for f in samples.values().flatten() {
        println!("{f}");
    }
    println!("--- first {} failures ---", fails.len());
    for f in &fails {
        println!("{f}");
    }
}
