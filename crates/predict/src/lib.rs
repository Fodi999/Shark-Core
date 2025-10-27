#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
#![deny(missing_docs, unused_must_use)]

//! Shark-Core: minimal local inference building blocks
//!
//! This crate provides a very small, auditable core used for local
//! inference experiments (Shark-style). The goal is transparency and
//! simplicity — small dense `Linear` layers, a tiny `SimpleModel` loader
//! that reads little-endian `f32` weights, a compact `Arena`/core helpers,
//! a lightweight `Memory` persistence, and a CLI `chat`.
//!
//! Layout (important files):
//! - `core.rs` — softmax, RNG helpers, arena placeholder
//! - `linear.rs` — tiny dense layer (`Linear::from_raw` + `forward`)
//! - `loader.rs` — helper to load f32 weight blobs
//! - `model.rs` — `Model` + `SimpleModel` convenience loader
//! - `memory.rs` — dialog persistence (bincode)
//! - `tokenizer.rs` — small tokenizer (char/word helpers)
//! - `bin/chat.rs` — REPL that uses `AI` (model + memory)

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

/// Deterministic binary search guess using a feedback closure.
///
/// `feedback` should behave like `guess.cmp(&target)`: return
/// - Ordering::Less when guess < target,
/// - Ordering::Greater when guess > target,
/// - Ordering::Equal when guess == target.
pub fn guess_number<F>(mut feedback: F) -> i32
where
    F: FnMut(i32) -> std::cmp::Ordering,
{
    let (mut low, mut high) = (0i32, 99i32);
    while low <= high {
        let mid = low + (high - low) / 2;
        match feedback(mid) {
            std::cmp::Ordering::Less => {
                // mid < target
                low = mid + 1;
            }
            std::cmp::Ordering::Greater => {
                // mid > target
                high = mid - 1;
            }
            std::cmp::Ordering::Equal => return mid,
        }
    }
    // If not found, return low as fallback
    low
}

/// Simple wrapper that, given a secret `target`, uses `guess_number` to find it.
pub fn guess(target: i32) -> i32 {
    guess_number(|n| n.cmp(&target))
}

/// Probabilistic guess that uses a seeded RNG so it's deterministic per seed.
/// It tries to converge on `target` by random steps shrinking over iterations.
pub fn probabilistic_guess(target: i32, seed: u64) -> i32 {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut guess = rng.gen_range(0..100);
    let mut step = 10i32;
    for _ in 0..20 {
        if guess == target {
            return guess;
        } else if guess < target {
            let delta = rng.gen_range(1..=step);
            guess = (guess + delta).min(99);
        } else {
            let delta = rng.gen_range(1..=step);
            guess = (guess - delta).max(0);
        }
        step = std::cmp::max(1, step / 2);
    }
    guess
}

/// Very small least-squares linear regressor using provided `(x,y)` pairs.
/// Returns predicted y for given `x0`.
pub fn linear_regressor_predict(pairs: &[(f64, f64)], x0: f64) -> Option<f64> {
    if pairs.is_empty() {
        return None;
    }
    let n = pairs.len() as f64;
    let sum_x = pairs.iter().map(|(x, _)| x).sum::<f64>();
    let sum_y = pairs.iter().map(|(_, y)| y).sum::<f64>();
    let sum_xy = pairs.iter().map(|(x, y)| x * y).sum::<f64>();
    let sum_x2 = pairs.iter().map(|(x, _)| x * x).sum::<f64>();
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < std::f64::EPSILON {
        return None;
    }
    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;
    Some(slope * x0 + intercept)
}

/// Hidden polynomial function used for symbolic discovery examples.
/// f(x) = 3*x^2 - 2*x + 7
pub fn hidden_function(x: f64) -> f64 {
    3.0 * x * x - 2.0 * x + 7.0
}

/// Discover coefficients (a, b, c) of a quadratic polynomial by simple
/// gradient-descent-like updates. Deterministic when given a `seed`.
pub fn discover_equation(seed: u64) -> (f64, f64, f64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut a = rng.gen_range(-10.0..10.0);
    let mut b = rng.gen_range(-10.0..10.0);
    let mut c = rng.gen_range(-10.0..10.0);

    // learning rate and iterations chosen for the toy example
    let lr = 0.0015;
    for _ in 0..20_000 {
        let x = rng.gen_range(-5.0..5.0);
        let predicted = a * x * x + b * x + c;
        let actual = hidden_function(x);
        let error = predicted - actual;

        // gradient-like updates
        a -= lr * error * x * x;
        b -= lr * error * x;
        c -= lr * error;
    }

    (a, b, c)
}

