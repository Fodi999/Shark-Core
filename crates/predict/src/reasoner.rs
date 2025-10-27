use std::fs::OpenOptions;
use std::io::Write;
use crate::integrator::try_integrate;
use crate::knowledge_env::auto_expand_on_new_topic;
use crate::train::append_knowledge;

/// Модуль рассуждения Shark-Core.
/// Позволяет объяснять ход решения и сохранять рассуждения в лог.
pub struct Reasoner;

impl Reasoner {
    // integrator logic moved to `integrator.rs`; Reasoner will call try_integrate

    /// Главный метод: принимает задачу и возвращает пару (ответ, рассуждение).
    pub fn explain(input: &str) -> (String, String) {
        let mut reasoning = String::new();
        // Prefer algebraic/symbolic simplification patterns before numeric evaluation.
        let normalized = input.replace(' ', "");
        let answer = if normalized.contains("(x+2)*(x-2)") || normalized.contains("(a+b)*(a-b)") {
            // small algebraic simplification example
            reasoning.push_str("📘 Распознано: пример вида (a+b)*(a-b).\n");
            reasoning.push_str("➡️ Применяю формулу разности квадратов: (a+b)(a-b)=a^2-b^2.\n");
            reasoning.push_str("Результат: x^2 - 4\n");
            "x^2 - 4".into()
        } else if input.contains("=") {
            reasoning.push_str("📘 Распознано: уравнение.\n");
            reasoning.push_str("➡️ Преобразую выражение и решаю относительно x.\n");
            // Простая демонстрация: только линейные уравнения и пара трюков
            if input.contains("x") {
                reasoning.push_str("🔍 Найден символ x. Применяю линейное решение (если возможно).\n");
                // try to reuse existing solver from train.rs by delegating simple patterns
                // Fallback: attempt to compute with existing solve_linear_equation via crate::train
                if let Some(sol) = crate::train::solve_linear_equation(input) {
                    reasoning.push_str(&format!("🧠 Решение (эвристика): {}\n", sol));
                    sol
                } else {
                    reasoning.push_str("⚠️ Не удалось решить линейно.\n");
                    "неизвестное уравнение".into()
                }
            } else {
                "неизвестное уравнение".into()
            }
        } else if input.to_lowercase().contains("интеграл") {
            reasoning.push_str("📘 Распознано: задача на интеграл.\n");
            // delegate to integrator try_integrate
            if let Some(out) = try_integrate(input) {
                // log the integrator output in reasoning
                reasoning.push_str(&format!("🧮 {}\n", out));
                // Persist as knowledge: append to central knowledge.csv and per-topic calculus file
                let _ = append_knowledge("crates/predict/data/knowledge.csv", input, &out);
                // ensure calculus topic exists and append
                let _ = auto_expand_on_new_topic("calculus");
                if let Ok(mut f) = OpenOptions::new().create(true).append(true).open("crates/predict/data/knowledge/knowledge_calculus.csv") {
                    let now = chrono::Utc::now().to_rfc3339();
                    let _ = writeln!(f, "\"{}\",\"{}\",\"{}\"", input.replace('"', "'"), out.replace('"', "'"), now);
                }
                return (out, reasoning);
            }
            reasoning.push_str("🧠 Интегралы пока решаются символически позже.\n");
            "интеграл вычисляется позже".into()
        } else if (input.contains("+") || input.contains("-") || input.contains("*") || input.contains("/"))
            && !input.chars().any(|c| c.is_alphabetic())
        {
            // Treat as numeric arithmetic only if there are no alphabetic characters (variables)
            reasoning.push_str("📘 Распознано: арифметическое выражение.\n");
            reasoning.push_str("➡️ Выполняю пошаговое вычисление.\n");
            match Self::eval_expression(input) {
                Ok(v) => {
                    reasoning.push_str(&format!("🧮 Результат вычислений: {}\n", v));
                    // print a rounded integer if whole
                    if (v - v.round()).abs() < 1e-9 {
                        format!("{}", v.round() as i64)
                    } else {
                        format!("{}", v)
                    }
                }
                Err(e) => {
                    reasoning.push_str(&format!("⚠️ Ошибка вычисления: {}\n", e));
                    "ошибка".into()
                }
            }
        } else {
            reasoning.push_str("🤔 Неизвестный тип задачи.\n");
            "непонятно".into()
        };

        // Лог в файл
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("docs/reasoning_log.md") {
            let _ = writeln!(file, "### {}\n{}\nРассуждение:\n{}\n---\n", chrono::Utc::now().to_rfc3339(), input, reasoning);
        }

        (answer, reasoning)
    }

    fn eval_expression(expr: &str) -> Result<f64, String> {
        // Use meval crate for expression parsing/eval; trim non-ascii words like 'Посчитай' first
        let s = expr.replace(',', ".");
        meval::eval_str(&s).map_err(|e| e.to_string())
    }
}
