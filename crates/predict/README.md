Shark-Core (predict crate)

Minimal, auditable local inference crate used in the verified_core workspace.

Structure

- src/core.rs      — softmax, RNG helpers, arena placeholder
- src/linear.rs    — simple dense layer (row-major weights)
- src/loader.rs    — load f32 little-endian weight blobs
- src/model.rs     — Model + SimpleModel loader (two-layer tiny MLP)
- src/memory.rs    — simple bincode-backed dialog memory
- src/tokenizer.rs — small tokenizer helpers (char/word)
- src/bin/chat.rs  — REPL for local chat

Goals

- Small and readable code, suitable for inspection and verification.
- Deterministic behavior (seeded RNGs), no unsafe, no panics or unwraps.
- Easy to replace parts: tokenizer, model loader, sampling strategy.

Getting started

- Build and run the chat REPL:

```bash
cargo build -p predict
cargo run -p predict --bin chat
```

- Use `weights/model_int4.bin` as a tiny f32 weight blob for the demo model.

Contributing

Keep additions minimal and well-tested. Prefer explicit small files over
large, complex abstractions.
