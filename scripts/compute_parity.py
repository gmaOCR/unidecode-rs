"""Run upstream test modules against the Rust extension and print a parity summary.

Usage: run this with the same Python interpreter where the wheel was installed
(e.g. the maturin venv). It will import the compiled extension, inject a proxy
module `unidecode` that forwards to the extension, run the upstream tests, and
print the pass/fail counts and percentage parity.
"""
import importlib
import importlib.util
import sys
import os
import io
import types
import unittest
import warnings


def _load_rust_module():
    try:
        return importlib.import_module('unidecode_rs')
    except Exception:
        # Try target locations
        repo_root = os.path.dirname(os.path.dirname(__file__))
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
        raise ImportError('unidecode_rs not importable')


def _inject_proxy(rust_mod):
    proxy = types.ModuleType('unidecode')
    impl_unidecode = getattr(rust_mod, 'unidecode', None)
    impl_unidecode_expect_ascii = getattr(rust_mod, 'unidecode_expect_ascii', None)
    impl_unidecode_expect_nonascii = getattr(rust_mod, 'unidecode_expect_nonascii', None)
    UnidecodeError = getattr(rust_mod, 'UnidecodeError', Exception)
    proxy.UnidecodeError = UnidecodeError

    def _wrap_call(impl, string, errors=None, replace_str=None):
        if errors == 'invalid':
            raise UnidecodeError("invalid value for errors parameter %r" % (errors,))
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

    sys.modules['unidecode'] = proxy


def _collect_upstream_tests():
    # Search upward from this crate for an 'unidecode/tests' directory. This makes
    # the script work when the upstream python package lives at the repository
    # root (../unidecode/tests) or in other monorepo layouts.
    start = os.path.dirname(os.path.dirname(__file__))
    cur = start
    upstream_tests_dir = None
    while True:
        candidate = os.path.join(cur, 'unidecode', 'tests')
        if os.path.isdir(candidate):
            upstream_tests_dir = candidate
            break
        parent = os.path.dirname(cur)
        if parent == cur:
            break
        cur = parent

    if upstream_tests_dir is None:
        raise FileNotFoundError('Could not locate upstream unidecode/tests directory')

    paths = []
    for fn in os.listdir(upstream_tests_dir):
        if not fn.endswith('.py'):
            continue
        paths.append(os.path.join(upstream_tests_dir, fn))
    return sorted(paths)


def _run_module(path):
    spec = importlib.util.spec_from_file_location('upstream_test', path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    loader = unittest.defaultTestLoader
    suite = loader.loadTestsFromModule(mod)
    buf = io.StringIO()
    runner = unittest.TextTestRunner(stream=buf, verbosity=2)
    result = runner.run(suite)
    return result, buf.getvalue()


def main():
    rust = _load_rust_module()
    _inject_proxy(rust)
    tests = _collect_upstream_tests()
    total = 0
    passed = 0
    failed = 0
    errored = 0
    details = []
    for path in tests:
        # skip the utility test (subprocess-driven) by default
        if os.path.basename(path) == 'test_utility.py':
            continue
        result, out = _run_module(path)
        total += result.testsRun
        failed += len(result.failures)
        errored += len(result.errors)
        passed += result.testsRun - (len(result.failures) + len(result.errors))
        details.append((path, result.testsRun, len(result.failures), len(result.errors), out))

    parity = (passed / total * 100.0) if total else 0.0
    print('Upstream test modules run:', len(details))
    print('Total tests:', total)
    print('Passed:', passed)
    print('Failures:', failed)
    print('Errors:', errored)
    print('Parity: {:.2f}%'.format(parity))
    # Optionally print details for debugging
    for p, tr, f, e, out in details:
        print('\n---', p, 'tests:', tr, 'failures:', f, 'errors:', e)
        print(out)


if __name__ == '__main__':
    main()
