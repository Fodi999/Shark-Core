# Verification process (sketch)

This document describes how we verify key functions and ensure determinism, safety and testability.

Principles
- Forbid unsafe code across all crates: `#![forbid(unsafe_code)]`.
- Deny `unwrap`/`expect`/`panic`/indexing: use Result and explicit checks.
- Determinism: identical inputs and fixed seeds produce identical outputs.
- Auditable pure functions: indicators must be pure, take slices and return owned results.

Verification steps
1. Unit tests for all pure functions (indicators) with fixed inputs and expected outputs.
2. Property tests where useful (later), but with deterministic RNG seeds.
3. Backtest regression tests with small synthetic datasets to validate PnL and slippage/fees.
4. CI to run `cargo test` and a reproducible nightly integration test.

Contracts on functions
- Document input shapes, valid ranges and error modes in doc comments.
- Use `Result<T, Error>` for recoverable errors, define explicit error enums.

Determinism
- If RNG is used, accept a `Seed`/`Rng` as input or a deterministic RNG type.
- Avoid global state; pass configuration explicitly.

Reporting
- Backtest must emit deterministic `Report` with metrics (PnL, max drawdown, trades, slippage total, commissions total).

## SPARK-like verification checklist and commands

Follow these steps to reach SPARK-like assurance (static typing + formal checks).

1) Static checks with Clippy (deny warnings and specific lints):

	cargo clippy --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::indexing_slicing

2) Formal contracts with Prusti (requires Prusti installed):

	Annotate functions with `prusti_contracts` attributes, e.g.:

	use prusti_contracts::*;

	#[requires(den != 0.0)]
	#[ensures(result.is_finite())]
	fn safe_div(num: f64, den: f64) -> f64 {
		 num / den
	}

	Then run:

	cargo prusti

3) Path-sensitive analysis with MIRAI (requires MIRAI installed):

	cargo mirai

4) Determinism and purity checks (run tests serially, repeatable):

	cargo test -- --test-threads=1 --nocapture

5) Property-based testing (QuickCheck / Proptest) to check invariants across many inputs.

	Add `quickcheck` / `proptest` to `dev-dependencies` and write properties in the `tests/` or crate tests.

6) Zero warnings / strict build:

	RUSTFLAGS='-D warnings' cargo build --all

Example full cycle (local):

	cargo clean
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test -- --test-threads=1
	# optional:
	# cargo mirai
	# cargo prusti

Also see `scripts/checks.sh` at the repo root to run the main steps locally.

## Prusti and MIRAI — how to run (optional, external tools)

If you want to push verification further (true formal proofs / path-sensitive analysis), install and run Prusti and MIRAI locally. These tools are optional and require extra setup.

Prusti (formal verification of contracts)

- Install Prusti server (see Prusti docs for platform-specific instructions):

	cargo install prusti-server

- Annotate functions with `prusti_contracts` and run:

	cargo prusti

Example annotation:

```rust
use prusti_contracts::*;

#[requires(x > 0.0)]
#[ensures(result > 0.0)]
fn sqrt_pos(x: f64) -> f64 {
		x.sqrt()
}
```

Notes:
- Prusti currently supports a subset of Rust; some language features and dependencies might not be supported.
- When Prusti fails to prove a function, it will typically produce a counterexample or a proof obligation you can inspect.

MIRAI (path-sensitive static analyzer)

- Install MIRAI (see MIRAI docs for the latest instructions):

	cargo install mirai

- Run:

	cargo mirai

Notes:
- MIRAI will analyze asserts, possible panics and many other properties across control-flow paths.
- MIRAI can be heavier and produce many warnings; treat it as an additional verification step after unit tests and clippy.

CI note

If you want to run Prusti/MIRAI in CI, prefer separate jobs with dedicated runners because both may require special toolchains or additional installation time.
