#![forbid(unsafe_code)]

use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

/// Minimal softmax implementation for logits slice
pub fn softmax(logits: &mut [f32]) {
    if logits.is_empty() { return; }
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0_f32;
    for v in logits.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    if sum == 0.0 { return; }
    for v in logits.iter_mut() { *v /= sum; }
}

/// Simple RNG wrapper returning a seeded ChaCha8Rng
pub fn make_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

/// A trivial arena allocator placeholder (not a real arena)
pub struct Arena {
    // placeholder for future fast allocation
    _cap: usize,
}
impl Arena {
    /// Create a new arena placeholder with given capacity hint.
    pub fn new(cap: usize) -> Self { Self { _cap: cap } }
}

/// Sample index from probabilities using provided RNG
pub fn sample_index(probs: &[f32], rng: &mut ChaCha8Rng) -> usize {
    use rand::Rng;
    let r: f32 = rng.gen();
    let mut acc = 0.0_f32;
    for (i, &p) in probs.iter().enumerate() {
        acc += p;
        if r <= acc {
            return i;
        }
    }
    probs.len().saturating_sub(1)
}
