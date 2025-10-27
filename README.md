Verified Core — minimal scaffold

This workspace is a starting point for a verified, deterministic trading core.

Structure (scaffolded):
- crates/indicators  — pure indicator implementations (SMA/EMA...) with tests
- crates/backtest    — deterministic backtest engine with fees/slippage and reports
- docs/              — verification and safety docs

Policies applied in crates: `#![forbid(unsafe_code)]` and denies for unwrap/expect/panic/indexing.

Next steps:
- implement more indicators
- add data ingestion and feature store crates
- implement engine + api crates
# Shark-Core
