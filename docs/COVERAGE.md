# Coverage guide for unidecode-rs

This project ships a pure Rust transliteration library plus optional Python bindings (PyO3). We track two distinct coverage dimensions:

1. Rust line & branch coverage (cargo-llvm-cov -> `lcov.info` + HTML report)
2. Python test coverage (pytest + pytest-cov -> `coverage.xml`)

Providing both lets CI and Codecov present a unified quality picture while allowing language–specific evolution.

---
## 1. Rust coverage

Convenience cargo aliases (see `.cargo/config.toml`) mirror the style used in `slugify-rs`.

### Fast HTML report
```bash
cargo coverage
```
Generates an interactive HTML report under `target/llvm-cov/html/index.html`.

### LCOV artifact (CI / Codecov)
```bash
cargo coverage-lcov
```
Outputs `lcov.info` at the crate root (filtered to exclude build script noise and binaries).

### Manual (expanded form)
```bash
rustup component add llvm-tools-preview            # once
cargo install cargo-llvm-cov                       # once
cargo llvm-cov --workspace --lcov --output-path lcov.info \
  --ignore-filename-regex '/usr/src|/rustc-|src/bin/.*|bin/.*'
```

### Current state (snapshot)
Core logic lines (excluding generated tables) are effectively near 100% covered: tests exercise
- ASCII fast path
- Override table hit/miss (binary search branches)
- Latin‑1 table path & generic block lookup
- All error policies (Default, Ignore, Replace, Preserve, Invalid, Strict) including strict error index edge cases
- Estimation fallback path (`estimated == 0`)
- Idempotence (unit + property tests)
- Random Unicode robustness (property test: no panics, ASCII guarantee)

Remaining uncovered lines are either generated mapping tables or structurally trivial lines with no business logic risk.

---
## 2. Python coverage

```bash
python -m venv .venv
. .venv/bin/activate
pip install --upgrade pip maturin pytest pytest-cov coverage
maturin develop --release --features python
pytest --cov=. --cov-report=xml:coverage.xml --cov-report=term-missing
```

Artifacts:
- `lcov.info`  (flag: `rust`)
- `coverage.xml` (flag: `python`)

Upload both in CI with separate flags; Codecov merges into a combined dashboard.

---
## 3. (Planned) unified local script

A helper script `scripts/coverage.sh` (to be added) will:
1. Create/activate venv & build the PyO3 wheel (develop mode)
2. Run Python tests with coverage -> `coverage.xml`
3. Run `cargo coverage-lcov` -> `lcov.info`
4. Print a concise summary (totals, uncovered logic lines)

---
## 4. Adding new tests
Rust:
- Prefer small, focused deterministic tests in `tests/` or per‑module `#[cfg(test)]` blocks.
- For invariants (ASCII output, idempotence), prefer property tests (see `tests/property_ascii.rs`).

Python:
- Golden / regression tests for specific code point classes.
- Upstream test harness parity adaptation lives under `tests/python/` (runs original reference suite against the Rust core). Use it to guard behavioral drift.

---
## 5. Quality thresholds
- Core logic: keep effective coverage ≥ 95%; strive for 100% on new code before merging.
- Generated tables are excluded from “meaningful” coverage discussions (they are data, validated indirectly by lookup tests).
- Any new branch (match arm, error handling path) must ship with at least one direct test.

---
## 6. Future enhancements
| Area | Rationale | Action |
|------|-----------|--------|
| SIMD ASCII scan | Potential speedup on large mixed strings | Add feature‑gated implementation + benchmark + property tests |
| Table density metrics | Detect sparse blocks for compression | Add analysis script; ensure coverage via synthetic block tests |
| Fuzz (cargo-fuzz) | Hardening against malformed UTF‑8 boundaries | Add minimal harness ensuring no panics + ASCII invariant |
| Coverage diff gate | Prevent regression in PRs | Add CI job using `cargo llvm-cov --fail-under-lines <N>` |

---
## 7. Troubleshooting
| Symptom | Fix |
|---------|-----|
| HTML report empty | Ensure `llvm-tools-preview` installed and no prior `cargo clean` race |
| Low reported % but tests thorough | Generated source (tables) inflates denominator—filter via ignore regex or interpret logic subset only |
| Codecov shows single language | Verify both uploads have distinct `-F rust` / `-F python` flags |

---
## 8. Summary
The project currently maintains exhaustive coverage for executable logic; remaining uncovered lines are non-critical data artifacts. Property tests and upstream parity harness minimize behavioral regression risk. Future work centers on performance experiments and automated gating of coverage regressions.

