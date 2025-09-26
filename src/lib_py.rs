#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::wrap_pyfunction;

#[cfg(feature = "python")]
#[pyfunction]
fn unidecode(input: &str) -> PyResult<String> {
    Ok(crate::unidecode_rust(input))
}

#[cfg(feature = "python")]
#[pymodule]
fn unidecode_rs(_py: Python, m: &pyo3::prelude::Bound<pyo3::types::PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(unidecode, m)?)?;
    Ok(())
}
