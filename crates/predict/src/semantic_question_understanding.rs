use std::collections::HashMap;

/// Interpret semantic meaning of questions and provide structured responses.
pub fn interpret_question(input: &str, knowledge: &HashMap<String, String>) -> Option<String> {
    let normalized = input.to_lowercase().trim().to_string();

    // Greetings
    if normalized.contains("привет") || normalized.contains("hello") || normalized == "hi" {
        return Some("Привет! Я Shark-Core. Задайте вопрос, например: 'что такое алгоритм?' или 'почему буквы важны?'.".to_string());
    }

    // Math expressions
    if normalized.chars().any(|c| c.is_digit(10)) && (normalized.contains('+') || normalized.contains('-') || normalized.contains('*') || normalized.contains('/')) {
        if let Ok(result) = meval::eval_str(&normalized.replace("=", "").replace("?", "")) {
            return Some(format!("Результат: {:.2}", result));
        }
    }

    if normalized.starts_with("что такое") {
        let concept = normalized
            .replace("что такое", "")
            .replace("?", "")
            .trim()
            .to_string();

        if let Some(answer) = knowledge.get(&concept) {
            return Some(format!("\"{}?\" — \"{}\".", input.trim_end_matches('?'), answer));
        } else {
            return Some(format!("Понятие \"{}\" пока неизвестно.", concept));
        }
    }

    if normalized.contains("приведи пример") {
        let concept = normalized.replace("приведи пример", "").trim().to_string();
        if let Some(example) = knowledge.get(&format!("{}_example", concept)) {
            return Some(format!("Пример для {}: {}.", concept, example));
        }
    }

    if normalized.starts_with("почему") {
        let cause = normalized.replacen("почему", "", 1).trim().to_string();

        if cause.contains("буквы") || cause.contains("алфавит") {
            return Some("Потому что буквы составляют основу письменного языка и коммуникации.".to_string());
        } else if cause.contains("2+2") {
            return Some("Потому что операция сложения определяет результат объединения двух чисел.".to_string());
        } else if cause.contains("истина") || cause.contains("ложь") {
            return Some("Потому что в логике 'истина' и 'ложь' — противоположные значения булевой алгебры.".to_string());
        } else if cause.contains("алгоритм") {
            return Some("Потому что алгоритм задаёт последовательность шагов для решения задачи.".to_string());
        } else {
            return Some(format!("Потому что '{}' основано на соответствующем правиле.", cause));
        }
    }

    if normalized.contains("в чём разница") {
        return Some("Разница между понятиями заключается в их функции или свойстве.".to_string());
    }

    Some("Я не понимаю этот вопрос полностью. Попробуйте спросить 'что такое [понятие]', 'почему [что-то]' или 'приведи пример [чего-то]'.".to_string())
}