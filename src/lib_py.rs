#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::wrap_pyfunction;
#[cfg(feature = "python")]
use pyo3::{create_exception, exceptions::PyException};

// Define custom exception at module level so we can construct it easily.
#[cfg(feature = "python")]
create_exception!(unidecode_rs, UnidecodeError, PyException);

#[cfg(feature = "python")]
// Upstream Python tests expect the C-API text signature to be simple (text)
// so we expose a compact signature to match their checks.
// PyModule is provided by the pyo3 prelude imported above; avoid importing
// the types::PyModule to prevent method-resolution conflicts with the macro
// generated types.
#[pyfunction(signature = (string, errors=None, replace_str=None), text_signature = "(string, errors=None, replace_str=None)")]
/// Transliterates a Unicode string to ASCII (mirror of Python `unidecode.unidecode`).
///
/// Parameters
/// ----------
/// text : str
///     Input Unicode text.
/// errors : Optional[str]
///     One of: "ignore" (drop unmapped), "replace" (use `replace_str` or '?'),
///     "strict" (raise UnidecodeError), "preserve" (keep original char),
///     "invalid" (alias of preserve), or None/"" (default == ignore).
/// replace_str : Optional[str]
///     Replacement string when `errors="replace"` (default '?').
///
/// Returns
/// -------
/// str
///     Transliteration (ASCII except in preserve/invalid modes where original
///     unmapped chars are emitted as-is).
///
/// Raises
/// ------
/// UnidecodeError
///     If `errors="strict"` and an unmapped character is encountered. The
///     exception exposes an `index` attribute giving the character index.
fn unidecode(string: &str, errors: Option<&str>, replace_str: Option<&str>) -> PyResult<String> {
    // Attempt to extract a Rust String from the Python object. If the Python
    // string contains unpaired surrogates, extraction may fail; in that case
    // we fall back to encoding/decoding via 'utf-16' with 'surrogatepass'.
    let string: String = string.to_string();
    use crate::ErrorsPolicy;
    let policy = match errors.unwrap_or("") {
        "" => ErrorsPolicy::Default,
        "ignore" => ErrorsPolicy::Ignore,
        "replace" => {
            let rep = replace_str.unwrap_or("?");
            ErrorsPolicy::Replace { replace: rep }
        }
        "preserve" => ErrorsPolicy::Preserve,
        // Accept 'invalid' as historical alias of 'preserve' (keeps original char)
        "invalid" => ErrorsPolicy::Preserve,
        "strict" => ErrorsPolicy::Strict,
        other => return Err(pyo3::exceptions::PyValueError::new_err(format!("unknown errors policy: {}", other)))
    };
    match crate::unidecode_with_policy_result(&string, policy) {
        Ok(s) => Ok(s),
        Err(idx) => {
            // Create error instance of UnidecodeError, attach index attribute, raise.
            let mut err = UnidecodeError::new_err("unidecode strict error");
            Python::with_gil(|py| {
                let _ = err.value(py).setattr("index", idx);
            });
            Err(err)
        }
    }
}

#[cfg(feature = "python")]
#[pyfunction(signature = (string, errors=None, replace_str=None), text_signature = "(string, errors=None, replace_str=None)")]
/// Alias matching upstream: `unidecode_expect_ascii(string, errors, replace_str)`
fn unidecode_expect_ascii(
    string: &str,
    errors: Option<&str>,
    replace_str: Option<&str>,
) -> PyResult<String> {
    unidecode(string, errors, replace_str)
}

#[cfg(feature = "python")]
#[pyfunction(signature = (string, errors=None, replace_str=None), text_signature = "(string, errors=None, replace_str=None)")]
/// Alias matching upstream: `unidecode_expect_nonascii(string, errors, replace_str)`
fn unidecode_expect_nonascii(
    string: &str,
    errors: Option<&str>,
    replace_str: Option<&str>,
) -> PyResult<String> {
    unidecode(string, errors, replace_str)
}

#[cfg(feature = "python")]
#[pymodule]
fn unidecode_rs(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // Expose UnidecodeError exception type and the unidecode function.
    m.add_function(wrap_pyfunction!(unidecode, m)?)?;
    // Expose upstream-compatible aliases so Python users can switch
    // imports transparently.
    m.add_function(wrap_pyfunction!(unidecode_expect_ascii, m)?)?;
    m.add_function(wrap_pyfunction!(unidecode_expect_nonascii, m)?)?;
    m.add("UnidecodeError", py.get_type::<UnidecodeError>())?;
    let version = env!("CARGO_PKG_VERSION");
    m.setattr("__version__", version)?;
    Ok(())
}
