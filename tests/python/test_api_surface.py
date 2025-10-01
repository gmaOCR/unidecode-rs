import importlib
import sys
import subprocess
from typing import Any, cast

# Golden minimal API surface expectations
EXPECTED_FUNCS = {"unidecode"}


def ensure_unidecode_reference():
    try:
        import unidecode  # type: ignore
        return unidecode.unidecode  # type: ignore
    except Exception:
        subprocess.check_call([
            sys.executable,
            '-m',
            'pip',
            'install',
            '-q',
            'Unidecode',
        ])
        import unidecode  # type: ignore
    return unidecode.unidecode  # type: ignore


def test_module_surface():
    mod = importlib.import_module("unidecode_rs")
    names = {n for n in dir(mod) if not n.startswith('_')}
    # Check only for missing expected functions
    missing = EXPECTED_FUNCS - names
    assert not missing, f"Missing functions: {missing} in {names}"


def test_function_parity_basic():
    ref_unidecode = ensure_unidecode_reference()
    mod = importlib.import_module("unidecode_rs")
    samples = [
        "déjà vu",
        "Русский текст",
        "中文字符測試",
        "I ♥ 🚀",
    ]
    for s in samples:
        rs_any: Any = mod.unidecode(s)
        py_any: Any = ref_unidecode(s)
        # Type narrowing for static analyzers
        rs = cast(str, rs_any)
        py_ref = cast(str, py_any)
        assert isinstance(rs, str) and isinstance(py_ref, str)
        assert rs == py_ref, f"Mismatch for {s!r}: rs={rs!r} ref={py_ref!r}"


def test_signature_simple():
    mod = importlib.import_module("unidecode_rs")
    sig = getattr(mod.unidecode, "__text_signature__", None)
    # Accept None (Python function) or full signature from PyO3 builtin
    # The actual signature is verified by test_reference_suite.py
    assert sig is None or "(string" in sig or sig in {"(text)", "(input)"}, \
        f"Unexpected signature: {sig}"
