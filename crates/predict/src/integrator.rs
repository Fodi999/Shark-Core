//! Модуль для простого символьного интегрирования и анализа выражений
//! Поддерживает полиномы вида x^n и константы

use regex::Regex;
use std::option::Option;

/// Integrate a simple polynomial expression of the form `x^n`, `x`, or a constant `k` over [a,b].
pub fn integrate_polynomial(expr: &str, a: f64, b: f64) -> Option<f64> {
    // Поддержка базовых выражений: x^n, x, константы
    let expr = expr.trim();

    // Интеграл от x^n
    let pow_re = Regex::new(r"^x\^(\d+)$").unwrap();
    if let Some(caps) = pow_re.captures(expr) {
        let n: f64 = caps[1].parse().ok()?;
        let coeff = 1.0 / (n + 1.0);
        let result = coeff * (b.powf(n + 1.0) - a.powf(n + 1.0));
        return Some(result);
    }

    // Интеграл от x → x^2/2
    if expr == "x" {
        return Some(0.5 * (b.powi(2) - a.powi(2)));
    }

    // Интеграл от константы k → kx
    if let Ok(k) = expr.parse::<f64>() {
        return Some(k * (b - a));
    }

    None
}

/// Интерфейс верхнего уровня для Reasoner:
pub fn try_integrate(query: &str) -> Option<String> {
    // Пример запроса: "Посчитай интеграл x^2 от 0 до 2"
    if !query.to_lowercase().contains("интеграл") {
        return None;
    }
    // try to find pattern: интеграл <expr> от <a> до <b>
    // use a regex to extract expr, a, b
    // find expr between 'интеграл' and 'от'
    let re = Regex::new(r"(?i)интеграл\s+([^\s]+)\s+от\s+([+-]?\d+(?:[\.,]\d+)?)\s+до\s+([+-]?\d+(?:[\.,]\d+)?)").unwrap();
    if let Some(caps) = re.captures(query) {
        let expr = caps.get(1).map(|m| m.as_str()).unwrap_or("x");
        let a_s = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
        let b_s = caps.get(3).map(|m| m.as_str()).unwrap_or("0");
        let a = a_s.replace(',', ".").parse::<f64>().unwrap_or(0.0);
        let b = b_s.replace(',', ".").parse::<f64>().unwrap_or(0.0);
        let expr_trim = expr.trim();
        if let Some(res) = integrate_polynomial(expr_trim, a, b) {
            return Some(format!("Интеграл {} от {} до {} = {:.4}", expr_trim, a, b, res));
        }
    }
    None
}
