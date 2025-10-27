Shark-Core

Shark-Core is a minimal local inference playground built in Rust. It's
designed to be small, auditable, and deterministic — suitable for
experiments where clarity and safety matter.

Key points
- Language: Rust (no unsafe code)
- Purpose: tiny, transparent inference building blocks — small dense
	layers, a toy two-layer model loader, deterministic RNGs, and local
	dialog memory.
- Features implemented:
	- softmax & sampling utilities
	- small `Linear` dense layer and `SimpleModel` loader (f32 blobs)
	- local `Memory` persistence (bincode) for dialog history
	- `chat` CLI (interactive REPL)

Quickstart

Build and run the chat REPL:

```bash
cargo build -p predict
cargo run -p predict --bin chat
```

Files of interest:
- `crates/predict/src/core.rs` — softmax, RNG, arena
- `crates/predict/src/linear.rs` — tiny dense layer
- `crates/predict/src/model.rs` — SimpleModel loader + Model
- `crates/predict/src/memory.rs` — dialog persistence
- `crates/predict/src/bin/chat.rs` — interactive CLI

License & contribution

Small, experimental code. Keep changes minimal and well-tested.
