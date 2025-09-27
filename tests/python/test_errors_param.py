import sys
from pathlib import Path

try:  # pragma: no cover
    import pytest  # type: ignore
except Exception:  # pragma: no cover
    pytest = None  # type: ignore


def _ensure_module():
    try:
        import unidecode_rs  # type: ignore  # noqa: F401
        return True
    except Exception:
        debug = Path(__file__).resolve().parents[2] / 'target' / 'debug'
        if debug.is_dir():
            sys.path.insert(0, str(debug))
            try:
                import unidecode_rs  # type: ignore  # noqa: F401
                return True
            except Exception:
                return False
        return False


def test_errors_param_not_implemented():
    if pytest is None:
        raise RuntimeError("pytest required for this test")
    if not _ensure_module():  # pragma: no cover - environment guard
        pytest.skip("unidecode_rs extension not built")
    import unidecode_rs  # type: ignore
    with pytest.raises(NotImplementedError):
        unidecode_rs.unidecode("é", errors="strict")


def test_errors_param_none_ok():
    if pytest is None:
        raise RuntimeError("pytest required for this test")
    if not _ensure_module():  # pragma: no cover
        pytest.skip("unidecode_rs extension not built")
    import unidecode_rs  # type: ignore
    # Should behave like normal transliteration when errors=None
    assert unidecode_rs.unidecode("é", errors=None) == "e"
