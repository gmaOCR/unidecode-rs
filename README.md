# unidecode-rs

Minimal Rust-backed transliteration library for benchmarking vs Python Unidecode.

Build and test (python extension):

```bash
python -m venv .venv
. .venv/bin/activate
pip install --upgrade pip maturin
maturin develop --release --features python
```