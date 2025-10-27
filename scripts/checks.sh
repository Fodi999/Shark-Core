#!/usr/bin/env bash
set -euo pipefail

# Local verification script following the SPARK-like checklist
# Run from repo root: ./scripts/checks.sh

echo "1) cargo clean"
cargo clean

echo "2) cargo fmt --check"
cargo fmt --all -- --check

echo "3) cargo clippy (strict)"
cargo clippy --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::indexing_slicing

echo "4) cargo test (single thread, deterministic output)"
cargo test -- --test-threads=1 --nocapture

echo "5) (optional) cargo mirai -- requires mirai installed"
if command -v cargo-mirai >/dev/null 2>&1 || command -v mirai >/dev/null 2>&1; then
	echo "Running cargo mirai..."
	# prefer cargo-mirai if available
	if command -v cargo-mirai >/dev/null 2>&1; then
		cargo-mirai || echo "cargo-mirai failed (non-fatal in this script)"
	else
		cargo mirai || echo "cargo mirai failed (non-fatal in this script)"
	fi
else
	echo "cargo-mirai / cargo mirai not found; skipping MIRAI step. Install MIRAI to run this step."
fi

echo "6) (optional) cargo prusti -- requires prusti installed"
if command -v cargo-prusti >/dev/null 2>&1 || command -v prusti >/dev/null 2>&1 || command -v cargo-prusti-server >/dev/null 2>&1; then
	echo "Running cargo prusti..."
	# try cargo-prusti first
	if command -v cargo-prusti >/dev/null 2>&1; then
		cargo-prusti || echo "cargo-prusti failed (non-fatal in this script)"
	else
		cargo prusti || echo "cargo prusti failed (non-fatal in this script)"
	fi
else
	echo "cargo prusti not found; skipping Prusti step. Install Prusti to run this step."
fi

echo "All requested checks completed (or optional steps skipped)."
