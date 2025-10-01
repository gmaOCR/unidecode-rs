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

# Import the compiled extension implementation from the compiled module.
# When installed via maturin, the compiled extension is available as
# unidecode_rs.unidecode_rs (the compiled .so file).
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
# (no further action needed; variables are set above)

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

    # Handle surrogate code units on narrow builds: warn and remove them.
    surrogate_count = sum(1 for ch in string if 0xd800 <= ord(ch) <= 0xdfff)
    if surrogate_count:
        import warnings
        for _ in range(surrogate_count):
            warnings.warn(
                "Surrogate character %r will be ignored. "
                "You might be using a narrow Python build.",
                RuntimeWarning,
                stacklevel=2,
            )
        string = ''.join(ch for ch in string if not (0xd800 <= ord(ch) <= 0xdfff))

    try:
        return _unidecode_impl(string, errors, replace_str)
    except UnidecodeError:
        # If the Rust impl raises UnidecodeError for 'invalid' mode,
        # we need to catch it and return the original string (preserve behavior)
        if errors in ('invalid', 'preserve'):
            return string
        raise


# Note: We don't set __text_signature__ on the Python wrapper because
# inspect.signature() correctly introspects Python function signatures.
# If needed for C extension compatibility, copy from _unidecode_impl.


def unidecode_expect_ascii(
    string: str,
    errors: Optional[str] = None,
    replace_str: Optional[str] = None,
) -> str:
    """Alias matching upstream: unidecode_expect_ascii(
    string, errors, replace_str)
    """
    assert _unidecode_expect_ascii_impl is not None
    surrogate_count = sum(1 for ch in string if 0xd800 <= ord(ch) <= 0xdfff)
    if surrogate_count:
        import warnings
        for _ in range(surrogate_count):
            warnings.warn(
                "Surrogate character %r will be ignored. You might be using a narrow Python build.",
                RuntimeWarning,
                stacklevel=2,
            )
        string = ''.join(ch for ch in string if not (0xd800 <= ord(ch) <= 0xdfff))
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
    surrogate_count = sum(1 for ch in string if 0xd800 <= ord(ch) <= 0xdfff)
    if surrogate_count:
        import warnings
        for _ in range(surrogate_count):
            warnings.warn(
                "Surrogate character %r will be ignored. You might be using a narrow Python build.",
                RuntimeWarning,
                stacklevel=2,
            )
        string = ''.join(ch for ch in string if not (0xd800 <= ord(ch) <= 0xdfff))
    return _unidecode_expect_nonascii_impl(string, errors, replace_str)
