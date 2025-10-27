/// Decode raw model output into a conservative, human-readable string.
///
/// This function performs a minimal, lossy post-processing step suitable for
/// UI presentation while preserving the original raw string elsewhere for
/// auditing. It filters the input to keep only simple printable characters
/// (letters, digits, space and a few punctuation marks), removes other
/// 'noise' tokens, and then performs tiny normalization: ensure the first
/// letter is capitalized and that the sentence ends with a terminal
/// punctuation mark ('.', '!' or '?'). If nothing readable can be produced,
/// the function returns a short fallback message.
///
/// Note: this is intentionally conservative — it does not attempt to
/// reconstruct words or correct grammar; for that, use a higher-level
/// rule-based corrector or retraining pipeline.
pub fn decode_raw(raw: &str) -> String {
    // Фильтруем только разрешённые символы и восстанавливаем структуру предложения
    let mut output = String::new();
    for ch in raw.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '.' | ',' | '?' | '!' => output.push(ch),
            _ => {} // игнорируем шумовые токены
        }
    }

    // Попробуем минимально нормализовать текст
    if output.is_empty() {
        return "(не удалось расшифровать ответ)".to_string();
    }

    // Заглавная первая буква, точка в конце
    let mut chars = output.chars();
    if let Some(first) = chars.next() {
        let mut result = String::new();
        result.push(first.to_ascii_uppercase());
        result.push_str(chars.as_str());
        if !result.ends_with('.') && !result.ends_with('!') && !result.ends_with('?') {
            result.push('.');
        }
        return result;
    }

    output
}
