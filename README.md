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

Common examples
```bash
# algebraic simplification (handled by the Reasoner)
cargo run -p predict --bin chat -- "Упростите (x+2)*(x-2)"

# run problems evaluation (produces docs/problems_report.md)
cargo run -p predict --bin chat -- "проверь задачи"

# trigger the scientist / discovery search
cargo run -p predict --bin chat -- "исследуй"
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

Available chat commands (examples used by the REPL)
- "проверь задачи" — run the problems evaluator and write `docs/problems_report.md`.
- "исследуй" / "исследуй закономерности" — run the scientist discovery/evolution routines.
- "Упростите ..." / "упростите ..." — request algebraic simplification (Reasoner).
- "объясн ..." / "рассужд ..." — ask for step-by-step reasoning from the Reasoner.
- "покажи структуру кода" — auto-scans Rust sources and writes `crates/predict/data/knowledge_rust.csv` and `docs/code_tree.md`.
- The system also tracks `unknowns` discovered during evaluation and attempts to re-solve them on startup (see data files below).

Data files (located in `crates/predict/data/`)
- `knowledge.csv` — Q→A knowledge base used for exact lookup and bootstrapping.
- `knowledge_rust.csv` — auto-generated summary of Rust source modules (from the scanner).
- `knowledge_science.csv` — discoveries / symbolic formulas found by the scientist (formula,mse,curiosity,date).
- `problems.csv` — evaluation problems (question,expected) used by the evaluator.
- `unknowns.csv` — recorded mismatches for later re-learning attempts.

Reasoner behavior (short)
- The Reasoner now attempts simple algebraic pattern matching (e.g. (a+b)*(a-b) → a^2 - b^2) before numeric evaluation.
- Numeric evaluation (via `meval`) is only used when an expression contains no alphabetic variables — this prevents attempts to numerically evaluate symbolic expressions.
- All explanations are appended to `docs/reasoning_log.md` with timestamps.

Next recommended improvements
- Replace ad-hoc CSV parsing/writing with the `csv` crate for robust quoting and streaming.
- Add a small planner module to break problems into steps and integrate with the Reasoner for stepwise solutions.
- Add unit tests for tokenizer, planner, and scientist memory loading.
- Run long-running research/evolution tasks in background threads to avoid blocking startup.

If you'd like, I can implement any of the suggestions above — tell me which to prioritize.

License & contribution

Small, experimental code. Keep changes minimal and well-tested.
