"""Parity tests between the upstream Python `unidecode` and the
Rust-backed `unidecode_rs` extension.

These tests are intentionally small and robust. They verify that the
same callables exist and that outputs match on a handful of representative
inputs. If the compiled extension is not available or not importable, the
tests are skipped rather than failing the suite.
"""

from __future__ import annotations

import importlib
import importlib.machinery
import importlib.util
import inspect
import sys
from pathlib import Path

import pytest


def _maybe_import_rust():
	"""Return the `unidecode_rs` module or skip tests if not available.

	Try a normal import first. If that fails, search the local
	`target/{debug,release}` directories for a compiled artifact and
	attempt to load it by path. If still not found, skip the tests.
	"""
	try:
		return importlib.import_module("unidecode_rs")
	except Exception:
		root = Path(__file__).resolve().parents[2]
		for sub in ("debug", "release"):
			d = root / "target" / sub
			if not d.is_dir():
				continue
			patterns = (
				"*unidecode_rs*.so",
				"*unidecode_rs*.pyd",
				"*unidecode_rs*.dll",
				"*unidecode_rs*.dylib",
			)
			for pat in patterns:
				for p in d.glob(pat):
					try:
						loader = importlib.machinery.ExtensionFileLoader(
							"unidecode_rs", str(p)
						)
						spec = importlib.util.spec_from_loader(loader.name, loader)
						if spec is None:
							continue
						module = importlib.util.module_from_spec(spec)
						loader.exec_module(module)
						sys.modules["unidecode_rs"] = module
						return module
					except Exception:
						# Try next candidate
						continue
		pytest.skip("compiled unidecode_rs not available")


def test_api_surface_matches_upstream():
	import unidecode as py_unidecode

	rust = _maybe_import_rust()

	expected = (
		"unidecode",
		"unidecode_expect_ascii",
		"unidecode_expect_nonascii",
	)

	for name in expected:
		assert hasattr(py_unidecode, name), f"upstream missing {name}"
		assert hasattr(rust, name), f"rust extension missing {name}"

		py_obj = getattr(py_unidecode, name)
		rs_obj = getattr(rust, name)

		assert callable(py_obj)
		assert callable(rs_obj)

		try:
			py_sig = inspect.signature(py_obj)
			rs_sig = inspect.signature(rs_obj)
			py_params = [p.name for p in py_sig.parameters.values()]
			rs_params = [p.name for p in rs_sig.parameters.values()]
			assert py_params == rs_params
		except (ValueError, TypeError):
			# extension signatures may be opaque; skip strict check
			pass


@pytest.mark.parametrize(
	"s",
	[
		"",
		"ASCII only",
		"déjà vu",
		"𝔘𝔫𝔦𝔠𝔬𝔡𝔢",
		"I ♥ 🚀",
		"PŘÍLIŠ ŽLUŤOUČKÝ KŮŇ",
	],
)
def test_outputs_match_for_representative_inputs(s: str) -> None:
	import unidecode as py_unidecode

	rust = _maybe_import_rust()

	assert py_unidecode.unidecode(s) == rust.unidecode(s)
	assert (
		py_unidecode.unidecode_expect_ascii(s)
		== rust.unidecode_expect_ascii(s)
	)
	assert (
		py_unidecode.unidecode_expect_nonascii(s)
		== rust.unidecode_expect_nonascii(s)
	)

