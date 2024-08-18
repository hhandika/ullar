use pyo3::prelude::*;
use ullar::helper::common::get_api_version;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn api_version() -> String {
    get_api_version()
}

/// A Python module implemented in Rust.
#[pymodule]
fn py_ullar(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(api_version, m)?)?;
    Ok(())
}
