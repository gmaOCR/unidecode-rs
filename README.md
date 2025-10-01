
# unidecode-rs - Unicode → ASCII transliteration faithful to Python

[![CI](https://github.com/gmaOCR/unidecode-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/gmaOCR/unidecode-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/unidecode-rs.svg)](https://crates.io/crates/unidecode-rs)
[![Docs](https://docs.rs/unidecode-rs/badge.svg)](https://docs.rs/unidecode-rs)
[![Coverage](https://codecov.io/gh/gmaOCR/unidecode-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/gmaOCR/unidecode-rs)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Python Wheels](https://img.shields.io/badge/python-wheels-blue)](https://pypi.org/project/unidecode-pyo3)

Fast Rust implementation (optional Python bindings via PyO3) targeting bit‑for‑bit equivalence with Python [Unidecode]. Provides:

- Same output as `Unidecode` for all covered tables
- Noticeably higher performance (see perf snapshot in tests)
- Golden tests comparing dynamically against the Python version
- High coverage on critical paths (bitmap + per‑block dispatch)

## Repository layout

```
src/                # Core library sources + generated tables
benches/            # Criterion benchmarks (Rust)
scripts/            # Developer helper scripts (bench_compare, coverage)
tests/              # Rust integration & golden tests
tests/python/       # Python parity & upstream harness
python/             # Python shim for upstream-compatible API
docs/               # Coverage and performance documentation
```

## Quick summary

- Rust usage: `unidecode_rs::unidecode("déjà") -> "deja"`
- Python usage: build extension with `maturin develop --features python`
- Idempotence: `unidecode(unidecode(x)) == unidecode(x)` (after first pass everything is ASCII)
- Golden tests: ensure exact parity with Python

## Rust example

```rust
use unidecode_rs::unidecode;

fn main() {
	println!("{}", unidecode("PŘÍLIŠ ŽLUŤOUČKÝ KŮŇ")); // PRILIS ZLUTOUCKY KUN
}
```

## Install / build (Rust only)

```bash
cargo add unidecode-rs
# or add manually in Cargo.toml then
cargo build
```

## Build the Python extension (development)

Prerequisites: Rust stable, Python ≥3.8, `pip`.

```bash
python -m venv .venv
source .venv/bin/activate
pip install --upgrade pip maturin
maturin develop --release --features python
python -c "import unidecode_rs; print(unidecode_rs.unidecode('déjà vu'))"
```

To build a distributable wheel:

```bash
maturin build --release --features python -o dist/
# Wheels are placed in dist/ directory
pip install dist/unidecode_pyo3-*.whl
```

Or install from PyPI:

```bash
pip install unidecode-pyo3
```

## Python API

```python
import unidecode_rs
print(unidecode_rs.unidecode("Příliš žluťoučký kůň"))
```

Minimal API: single function `unidecode(text: str, errors: Optional[str] = None, replace_str: Optional[str] = None) -> str`.

## Idempotence - what is it?

A function is idempotent if applying it multiple times yields the same result as applying it once. Here:

```
unidecode(unidecode(s)) == unidecode(s)
```

After the first transliteration the output is pure ASCII; a second pass does nothing. A dedicated test validates this over multi‑script samples.

## Golden tests (Python parity)

`golden_equivalence` tests run the Python `Unidecode` library in a subprocess and diff outputs across samples (Latin + accents, Cyrillic, Greek, CJK, emoji). Any mismatch fails the test.

Targeted run:

```bash
cargo test -- --nocapture golden_equivalence
```

## Coverage & critical paths

Dispatch design:

- Presence bitmap per 256‑codepoint block (`BLOCK_BITMAPS`) for quick negative checks.
- Large generated `match` providing PHF table access per block.

Extra tests (`lookup_paths.rs` + internal tests in `lib.rs`) exercise:

- Bit zero ⇒ `lookup` returns `None` (negative path)
- Bit one ⇒ `lookup` returns non‑empty string
- Out‑of‑range block ⇒ early exit
- ASCII parity / idempotence

Generate local report via `cargo llvm-cov` (alias if configured). Detailed guidance moved to `docs/COVERAGE.md`.

```bash
cargo llvm-cov --html
# Or use the provided script:
./scripts/coverage.sh
```

## Upstream test harness

Beyond Rust & golden tests, a Python harness reuses the **original upstream** `Unidecode` test suite to assert behavioral parity.

Main file: `tests/python/test_reference_suite.py`

Characteristics:

- Dynamically loads the upstream base test class (via `_reference/upstream_loader.py`).
- Monkeypatches `unidecode.unidecode` to point to the Rust implementation (`unidecode_rs.unidecode`).
- Implements full `errors=` modes (`ignore`, `replace`, `strict`, `preserve`) for parity.
- Overrides surrogate tests with lean variants to avoid warning noise while maintaining assertions.

Run only this suite:

```bash
pytest -q tests/python/test_reference_suite.py
```

Expected (evolving) report:

```
14 passed, 2 xfailed, 4 xpassed  # exemple actuel
```

`xfail` / `xpass` policy:

- Temporary `xfail` removed once feature implemented; a former `xfail` that passes becomes a normal pass.

Parity roadmap:

1. (Done) Implement `errors=` modes.
2. Finalize surrogate handling parity (optional warning replication toggle).
3. Extend tables to cover remaining mathematical alphanumeric symbols not yet mapped (e.g., script variants currently partial).
4. Add multi‑corpus benchmarks (Latin, mixed CJK, emoji) for stable metrics.
5. Provide exhaustive table diff script (block by block) with machine‑readable output.

Current limitations:

- Some mathematical script / stylistic letter ranges may still map to empty until table extension is complete.
- Generated table lines unexecuted in coverage are data-only, low semantic value.

How to contribute:

1. Add a targeted parity test (Rust or Python) reproducing a divergence.
2. Extend the table or adjust logic.
3. Run `pytest tests/python/test_reference_suite.py` and `cargo test`.
4. Update this section if a batch of former gaps is closed.

---

## Performance

A micro performance snapshot in `golden_equivalence.rs::performance_snapshot` runs 5 iterations on mixed‑script text vs Python. Numbers are indicative only; for robust measurement use Criterion benchmarks or the comparison script:

```bash
python scripts/bench_compare.py
```

## Philosophy

1. Fidelity: match Python before adding new rules.
2. Safety: no panics for any valid Unicode scalar value.
3. Performance: avoid unnecessary copies (ASCII fast path, heuristic pre‑allocation).
4. Maintainability: generated code isolated, core logic compact.

## Development / tests

```bash
cargo test
# (optional) fallback feature using deunicode
cargo test --features fallback-deunicode
```

Python tests (after building extension):

```bash
pytest tests/python
```

## License

GPL-3.0-or-later. Tables derived from public data of the Python [Unidecode] project.

## Acknowledgements

- Original Python project [Unidecode]
- Rust & PyO3 community

[Unidecode]: https://pypi.org/project/Unidecode/
