#![forbid(unsafe_code)]

/// Public alphabet used by model decoders. Expanded to include lowercase letters,
/// space and common punctuation so the generator can produce readable text.
pub const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?+-=*/()[]{}<>:'\"";

/// Very small tokenizer that splits on whitespace and punctuation.
pub fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

/// Detokenize back into a string (join with spaces)
pub fn detokenize(tokens: &[String]) -> String {
    tokens.join(" ")
}
