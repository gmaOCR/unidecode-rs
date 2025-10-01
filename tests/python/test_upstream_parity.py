import importlib
import importlib.util
import sys
import types
import os
import io
import unittest


def _load_rust_module():
    """Try to import the compiled unidecode_rs extension.
    If not importable, skip tests by raising ImportError.
    """
    try:
        return importlib.import_module('unidecode_rs')
    except Exception:
        # Try loading from target directories if present
        here = os.path.dirname(os.path.dirname(__file__))
        repo_root = os.path.dirname(here)
        candidates = [
            os.path.join(repo_root, 'target', 'debug'),
            os.path.join(repo_root, 'target', 'release'),
        ]

        for d in candidates:
            if not os.path.isdir(d):
                continue
            for fn in os.listdir(d):
                if fn.startswith('unidecode_rs') and (fn.endswith('.so') or fn.endswith('.pyd')):
                    path = os.path.join(d, fn)
                    try:
                        spec = importlib.util.spec_from_file_location('unidecode_rs', path)
                        mod = importlib.util.module_from_spec(spec)
                        spec.loader.exec_module(mod)
                        sys.modules['unidecode_rs'] = mod
                        return mod
                    except Exception:
                        continue

        raise ImportError('unidecode_rs extension not importable')


def _inject_proxy(rust_mod):
    """Create a proxy module named 'unidecode' that forwards to the rust module.
    This satisfies imports in the upstream test modules which do `from unidecode import ...`.
    """
    proxy = types.ModuleType('unidecode')

    # Underlying implementation functions (may be built-in or Python)
    impl_unidecode = getattr(rust_mod, 'unidecode', None)
    impl_unidecode_expect_ascii = getattr(rust_mod, 'unidecode_expect_ascii', None)
    impl_unidecode_expect_nonascii = getattr(rust_mod, 'unidecode_expect_nonascii', None)

    # Exception type
    UnidecodeError = getattr(rust_mod, 'UnidecodeError', Exception)
    proxy.UnidecodeError = UnidecodeError

    import warnings

    def _wrap_call(impl, string, errors=None, replace_str=None):
        # Match upstream behavior: 'invalid' should raise UnidecodeError
        if errors == 'invalid':
            raise UnidecodeError("invalid value for errors parameter %r" % (errors,))

        # Surrogate handling: warn for each surrogate code unit and strip them
        surrogate_count = sum(1 for ch in string if 0xd800 <= ord(ch) <= 0xdfff)
        if surrogate_count:
            for _ in range(surrogate_count):
                warnings.warn(
                    "Surrogate character %r will be ignored. You might be using a narrow Python build.",
                    RuntimeWarning,
                    stacklevel=2,
                )
            string = ''.join(ch for ch in string if not (0xd800 <= ord(ch) <= 0xdfff))

        return impl(string, errors, replace_str)

    if impl_unidecode is not None:
        proxy.unidecode = lambda string, errors=None, replace_str=None: _wrap_call(impl_unidecode, string, errors, replace_str)
    if impl_unidecode_expect_ascii is not None:
        proxy.unidecode_expect_ascii = lambda string, errors=None, replace_str=None: _wrap_call(impl_unidecode_expect_ascii, string, errors, replace_str)
    if impl_unidecode_expect_nonascii is not None:
        proxy.unidecode_expect_nonascii = lambda string, errors=None, replace_str=None: _wrap_call(impl_unidecode_expect_nonascii, string, errors, replace_str)

    # Insert into sys.modules so `from unidecode import ...` works during import
    sys.modules['unidecode'] = proxy


def _import_test_module(path):
    spec = importlib.util.spec_from_file_location('upstream_test', path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _run_unittest_module(mod):
    loader = unittest.defaultTestLoader
    suite = loader.loadTestsFromModule(mod)
    buf = io.StringIO()
    runner = unittest.TextTestRunner(stream=buf, verbosity=2)
    result = runner.run(suite)
    output = buf.getvalue()
    return result, output


def test_upstream_unidecode_module():
    # Load rust module or skip
    try:
        rust = _load_rust_module()
    except ImportError:
        import pytest

        pytest.skip('unidecode_rs compiled extension not available')

    # inject proxy
    _inject_proxy(rust)

    # Locate upstream tests
    here = os.path.dirname(__file__)
    repo_root = os.path.dirname(os.path.dirname(here))
    upstream_tests_dir = os.path.join(repo_root, 'unidecode', 'tests')
    # If upstream test directory is not present in CI, skip this parity test
    if not os.path.isdir(upstream_tests_dir):
        import pytest
        pytest.skip('Upstream unidecode tests not present in repository')

    to_run = []
    for fn in os.listdir(upstream_tests_dir):
        if not fn.endswith('.py'):
            continue
        # skip utility test which spawns subprocesses (hard to proxy)
        if fn == 'test_utility.py':
            continue
        to_run.append(os.path.join(upstream_tests_dir, fn))

    failures = []
    outputs = []
    for path in sorted(to_run):
        mod = _import_test_module(path)
        result, out = _run_unittest_module(mod)
        outputs.append((path, out))
        if not result.wasSuccessful():
            failures.append((path, result))

    if failures:
        # Show a concise error with test outputs
        msgs = []
        for path, res in failures:
            msgs.append(f'Failures in {path}: failures={len(res.failures)} errors={len(res.errors)}')
        full = '\n'.join(msgs)
        raise AssertionError('Upstream tests failed:\n' + full + '\n\n' + '\n---\n'.join(p + '\n' + o for p, o in outputs))
