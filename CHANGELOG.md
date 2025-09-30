# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## 0.1.0 - Initial release
- Initial Rust implementation of Unidecode-style transliteration.
- Provides a `unidecode_rs` crate with optional PyO3/Python bindings (feature `python`).
- A Python shim and parity test harness were added to match upstream `unidecode` behavior and to run upstream tests for parity verification.
- Manylinux wheel build support via `maturin` and a GitHub Actions workflow for publishing (supports OIDC Trusted Publisher and a `PYPI_API_TOKEN` fallback).
- Benchmarks comparing the Rust extension vs pure-Python implementation included.
- CI updated to run `cargo fmt` and `cargo clippy` and to enforce formatting and linting.

### Notes
- This is the initial public release. The goal for this version is feature parity with the Python `unidecode` package; users are encouraged to run the included parity tests.
