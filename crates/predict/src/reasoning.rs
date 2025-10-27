use std::collections::HashMap;

/// Detect query mode based on keywords.
pub fn detect_mode(input: &str) -> &'static str {
    let lower = input.to_lowercase();
    if lower.contains('?') || lower.starts_with("что") || lower.starts_with("кто") || lower.starts_with("как") || lower.starts_with("почему") {
        "question"
    } else if lower.contains("покажи") || lower.contains("как") || lower.contains("сделай") {
        "instruction"
    } else {
        "statement"
    }
}

/// Simple trigram similarity for strings (0.0 to 1.0).
pub fn trigram_similarity(a: &str, b: &str) -> f64 {
    let a_trigrams = get_trigrams(a);
    let b_trigrams = get_trigrams(b);
    let intersection: usize = a_trigrams.keys().filter(|k| b_trigrams.contains_key(*k)).count();
    let union = a_trigrams.len() + b_trigrams.len() - intersection;
    if union == 0 { 1.0 } else { intersection as f64 / union as f64 }
}

fn get_trigrams(s: &str) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len().saturating_sub(2) {
        let trigram = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
        *map.entry(trigram).or_insert(0) += 1;
    }
    map
}

/// Find closest knowledge entry by trigram similarity.
pub fn find_closest_concept(input: &str, knowledge: &std::collections::HashMap<String, String>) -> Option<(String, String)> {
    let knowledge_vec: Vec<(String, String)> = knowledge.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let mut best = None;
    let mut best_sim = 0.0;
    for (q, a) in knowledge_vec {
        let sim = trigram_similarity(input, &q);
        if sim > best_sim && sim > 0.3 { // threshold
            best_sim = sim;
            best = Some((q.clone(), a.clone()));
        }
    }
    best
}

/// Extract rule and example from answer.
pub fn parse_answer(a: &str) -> (String, String) {
    if let Some(ex_pos) = a.find("Пример:") {
        let rule = a[..ex_pos].trim().to_string();
        let example = a[ex_pos..].trim().to_string();
        (rule, example)
    } else {
        (a.to_string(), "".to_string())
    }
}

/// Build reasoned response based on mode.
pub fn reason_response(input: &str, knowledge: &std::collections::HashMap<String, String>) -> String {
    // Universal handler for "what is ..." questions
    if input.to_lowercase().starts_with("что такое") {
        let concept = input["что такое".len()..].trim().trim_end_matches('?').to_lowercase();
        if let Some(answer) = knowledge.get(&concept) {
            return format!("\"{}\" — \"{}\".", input, answer);
        } else {
            return format!("Понятие \"{}\" пока неизвестно.", concept);
        }
    }

    let mode = detect_mode(input);
    match mode {
        "question" => {
            if let Some((q, a)) = find_closest_concept(input, knowledge) {
                let (rule, example) = parse_answer(&a);
                format!("{} — {}. {}", q, rule, example)
            } else {
                "Не нашел подходящего ответа в знаниях.".to_string()
            }
        }
        "instruction" => {
            if let Some((q, a)) = find_closest_concept(input, knowledge) {
                format!("Инструкция: {}. {}", q, a)
            } else {
                "Не понял инструкцию.".to_string()
            }
        }
        _ => {
            if let Some((q, a)) = find_closest_concept(input, knowledge) {
                format!("Утверждение: {}. {}", q, a)
            } else {
                "Не нашел связи.".to_string()
            }
        }
    }
}