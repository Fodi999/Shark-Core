use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Write, BufRead, BufReader};

/// Update memory frequencies by incrementing counts for each word in the list.
/// Loads existing frequencies, updates them, and saves back to the file.
pub fn update_memory_freq(words: &[String], path: &str) {
    let mut freq: HashMap<String, usize> = load_memory_freq(path);

    for w in words {
        *freq.entry(w.clone()).or_insert(0) += 1;
    }

    save_memory_freq(&freq, path);
}

/// Load word frequencies from a CSV file (word,freq format).
/// Returns empty map if file doesn't exist or can't be read.
pub fn load_memory_freq(path: &str) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    if let Ok(file) = fs::File::open(path) {
        for line in BufReader::new(file).lines().flatten() {
            if let Some((word, count)) = line.split_once(',') {
                if let Ok(n) = count.trim().parse::<usize>() {
                    map.insert(word.trim().to_string(), n);
                }
            }
        }
    }
    map
}

/// Save word frequencies to a CSV file (word,freq format).
fn save_memory_freq(freq: &HashMap<String, usize>, path: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).write(true).truncate(true).open(path) {
        for (w, n) in freq {
            let _ = writeln!(file, "{},{}", w, n);
        }
    }
}