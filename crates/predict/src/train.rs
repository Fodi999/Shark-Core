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

/// Find an exact answer for `question` in a CSV of input,output pairs.
/// Returns `Some(output)` if a matching input row is found.
pub fn find_answer(path: &str, question: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().skip(1) {
        if let Ok(l) = line {
            let parts: Vec<&str> = l.split(',').collect();
            if parts.len() != 2 { continue; }
            let input = parts[0].trim_matches('"');
            let output = parts[1].trim_matches('"');
            if input == question {
                return Some(output.to_string());
            }
        }
    }
    None
}