/// Core utilities: softmax, RNG helpers, arena placeholder.
pub mod core;
/// Minimal model container and generation helpers.
pub mod model;
/// Linear (dense) layer helper.
pub mod linear;
/// Training helpers (tiny demo loader)
pub mod train;
/// Reasoner: stepwise explanation and reasoning logs.
pub mod reasoner;
/// Self-repair utilities: scan missing/broken modules and restore minimal stubs.
pub mod self_repair;
/// Knowledge environment helpers (expand directories, merge sources)
pub mod knowledge_env;
/// Simple integrator for polynomials and a small query interface.
pub mod integrator;
/// Weight loader (file helpers).
pub mod loader;
/// Local tokenizer utilities.
pub mod tokenizer;
/// Simple persistent memory for dialogs.
pub mod memory;
/// (internal) Scientist and small demo helpers remain in the crate but are
/// not re-exported as part of the public minimal API.
pub mod scientist;
mod simple_model;
/// Conservative decoding helpers for presenting model output.
///
/// Contains `decode_raw` which performs a minimal, lossy transformation of
/// raw model bytes into a human-readable string suitable for UI display.
pub mod decode;
pub use decode::decode_raw;
/// Small rule-based grammar/interpretation helpers (toy diagnostic layer).
pub mod grammar;
pub use grammar::interpret;
/// Contextual interpretation helpers (frequency-based word selection).
pub mod context;
pub use context::{interpret_contextual, load_memory_freq, save_memory_freq, update_memory_freq, interpret_contextual_with_memory};
/// Memory frequency helpers for persistent word learning.
pub mod memory_freq;
pub use memory_freq::*;
/// Reasoning helpers for query understanding and response building.
pub mod reasoning;
pub use reasoning::*;
/// Semantic question understanding helpers.
pub mod semantic_question_understanding;
pub use semantic_question_understanding::*;

use crate::model::Model;
use crate::memory::Memory;

/// Simple AI wrapper combining a `Model` and persistent `Memory`.
pub struct AI {
    /// underlying model used for generation
    pub model: Model,
    /// persistent memory for dialogs
    pub memory: Memory,
    /// knowledge base for reasoning
    pub knowledge: std::collections::HashMap<String, String>,
}

impl AI {
    /// Create AI by loading model weights from `path` and memory from default file.
    pub fn new(path: &str) -> Self {
        let model = Model::load(path);
        let memory = Memory::load("memory.db");
        let knowledge = load_knowledge_for_reasoning();
        Self { model, memory, knowledge }
    }

    /// Produce a response for the given input, persist dialog to memory.
    pub fn chat(&mut self, input: &str) -> String {
        // Try reasoning first if it looks like a query
        if detect_mode(input) != "statement" {
            let reasoned = reason_response(input, &self.knowledge);
            if !reasoned.contains("Не нашел") {
                let _ = self.memory.save_dialog(input, &reasoned);
                return reasoned;
            }
        }
        // Fallback to model generation
        let context = self.memory.build_context(input);
        let response_raw = self.model.generate(&context);
        let _ = self.memory.save_dialog(input, &response_raw);
        response_raw
    }
}

/// Load knowledge as map for reasoning.
pub fn load_knowledge_for_reasoning() -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut knowledge = HashMap::new();
    if let Ok(content) = std::fs::read_to_string("crates/predict/data/knowledge.csv") {
        for line in content.lines().skip(1) {
            if let Some((q, a)) = line.split_once(',') {
                knowledge.insert(q.trim().trim_matches('"').to_lowercase(), a.trim().trim_matches('"').to_string());
            }
        }
    }
    knowledge
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predict_hidden_number_binary() {
        let target = 23;
        let result = guess(target);
        assert_eq!(result, target);
    }

    #[test]
    fn probabilistic_prediction_close() {
        let target = 23;
        let result = probabilistic_guess(target, 42);
        assert!((result - target).abs() <= 5, "не попал в диапазон");
    }

    #[test]
    fn linear_regressor_learns_simple() {
        // y = 2x + 3
        let pairs = vec![(1.0, 5.0), (2.0, 7.0), (3.0, 9.0), (4.0, 11.0)];
        let pred = linear_regressor_predict(&pairs, 10.0);
        assert!(pred.is_some());
        let v = pred.unwrap();
        assert!((v - 23.0).abs() < 1e-8);
    }

    #[test]
    fn discover_hidden_equation_converges() {
        let (a, b, c) = discover_equation(42);
        // print for debugging if needed
        println!("Predicted coefficients: a={:.4}, b={:.4}, c={:.4}", a, b, c);
        assert!((a - 3.0).abs() < 0.1, "a не сходится");
        assert!((b + 2.0).abs() < 0.1, "b не сходится");
        assert!((c - 7.0).abs() < 0.1, "c не сходится");
    }
}
