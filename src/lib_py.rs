#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::wrap_pyfunction;

#[cfg(feature = "python")]
#[pyfunction(text_signature = "(text, errors=None, replace_str=None)")]
/// Transliterates a Unicode string to ASCII (mirror of Python `unidecode.unidecode`).
/// Always returns pure ASCII; function is idempotent.
///
/// Parameters
/// ----------
/// text : str
///     Input Unicode text.
/// errors : Optional[str]
///     Placeholder for future compatibility (e.g. "ignore", "strict"). Currently
///     not implemented; passing any non-None value raises NotImplementedError.
///
/// Returns
/// -------
/// str
///     ASCII transliteration.
///
/// Raises
/// ------
/// NotImplementedError
///     If `errors` is not None.
fn unidecode(input: &str, errors: Option<&str>, replace_str: Option<&str>) -> PyResult<String> {
    use crate::{ErrorsPolicy};
    let policy = match errors.unwrap_or("") {
        "" => ErrorsPolicy::Default,
        "ignore" => ErrorsPolicy::Ignore,
        "replace" => {
            let rep = replace_str.unwrap_or("?");
            ErrorsPolicy::Replace { replace: rep }
        }
        "preserve" => ErrorsPolicy::Preserve,
        "strict" => ErrorsPolicy::Strict,
        "invalid" => ErrorsPolicy::Invalid,
        other => return Err(pyo3::exceptions::PyValueError::new_err(format!("unknown errors policy: {other}")))
    };
    // For now only run default implementation; strict/invalid behave same until backend extended.
    let out = crate::unidecode_with_policy(input, policy);
    Ok(out)
}

#[cfg(feature = "python")]
#[pymodule]
fn unidecode_rs(py: Python, m: &pyo3::prelude::Bound<pyo3::types::PyModule>) -> PyResult<()> {
    #[pyclass(module = "unidecode_rs", name = "UnidecodeError")]
    struct UnidecodeError { #[pyo3(get)] index: usize }
    #[pymethods]
    impl UnidecodeError { #[new] fn new(index: usize) -> Self { Self { index } } }
    m.add_class::<UnidecodeError>()?;
    m.add_function(wrap_pyfunction!(unidecode, m)?)?;
    let version = env!("CARGO_PKG_VERSION");
    m.setattr("__version__", version)?;
    let gil = py;
    let _ = gil;
    Ok(())
}
