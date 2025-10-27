#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{BufRead, BufReader};

/// Very small "training" loader that reads a CSV of input→output pairs and prints them.
/// This is intentionally tiny and side-effecting for demo purposes.
pub fn train_from_csv(path: &str) {
    let file = File::open(path).expect("no knowledge.csv found");
    let reader = BufReader::new(file);

    println!("[train] loading dataset from {path}");
    for (i, line) in reader.lines().enumerate().skip(1) {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 2 { continue; }
        let input = parts[0].trim_matches('"');
        let output = parts[1].trim_matches('"');
        println!("[{i}] Q: {input} → A: {output}");
    }
    println!("[train] dataset ready.");
}
