"""Dynamic upstream test loader for original Unidecode test suite.

This module fetches the upstream ``tests/test_unidecode.py`` file from the
official Python Unidecode repository (avian2/Unidecode) at test runtime and
exposes ``BaseTestUnidecode`` so we can reuse its methods against the Rust
implementation.

Rationale:
* We intentionally do not vendor the full upstream Python package to avoid
  copying large tables and GPLv2+ code (the project here focuses on a Rust
  re‑implementation). Only the test logic is transiently fetched.
* The upstream test file is GPLv2+; fetching and executing it at test time
  keeps derivative content out of the committed source while still giving us
  behavioral parity signals. If long‑term distribution of a vendored copy is
  desired, ensure overall license compatibility first.

Environment / network:
* If the network fetch fails (offline CI) we fall back to a cached local copy
  under ``_reference/cache_test_unidecode.py`` when present.
* If neither is available we raise ``RuntimeError`` so the harness can decide
  to skip or xfail gracefully.

Security considerations:
* We fetch a known trusted file from the main branch of the upstream repo.
* The content is executed with a constrained globals dict providing only the
  symbols required by the upstream file. Review this file periodically.

NOTE: If reproducibility without network is required, run the helper script:
    python -m tests.python._reference.upstream_loader --cache
which will download and store the file locally so subsequent test runs are
offline.
"""

from __future__ import annotations

import sys
import types
import urllib.request
from pathlib import Path
from typing import Any, Dict

UPSTREAM_RAW_URL = (
    "https://raw.githubusercontent.com/avian2/Unidecode/master/tests/"
    "test_unidecode.py"
)

CACHE_FILE = Path(__file__).with_name("cache_test_unidecode.py")


def _fetch_upstream() -> str:
    try:
        with urllib.request.urlopen(  # nosec B310
            UPSTREAM_RAW_URL, timeout=15
        ) as resp:
            raw = resp.read()
        # The file declares utf-8; enforce decode.
        return raw.decode("utf-8", errors="strict")
    except Exception as e:  # pragma: no cover - network variability
        if CACHE_FILE.is_file():
            return CACHE_FILE.read_text(encoding="utf-8")
        raise RuntimeError(f"Unable to fetch upstream test file: {e}") from e


def _ensure_stub_unidecode_module() -> None:
    """Install a lightweight stub 'unidecode' module if missing.

    Provides: unidecode (Rust binding), placeholders for unsupported APIs
    so that tests referencing them fail in a controlled manner later.
    """
    if "unidecode" in sys.modules:
        return
    try:
        import unidecode_rs  # noqa: F401
    except Exception as e:  # pragma: no cover
        raise RuntimeError(f"unidecode_rs module not available: {e}") from e

    import unidecode_rs as unidecode_rs_mod

    stub = types.ModuleType("unidecode")

    class UnidecodeError(Exception):
        pass

    def unidecode(text: str, *_, **__) -> str:  # type: ignore
        return unidecode_rs_mod.unidecode(text)

    def _unsupported(*_a, **_k):  # pragma: no cover - executed only if called
        raise NotImplementedError(
            "Feature not implemented in Rust binding yet"
        )

    for name, obj in (
        ("unidecode", unidecode),
        ("unidecode_expect_ascii", _unsupported),
        ("unidecode_expect_nonascii", _unsupported),
        ("UnidecodeError", UnidecodeError),
    ):
        setattr(stub, name, obj)
    sys.modules["unidecode"] = stub


def load_base_test_class() -> type:
    """Return upstream ``BaseTestUnidecode`` class.

    Downloads (or loads cached) upstream test file and executes it in an
    isolated namespace exposing the stub unidecode module.
    """
    _ensure_stub_unidecode_module()
    source = _fetch_upstream()
    
    class _WarningLogger(list):  # minimal stand-in
        def start(self, *_, **__):  # noqa: D401
            return self

        def stop(self, *_, **__):  # noqa: D401
            return self

        # Provide list attribute 'log' matching upstream expectations.
        def __init__(self, *a, **k):  # noqa: D401
            super().__init__(*a, **k)
            self.log = []  # type: ignore[attr-defined]

        # Compatibility helper (not used by upstream now) to append warnings
        def add(self, *record):  # noqa: D401
            self.log.append(record)

        # Graceful context manager usage
        def __enter__(self):  # noqa: D401
            return self

        def __exit__(self, exc_type, exc, tb):  # noqa: D401
            return False

    g: Dict[str, Any] = {
        "__name__": "_upstream_unidecode_tests",
        "__file__": "test_unidecode.py",
        "sys": sys,
        "unittest": __import__("unittest"),
        "warnings": __import__("warnings"),
        # Provide placeholder so class bodies referencing 'unidecode'
        # do not fail during execution of the upstream test file.
        "unidecode": (lambda x, *a, **k: x),  # type: ignore[arg-type]
        "unidecode_expect_ascii": (lambda x, *a, **k: x),  # placeholder
        "unidecode_expect_nonascii": (lambda x, *a, **k: x),  # placeholder
        # Capture warnings context placeholder used by upstream tests.
        "WarningLogger": _WarningLogger,
    }
    l: Dict[str, Any] = {}
    code = compile(source, "test_unidecode.py", "exec")
    exec(code, g, l)  # noqa: S102
    base = l.get("BaseTestUnidecode") or g.get("BaseTestUnidecode")
    if not isinstance(base, type):  # pragma: no cover
        raise RuntimeError("BaseTestUnidecode not found in upstream test file")
    return base


def _cmd_cache() -> int:
    try:
        text = _fetch_upstream()
    except RuntimeError as e:
        print(f"Download failed: {e}", file=sys.stderr)
        return 1
    CACHE_FILE.write_text(text, encoding="utf-8")
    print(f"Cached upstream test file to {CACHE_FILE}")
    return 0


if __name__ == "__main__":  # pragma: no cover - manual usage helper
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--cache", action="store_true", help="Download and cache upstream file"
    )
    args = parser.parse_args()
    if args.cache:
        raise SystemExit(_cmd_cache())
    # Otherwise just attempt load to verify
    cls = load_base_test_class()
    print(f"Loaded upstream class: {cls.__name__}")
