#![forbid(unsafe_code)]

use crate::loader;
use crate::core;
use crate::linear::Linear;
use crate::tokenizer::ALPHABET;

/// Small toy model with a tiny embedding + MLP for deterministic generation.
pub struct Model {
    /// first linear layer (embed -> hidden)
    pub lin1: Linear,
    /// second linear layer (hidden -> vocab)
    pub lin2: Linear,
    /// vocabulary size used by the decoder
    pub vocab_size: usize,
}

impl Model {
    /// Load weights and construct a tiny model. If weights are missing or too small,
    /// layers are created with zero weights (deterministic fallback).
    pub fn load(path: &str) -> Self {
        // load raw bytes and convert to f32 little-endian chunks
        let raw_bytes = loader::load_weights(path).unwrap_or_default();
        let mut floats = vec![];
        let mut i = 0usize;
        while i + 4 <= raw_bytes.len() {
            let b = &raw_bytes[i..i+4];
            let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            floats.push(v);
            i += 4;
        }

        // model dims (toy)
        let embed = 32usize;
        let hidden = 64usize;
        let vocab = ALPHABET.len();

        // carve floats into layers: lin1 expects embed->hidden, lin2 hidden->vocab
        let needed1 = embed * hidden + hidden;
        let needed2 = hidden * vocab + vocab;
        let mut offset = 0usize;
        let slice1 = if floats.len() >= offset + needed1 { &floats[offset..offset+needed1] } else { &[] };
        offset += needed1;
        let slice2 = if floats.len() >= offset + needed2 { &floats[offset..offset+needed2] } else { &[] };

        let lin1 = Linear::from_raw(embed, hidden, slice1);
        let lin2 = Linear::from_raw(hidden, vocab, slice2);
        Self { lin1, lin2, vocab_size: vocab }
    }

    /// Generate a short response from a context string using a very small autoreg loop.
    /// This is deterministic and not intended to be a real language model.
    pub fn generate(&self, context: &str) -> String {
        // simple tokenization: split words, but we'll generate characters from alphabet
        let toks = context.as_bytes();
        // compute a simple seed vector from context bytes: embed size = lin1.in_dim
        let embed_dim = self.lin1.in_dim;
        let mut emb = vec![0.0f32; embed_dim];
        for (i, &b) in toks.iter().enumerate() {
            emb[i % embed_dim] += (b as f32) * 0.01;
        }

        // autoregressive character generation (max 64 chars)
        // create a deterministic RNG seeded from context
        let mut seed: u64 = 0x9e3779b97f4a7c15u64;
        for &b in toks.iter() {
            seed = seed.wrapping_mul(31).wrapping_add(b as u64);
        }
        let mut rng = core::make_rng(seed);

        let mut out = Vec::new();
        for _ in 0..64 {
            let h = self.lin1.forward(&emb);
            // ReLU
            let h: Vec<f32> = h.into_iter().map(|v| if v>0.0 { v } else { 0.0 }).collect();
            let mut logits = self.lin2.forward(&h);
            // to f32 slice for softmax
            core::softmax(&mut logits);
            // sample from distribution using RNG
            let idx = core::sample_index(&logits, &mut rng);
            out.push(ALPHABET[idx]);
            // update emb with last char to have some state
            let last = ALPHABET[idx] as f32;
            for i in 0..embed_dim { emb[i] = emb[i] * 0.9 + (last * (i as f32 + 1.0) * 1e-3); }
        }

        String::from_utf8_lossy(&out).to_string()
    }
}

/// Minimal two-layer model that matches the example "Shark-style" loader.
///
/// Loads f32 weights from a file and slices them into two linear layers. The
/// struct itself is public but internal fields are private; use `load` and
/// `forward` to interact with the model.
pub struct SimpleModel {
    embed: usize,
    hidden: usize,
    vocab: usize,
    layer1: Linear,
    layer2: Linear,
}

impl SimpleModel {
    /// Load f32 weights (little-endian) and construct two Linear layers.
    /// Layout expected: w1 (embed*hidden), b1 (hidden), w2 (hidden*vocab), b2 (vocab)
    pub fn load(path: &str, embed: usize, hidden: usize, vocab: usize) -> Self {
        let data = crate::loader::load_f32_file(path).expect("cannot read weights");
    let needed1 = embed * hidden + hidden;

        // guard against too-small data by using as-slice or empty fallback
        let w1 = if data.len() >= embed * hidden { &data[..embed * hidden] } else { &[] };
        let b1 = if data.len() >= embed * hidden + hidden { &data[embed * hidden..needed1] } else { &[] };
        let start_w2 = needed1;
        let end_w2 = needed1 + hidden * vocab;
        let w2 = if data.len() >= end_w2 { &data[start_w2..end_w2] } else { &[] };
    let b2 = if data.len() >= end_w2 + vocab { &data[end_w2..end_w2 + vocab] } else { &[] };

        // assemble raw buffers as weights followed by biases for from_raw helper
        let mut raw1 = Vec::with_capacity(w1.len() + b1.len());
        raw1.extend_from_slice(w1);
        raw1.extend_from_slice(b1);
        let mut raw2 = Vec::with_capacity(w2.len() + b2.len());
        raw2.extend_from_slice(w2);
        raw2.extend_from_slice(b2);

        let l1 = Linear::from_raw(embed, hidden, &raw1);
        let l2 = Linear::from_raw(hidden, vocab, &raw2);
        Self { embed, hidden, vocab, layer1: l1, layer2: l2 }
    }

    /// Forward pass: input is expected to be `embed`-long. Applies tanh after first layer
    /// to mimic the lightweight activation in the example.
    pub fn forward(&self, input: &[f32], _arena: &mut crate::core::Arena) -> Vec<f32> {
        let h = self.layer1.forward(input);
        let h: Vec<f32> = h.into_iter().map(|v| v.tanh()).collect();
        self.layer2.forward(&h)
    }
}
