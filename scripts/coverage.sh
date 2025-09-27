#!/usr/bin/env bash
set -euo pipefail

# Unified coverage script (Rust + optional Python) for unidecode-rs
# Mirrors slugify-rs approach for consistency.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/5] Rust lcov (instrumented tests)"
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  rustup component add llvm-tools-preview >/dev/null 2>&1 || true
  cargo install cargo-llvm-cov --quiet
fi
cargo coverage-lcov >/dev/null

RUST_LCOV="lcov.info"
if [[ -f "$RUST_LCOV" ]]; then
  echo "Rust lcov generated: $RUST_LCOV"; grep -E '^SF:' "$RUST_LCOV" | wc -l | xargs echo "  source files tracked:";
else
  echo "Rust lcov missing" >&2; exit 1
fi

echo "[2/5] Python environment setup (optional if bindings not needed)"
if [[ ! -d .venv ]]; then
  python -m venv .venv
fi
# shellcheck disable=SC1091
source .venv/bin/activate
pip install --disable-pip-version-check -q --upgrade pip maturin pytest pytest-cov coverage

echo "[3/5] Build PyO3 extension"
maturin develop --release --features python >/dev/null

echo "[4/5] Run Python tests + coverage"
pytest tests/python --cov=. --cov-report=xml:coverage.xml --maxfail=1 -q || echo "(No python tests or failures)"

if [[ -f coverage.xml ]]; then
  echo "Python coverage.xml generated"
else
  echo "No coverage.xml produced (no tests?)"
fi

echo "[5/5] Summary"
if command -v coverage >/dev/null 2>&1; then
  coverage report -m || true
fi

echo "Done. Artifacts: lcov.info (Rust), coverage.xml (Python)";
