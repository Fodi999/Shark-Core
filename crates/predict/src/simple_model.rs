//! Simple linear layer for demonstration (no unsafe, pure Rust)
use crate::core::Arena;

/// Simple Linear layer storing weights as rows
pub struct SimpleLinear {
    /// Weight matrix rows (out_dim x in_dim)
    pub weights: Vec<Vec<f32>>,
    /// Bias vector (out_dim)
    pub bias: Vec<f32>,
}

impl SimpleLinear {
    /// Create a new SimpleLinear layer from per-row weights and bias.
    pub fn new(weights: Vec<Vec<f32>>, bias: Vec<f32>) -> Self {
        Self { weights, bias }
    }

    /// Forward pass: y = Wx + b
    pub fn forward(&self, input: &[f32], _arena: &mut Arena) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.bias.len());
        for (i, row) in self.weights.iter().enumerate() {
            let mut sum = self.bias[i];
            for (w, &x) in row.iter().zip(input.iter()) {
                sum += w * x;
            }
            out.push(sum);
        }
        out
    }
}
