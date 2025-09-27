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
#[pyfunction(signature = (text, errors=None, replace_str=None), text_signature = "(text, errors=None, replace_str=None)")]
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
fn unidecode(text: &str, errors: Option<&str>, replace_str: Option<&str>) -> PyResult<String> {
    use crate::ErrorsPolicy;
    let policy = match errors.unwrap_or("") {
        "" => ErrorsPolicy::Default,
        "ignore" => ErrorsPolicy::Ignore,
        "replace" => {
            let rep = replace_str.unwrap_or("?");
            ErrorsPolicy::Replace { replace: rep }
        }
        "preserve" => ErrorsPolicy::Preserve,
        "invalid" => ErrorsPolicy::Invalid,
        "strict" => ErrorsPolicy::Strict,
        other => return Err(pyo3::exceptions::PyValueError::new_err(format!("unknown errors policy: {other}")))
    };
    match crate::unidecode_with_policy_result(text, policy) {
        Ok(s) => Ok(s),
        Err(idx) => {
            // Create error instance, attach index attribute, raise.
            let mut err = UnidecodeError::new_err("unidecode strict error");
            Python::with_gil(|py| {
                let _ = err.value(py).setattr("index", idx);
            });
            Err(err)
        }
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn unidecode_rs(py: Python, m: &pyo3::prelude::Bound<pyo3::types::PyModule>) -> PyResult<()> {
    m.add("UnidecodeError", py.get_type::<UnidecodeError>())?;
    m.add_function(wrap_pyfunction!(unidecode, m)?)?;
    let version = env!("CARGO_PKG_VERSION");
    m.setattr("__version__", version)?;
    let gil = py;
    let _ = gil;
    Ok(())
}
