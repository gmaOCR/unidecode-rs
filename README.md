# unidecode-rs

Rust implementation of the Unidecode transliteration logic with optional
PyO3 bindings to expose a drop-in replacement for the Python `unidecode`
package.

This repository contains:

- `src/` — Rust implementation and PyO3 bindings (optional feature `python`).
- `python/` — a small Python shim that provides upstream-compatible
  signatures and forwards to the compiled extension when available.
- `tests/` — Rust unit tests and a parity harness for upstream Python tests.
- `bench/` — benchmark helpers comparing pure-Python `unidecode` vs the
  compiled `unidecode-rs` extension.

## Quickstart — Rust library usage

Add `unidecode-rs` as a dependency in your `Cargo.toml` (example):

```toml
[dependencies]
unidecode-rs = { git = "https://github.com/gmaOCR/unidecode-rs", tag = "v0.0.1" }
```

Then call the API from Rust:

```rust
use unidecode_rs::slugify; // example public function in this repo

let out = unidecode_rs::unidecode("Héllo Wörld — café");
println!("{}", out);
```

See `src/lib.rs` and `src/lib_py.rs` for additional exported functions.

## Quickstart — Python users (drop-in replacement)

If you want to replace the pure-Python `unidecode` package with the
Rust-backed implementation (faster), follow these steps.

1. Build and install the Python wheel using `maturin` (local develop):

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip maturin
cd unidecode-rs
maturin develop --release --features python
```

This will build the compiled extension and install a small Python package
that exposes the same API surface as upstream `unidecode`.

2. Replace imports in your Python code

If your code does `from unidecode import unidecode`, the recommended way is
to install `unidecode-rs` into the same environment (see above). The
repository also contains a small shim at `unidecode-rs/python/unidecode_rs`
which ensures the exported callables use the same parameter names and raise
the same exception types as upstream.

3. Compatibility notes

- The shim aims to provide identical function signatures and semantics to
  the upstream `unidecode` including `errors` handling and surrogate
  behavior. Where upstream behavior depends on narrow/broad Py builds we
  mirror the upstream tests by warning and stripping surrogates.
- If you need `inspect.signature` compatibility, the shim exposes the
  textual signature `(string, errors=None, replace_str=None)` so tooling
  that introspects signatures will work as expected.

## Benchmarks

See `bench/bench_unidecode_compare.py` — it compares call latency and
throughput for representative inputs. During development the Rust
implementation showed sizable speedups (multi‑x) vs the pure Python
implementation for large inputs.

## Publishing to PyPI (OIDC)

This repository includes a GitHub Actions workflow to publish manylinux
wheels to PyPI using OIDC token minting (no long-lived PyPI token in the
repo). See `.github/workflows/publish-pypi.yml` for implementation. To
publish:

1. Tag a release on GitHub (e.g. `v1.2.3`) and push the tag.
2. The workflow builds manylinux wheels using `maturin` and exchanges an
   OIDC token for a short-lived PyPI API token (mint). The workflow then
   uploads dists to PyPI.

Note: see the workflow file for details and required runner permissions.

## Development notes

- Use `cargo test` for Rust unit tests.
- Use `maturin develop --release --features python` to iterate on Python
  bindings and local tests.
- The repo contains a parity harness that runs the upstream `unidecode`
  Python tests against this compiled extension to track functional parity.

## License

Distributed under the project license (see `LICENSE`).
# unidecode-rs — Unicode → ASCII transliteration faithful to Python

[![CI](https://github.com/gmaOCR/unidecode-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/gmaOCR/unidecode-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/unidecode-rs.svg)](https://crates.io/crates/unidecode-rs)
[![Docs](https://docs.rs/unidecode-rs/badge.svg)](https://docs.rs/unidecode-rs)
[![Coverage](https://codecov.io/gh/gmaOCR/unidecode-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/gmaOCR/unidecode-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Python Wheels](https://img.shields.io/badge/python-wheels-blue)](https://pypi.org/project/Unidecode/) 

Fast Rust implementation (optional Python bindings via PyO3) targeting bit‑for‑bit equivalence with Python [Unidecode]. Provides:

- Same output as `Unidecode` for all covered tables
- Noticeably higher performance (see perf snapshot in tests)
- Golden tests comparing dynamically against the Python version
- High coverage on critical paths (bitmap + per‑block dispatch)

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
maturin build --release --features python -i python
pip install target/wheels/*.whl
```

## Python API

```python
import unidecode_rs
print(unidecode_rs.unidecode("Příliš žluťoučký kůň"))
```

Minimal API: single function `unidecode(text: str, errors: Optional[str] = None, replace_str: Optional[str] = None) -> str`.

## Idempotence — what is it?

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

A micro performance snapshot in `golden_equivalence.rs::performance_snapshot` runs 5 iterations on mixed‑script text vs Python. Numbers are indicative only; for robust measurement use Criterion benchmarks or larger corpora.

## Repository layout

```
src/                # Core library sources + generated tables
benches/            # Criterion or std benches (Rust)
scripts/            # Developer helper scripts (bench_compare, coverage)
tests/              # Rust integration & golden tests
tests/python/       # Python parity & upstream harness
docs/               # Coverage and performance documentation
```

`docs/PERFORMANCE_PLAN.md` details next-step performance ideas.

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

MIT. Tables derived from public data of the Python [Unidecode] project.

## Acknowledgements

- Original Python project [Unidecode]
- Rust & PyO3 community

[Unidecode]: https://pypi.org/project/Unidecode/
