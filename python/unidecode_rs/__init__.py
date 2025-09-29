"""Pure-Python shim to expose upstream-compatible signatures.

The compiled extension provides the core implementation as
``unidecode_rs.unidecode_rs`` (extension module). Some Python tools
inspect.signature() more reliably when functions are implemented in
Python. This shim defines small wrappers with the exact parameter names
used by the upstream ``unidecode`` package and forwards calls to the
compiled functions.
"""
from __future__ import annotations

from typing import Optional

# Import the compiled extension implementation. When installed by maturin the
# package layout includes a compiled module `unidecode_rs.unidecode_rs` which
# exports the core functions; the shim forwards to those.
try:
    from .unidecode_rs import (
        unidecode as _unidecode_impl,
        unidecode_expect_ascii as _unidecode_expect_ascii_impl,
        unidecode_expect_nonascii as _unidecode_expect_nonascii_impl,
        UnidecodeError,
    )
except Exception:  # pragma: no cover - compiled extension may be absent
    # Allow import-time failure when the compiled extension isn't present; the
    # test harness will skip in that case.
    _unidecode_impl = None  # type: ignore
    _unidecode_expect_ascii_impl = None  # type: ignore
    _unidecode_expect_nonascii_impl = None  # type: ignore
    UnidecodeError = Exception  # type: ignore

__all__ = [
    "unidecode",
    "unidecode_expect_ascii",
    "unidecode_expect_nonascii",
    "UnidecodeError",
]


def unidecode(
    string: str,
    errors: Optional[str] = None,
    replace_str: Optional[str] = None,
) -> str:
    """Transliterate text to ASCII.

    Signature matches upstream: (string, errors=None, replace_str=None)
    """
    assert _unidecode_impl is not None
    return _unidecode_impl(string, errors, replace_str)


def unidecode_expect_ascii(
    string: str,
    errors: Optional[str] = None,
    replace_str: Optional[str] = None,
) -> str:
    """Alias matching upstream: unidecode_expect_ascii(
    string, errors, replace_str)
    """
    assert _unidecode_expect_ascii_impl is not None
    return _unidecode_expect_ascii_impl(string, errors, replace_str)


def unidecode_expect_nonascii(
    string: str,
    errors: Optional[str] = None,
    replace_str: Optional[str] = None,
) -> str:
    """Alias matching upstream: unidecode_expect_nonascii(
    string, errors, replace_str)
    """
    assert _unidecode_expect_nonascii_impl is not None
    return _unidecode_expect_nonascii_impl(string, errors, replace_str)
