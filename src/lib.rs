use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3_async_runtimes::tokio;

pub mod common;
pub mod github;
pub mod utils;

// Setup function for the async runtime
#[pyfunction]
fn setup_async(_py: Python) -> PyResult<()> {
    let mut builder = ::tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    tokio::init(builder);
    Ok(())
}

// GitHub module
#[pymodule]
fn github_module(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<github::provider::GitHubProviderPy>()?;
    Ok(())
}

// Main module
#[pymodule]
fn gradelib(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(setup_async, m)?)?;

    // Add the GitHub submodule
    let github_module = pyo3::wrap_pymodule!(github_module);
    m.add_wrapped(github_module)?;

    Ok(())
}
