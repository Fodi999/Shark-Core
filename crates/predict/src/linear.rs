#![forbid(unsafe_code)]

/// Simple dense (linear) layer: out = W * in + b
/// Dense layer container
pub struct Linear {
    /// input dimension
    pub in_dim: usize,
    /// output dimension
    pub out_dim: usize,
    /// weights in row-major order: out_dim x in_dim
    pub weights: Vec<f32>,
    /// bias vector of length out_dim
    pub bias: Vec<f32>,
}

impl Linear {
    /// Create a Linear layer from raw weight buffer. If buffer too small, fill zeros.
    pub fn from_raw(in_dim: usize, out_dim: usize, raw: &[f32]) -> Self {
        let expected = out_dim * in_dim;
        let mut weights = vec![0.0_f32; expected];
        for i in 0..expected.min(raw.len()) {
            weights[i] = raw[i];
        }
        let mut bias = vec![0.0_f32; out_dim];
        // if raw contains bias after weights, copy
        if raw.len() >= expected + out_dim {
            for i in 0..out_dim {
                bias[i] = raw[expected + i];
            }
        }
        Self { in_dim, out_dim, weights, bias }
    }

    /// Forward pass for a single input vector
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut out = vec![0.0_f32; self.out_dim];
        for o in 0..self.out_dim {
            let mut s = 0.0_f32;
            let base = o * self.in_dim;
            for i in 0..self.in_dim {
                s += self.weights[base + i] * input[i];
            }
            s += self.bias[o];
            out[o] = s;
        }
        out
    }
}
