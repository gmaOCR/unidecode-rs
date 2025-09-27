import sys
from pathlib import Path

pytest = None  # Lightweight standalone tests (no dependency)


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


def test_errors_default_and_none():
    if not _ensure_module():  # pragma: no cover - environment guard
        return  # environment without built module: treat as noop
    import unidecode_rs  # type: ignore
    assert unidecode_rs.unidecode("é") == "e"
    assert unidecode_rs.unidecode("é", errors=None) == "e"


def test_errors_ignore():
    if not _ensure_module():  # pragma: no cover
        return
    import unidecode_rs  # type: ignore
    # Emoji removed in ignore mode
    assert unidecode_rs.unidecode("😀", errors="ignore") == ""


def test_errors_replace():
    if not _ensure_module():  # pragma: no cover
        return
    import unidecode_rs  # type: ignore
    assert unidecode_rs.unidecode("😀", errors="replace") == "?"
    assert unidecode_rs.unidecode(
        "😀", errors="replace", replace_str="[x]"
    ) == "[x]"


def test_errors_preserve_and_invalid():
    if not _ensure_module():  # pragma: no cover
        return
    import unidecode_rs  # type: ignore
    for mode in ("preserve", "invalid"):
        assert unidecode_rs.unidecode("😀", errors=mode) == "😀"


def test_errors_strict():
    if not _ensure_module():  # pragma: no cover
        return
    import unidecode_rs  # type: ignore
    # unmapped first char -> index 0
    try:
        unidecode_rs.unidecode("😀a", errors="strict")
        raise AssertionError(
            "strict mode did not raise for unmapped first char"
        )
    except unidecode_rs.UnidecodeError as exc:  # type: ignore
        assert getattr(exc, "index", None) == 0
    # mapped then unmapped -> index 1
    try:
        unidecode_rs.unidecode("é😀", errors="strict")
        raise AssertionError(
            "strict mode did not raise for second unmapped char"
        )
    except unidecode_rs.UnidecodeError as exc2:  # type: ignore
        assert getattr(exc2, "index", None) == 1


def test_errors_unknown_policy():
    if not _ensure_module():  # pragma: no cover
        return
    import unidecode_rs  # type: ignore
    try:
        unidecode_rs.unidecode("é", errors="does_not_exist")
        raise AssertionError("unknown policy did not raise ValueError")
    except ValueError:
        pass
