use std::collections::HashMap;

/// Interpret raw decoded text contextually by counting word frequencies
/// and returning the top most frequent words/phrases.
///
/// This function builds a frequency map of mapped values from the grammar map,
/// sorts them by frequency descending, and returns the top 10 as a comma-separated string.
/// Useful for concise, context-relevant interpretation instead of concatenating all matches.
pub fn interpret_contextual(raw: &str, map: &HashMap<char, String>) -> String {
    let mut freq: HashMap<&str, usize> = HashMap::new();

    for ch in raw.chars() {
        let lower_ch = ch.to_ascii_lowercase();
        if let Some(val) = map.get(&lower_ch) {
            *freq.entry(val.as_str()).or_insert(0) += 1;
        }
    }

    // Сортировка по частоте (наиболее вероятные слова/понятия)
    let mut pairs: Vec<(&str, usize)> = freq.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    // Берём только топ-10 (чтобы не захламлять вывод)
    pairs.iter()
        .take(10)
        .map(|(word, _)| word.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Load word frequencies from a CSV file (word,freq).
/// If file doesn't exist, returns empty map.
pub fn load_memory_freq(path: &str) -> HashMap<String, usize> {
    let mut freq = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines().skip(1) {
            if let Some((word, count_str)) = line.split_once(',') {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    freq.insert(word.trim().to_string(), count);
                }
            }
        }
    }
    freq
}

/// Save word frequencies to a CSV file.
pub fn save_memory_freq(path: &str, freq: &HashMap<String, usize>) {
    use std::fs::File;
    use std::io::Write;
    if let Ok(mut file) = File::create(path) {
        writeln!(file, "word,freq").ok();
        for (word, count) in freq {
            writeln!(file, "{},{}", word, count).ok();
        }
    }
}

/// Update memory frequencies with current interpretation words.
pub fn update_memory_freq(memory_freq: &mut HashMap<String, usize>, interpretation: &str) {
    for word in interpretation.split(", ") {
        let word = word.trim();
        if !word.is_empty() {
            *memory_freq.entry(word.to_string()).or_insert(0) += 1;
        }
    }
}

/// Interpret with memory weighting: combine current freq with historical memory.
/// context_temp (0.0-1.0): 0.0 = pure memory priority, 1.0 = pure current priority.
/// Now uses loaded memory_freq for stronger weighting.
pub fn interpret_contextual_with_memory(
    raw: &str,
    map: &HashMap<char, String>,
    memory_freq: &HashMap<String, usize>,
    context_temp: f64,
) -> String {
    let mut freq: HashMap<String, f64> = HashMap::new();

    for ch in raw.chars() {
        let lower_ch = ch.to_ascii_lowercase();
        if let Some(val) = map.get(&lower_ch) {
            let current = freq.entry(val.clone()).or_insert(0.0);
            *current += 1.0;
        }
    }

    // Weight with memory, using actual freq counts
    for (word, mem_count) in memory_freq {
        if let Some(current) = freq.get_mut(word) {
            *current += (*mem_count as f64) * (1.0 - context_temp);
        } else if context_temp < 0.5 {
            // Add memory words if temp is low
            freq.insert(word.clone(), (*mem_count as f64) * (1.0 - context_temp));
        }
    }

    // Sort by weighted freq descending
    let mut pairs: Vec<(String, f64)> = freq.into_iter().collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Top 10
    pairs.iter()
        .take(10)
        .map(|(word, _)| word.clone())
        .collect::<Vec<_>>()
        .join(", ")
}