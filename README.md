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

Build & Run

To run a single-shot prompt from the terminal (non-interactive):

```bash
cargo run -p predict --bin chat -- "Hello Shark-Core!"
```

To build an optimized macOS binary for release:

```bash
cargo build --release -p predict --bin chat
# resulting binary: target/release/chat
```

If you want to publish a binary release on GitHub, build the release
binary locally and upload the resulting `target/release/chat` to a
GitHub release (or use the `gh` CLI: `gh release create v0.1.0 target/release/chat`).


Files of interest:
- `crates/predict/src/core.rs` — softmax, RNG, arena
- `crates/predict/src/linear.rs` — tiny dense layer
- `crates/predict/src/model.rs` — SimpleModel loader + Model
- `crates/predict/src/memory.rs` — dialog persistence
- `crates/predict/src/bin/chat.rs` — interactive CLI

License & contribution

Small, experimental code. Keep changes minimal and well-tested.
