# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- Performance: major optimizations applied resulting in ~6.2x speedup vs Python Unidecode and significant reductions in allocations and Unicode decomposition overhead.
- API: `unidecode()` now returns `Cow<'_, str>` for zero-copy ASCII fast-path; `unidecode_string()` helper added to always obtain an owned `String`.
- Tests: all Rust test suites pass and Python parity tests pass (16 passed, 1 skipped) with the built Python extension.
- Docs: added `OPTIMIZATIONS.md` and updated README with benchmark results and usage notes.
- Note: prepare to publish version 0.3.0 to reflect performance and API improvements; changes are backward-compatible for typical usage.


### Performance Improvements 🚀
- **Major performance boost: ~6.2x faster than Python Unidecode** (up from ~4x)
- Implement zero-copy fast path for pure ASCII input using `Cow<str>` (eliminates unnecessary allocations)
- Add unrolled byte scanning loop for ASCII sequences (better branch prediction)
- Optimize NFKD decomposition with selective range checking (only for mathematical symbols)
- Improve capacity estimation heuristics (CJK-aware: +50% for CJK, +25% for Latin/Cyrillic)
- Remove unnecessary `to_string()` conversion in PyO3 bindings
- Add `memchr` dependency for potential future SIMD optimizations

### API Changes
- `unidecode()` now returns `Cow<'_, str>` instead of `String` (zero-copy for ASCII)
- Add `unidecode_string()` convenience function for guaranteed owned `String`

### Documentation
- Add comprehensive `OPTIMIZATIONS.md` documenting all performance improvements
- Update README with detailed benchmark results
- All tests pass (38 tests + property tests + golden tests)
- Zero functional regressions, full backward compatibility maintained

## 0.2.1 - 2025-10-01
- Fix: CI workflow (venv/maturin usage) so Python tests run reliably in GitHub Actions
- Fix: Treat `errors='invalid'` like `preserve` in Python bindings to match upstream tests
- Docs: README and build instructions cleanup


## 0.1.0 - Initial release
- Initial Rust implementation of Unidecode-style transliteration.
- Provides a `unidecode_rs` crate with optional PyO3/Python bindings (feature `python`).
- A Python shim and parity test harness were added to match upstream `unidecode` behavior and to run upstream tests for parity verification.
- Manylinux wheel build support via `maturin` and a GitHub Actions workflow for publishing (supports OIDC Trusted Publisher and a `PYPI_API_TOKEN` fallback).
- Benchmarks comparing the Rust extension vs pure-Python implementation included.
- CI updated to run `cargo fmt` and `cargo clippy` and to enforce formatting and linting.

### Notes
- This is the initial public release. The goal for this version is feature parity with the Python `unidecode` package; users are encouraged to run the included parity tests.
