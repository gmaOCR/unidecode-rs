"""Build the extension then compare Python Unidecode vs Rust extension.

This script is a quick-and-dirty benchmark harness. It will install
maturin into the current Python, build the extension in "develop" mode and
compare throughput for a large sample.
"""

import sys
import time
import subprocess
from pathlib import Path


def prepare_sample(path: Path) -> str:
    if not path.exists():
        text = "C'est déjà l'été! Привет мир! こんにちは 世界 🌍\n"
        text = text * 10000
        path.write_text(text, encoding="utf8")
    return path.read_text(encoding="utf8")


def main() -> None:
    sample = Path(__file__).with_name("sample.txt")
    text = prepare_sample(sample)

    print("installing build tools...")
    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", "--upgrade", "pip", "maturin"]
    )
    subprocess.check_call(
        [
            "maturin",
            "develop",
            "--release",
            "--features",
            "python",
        ]
    )

    print("importing modules...")
    import unidecode as py_unidecode

    rust_fn = None
    try:
        import unidecode_rs  # type: ignore

        rust_fn = unidecode_rs.unidecode
    except Exception as exc:
        print("rust extension import failed:", exc)

    # warmup
    print("warming up...")
    py_unidecode.unidecode(text[:1000])
    if rust_fn:
        rust_fn(text[:1000])

    # benchmark python unidecode
    print("benchmarking python unidecode...")
    t0 = time.perf_counter()
    py_out = py_unidecode.unidecode(text)
    t1 = time.perf_counter()
    print("python time:", t1 - t0)

    # benchmark rust if available
    if rust_fn:
        print("benchmarking rust unidecode...")
        t0 = time.perf_counter()
        rust_out = rust_fn(text)
        t1 = time.perf_counter()
        print("rust time:", t1 - t0)

        # compare outputs (first 200 chars)
        print("outputs equal (prefix):", py_out[:200] == rust_out[:200])


if __name__ == "__main__":
    main()
