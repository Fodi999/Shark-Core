use std::collections::HashMap;

/// Load grammar mappings from a CSV file.
///
/// The CSV should have lines like "key,value" where key is a single character
/// and value is the word/phrase to map to. The first line (header) is skipped.
/// If the file can't be read, an empty map is returned.
/// For long values, only the first word is kept for brevity.
pub fn load_grammar_from_csv(path: &str) -> HashMap<char, String> {
    let mut map = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines().skip(1) {
            if let Some((key_part, val_part)) = line.split_once(',') {
                let key_str = key_part.trim().trim_matches('"');
                let val_str = val_part.trim().trim_matches('"');
                if let Some(key) = key_str.chars().next() {
                    // Take only the first word for brevity
                    let short_val = val_str.split_whitespace().next().unwrap_or(&val_str).to_string();
                    map.insert(key, short_val);
                }
            }
        }
    }
    map
}

/// Small toy grammar/dictionary used to interpret noisy model output.
///
/// This module loads mappings from a CSV file to interpret decoded tokens.
/// It is a lightweight experiment helper that demonstrates how a rule-based
/// "interpretation" layer can map decoded tokens to approximate meaning.
/// It is not a proper lexicon — it's only useful for toy demos and diagnostics.
pub fn build_grammar_map() -> HashMap<char, String> {
    load_grammar_from_csv("crates/predict/data/knowledge/knowledge_alphabet.csv")
}

/// Translate a "raw" (decoded) answer into an approximate phrase based on
/// the loaded grammar map.
///
/// The function walks characters in `raw` and appends mapped words when a
/// character is present in the grammar map. Words are separated by spaces.
/// If no mapping is found, a fallback message is returned indicating that
/// the answer is unclear and retraining may be required.
pub fn interpret(raw: &str) -> String {
    let map = build_grammar_map();
    let mut result = String::new();

    for ch in raw.chars() {
        let lower_ch = ch.to_ascii_lowercase();
        if let Some(word) = map.get(&lower_ch) {
            result.push_str(word);
            result.push(' ');
        }
    }

    if result.is_empty() {
        "(непонятный ответ — требуется переобучение)".to_string()
    } else {
        result.trim().to_string()
    }
}
