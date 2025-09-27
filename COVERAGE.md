# Coverage guidance for unidecode-rs

This crate exposes both a Rust API and optional Python bindings via PyO3.
We collect two kinds of coverage:

1. Rust line coverage (cargo-llvm-cov -> lcov.info)
2. Python test coverage (pytest + pytest-cov -> coverage.xml)

## Rust coverage

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov # once
# Run instrumentation on tests
cargo llvm-cov --lcov --output-path lcov.info
```

## Python coverage

```bash
python -m venv .venv
. .venv/bin/activate
pip install --upgrade pip maturin pytest pytest-cov coverage
maturin develop --release --features python
pytest --cov=. --cov-report=xml:coverage.xml --cov-report=term-missing
```

Artifacts:
- `lcov.info`  -> upload with Codecov flag `rust`
- `coverage.xml` -> upload with Codecov flag `python`

## Combined view
Codecov will merge both uploads (flags differentiate sources). Configure the
`CODECOV_TOKEN` secret in the repository for private repos; public repos can
omit the token.

## Adding new tests
- Rust: prefer small deterministic unit tests in `tests/` or `#[cfg(test)]` modules.
- Python: add integration / golden tests exercising edge code points.

## Next optimization stages
Future performance improvements (bitset skip, dense block arrays) should be
covered by adding benches plus golden test assertions to guard correctness.
