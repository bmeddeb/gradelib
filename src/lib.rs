use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// Use pyo3-async-runtimes
use pyo3_async_runtimes::tokio;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc; // Needed for calling method via Arc

// Declare repo module (references repo.rs)
mod repo;
// Import necessary public items from repo module
use repo::{InternalCloneStatus, InternalRepoCloneTask, InternalRepoManagerLogic};

// --- Exposed Python Class: CloneStatus ---
#[pyclass(name = "CloneStatus", module = "gradelib")] // Add module for clarity
#[derive(Debug, Clone)]
pub struct ExposedCloneStatus {
    #[pyo3(get)]
    pub status_type: String,
    #[pyo3(get)]
    pub progress: Option<u8>,
    #[pyo3(get)]
    pub error: Option<String>,
}

// Conversion from internal Rust enum to exposed Python class
impl From<InternalCloneStatus> for ExposedCloneStatus {
    fn from(status: InternalCloneStatus) -> Self {
        match status {
            InternalCloneStatus::Queued => Self {
                status_type: "queued".to_string(),
                progress: None,
                error: None,
            },
            InternalCloneStatus::Cloning(p) => Self {
                status_type: "cloning".to_string(),
                progress: Some(p),
                error: None,
            },
            InternalCloneStatus::Completed => Self {
                status_type: "completed".to_string(),
                progress: None,
                error: None,
            },
            InternalCloneStatus::Failed(e) => Self {
                status_type: "failed".to_string(),
                progress: None,
                error: Some(e),
            },
        }
    }
}

// --- Exposed Python Class: CloneTask ---
#[pyclass(name = "CloneTask", module = "gradelib")] // Add module for clarity
#[derive(Debug, Clone)]
pub struct ExposedCloneTask {
    #[pyo3(get)]
    pub url: String,
    #[pyo3(get)]
    pub status: ExposedCloneStatus, // Uses the exposed status type
    #[pyo3(get)]
    pub temp_dir: Option<String>,
}

// Conversion from internal Rust struct to exposed Python class
impl From<InternalRepoCloneTask> for ExposedCloneTask {
    fn from(task: InternalRepoCloneTask) -> Self {
        Self {
            url: task.url,
            status: task.status.into(), // Convert internal status via its From impl
            temp_dir: task.temp_dir.map(|p| p.to_string_lossy().to_string()),
        }
    }
}

// --- Exposed Python Class: RepoManager ---
#[pyclass(name = "RepoManager", module = "gradelib")] // Add module for clarity
#[derive(Clone)]
pub struct RepoManager {
    // Holds the internal logic handler using Arc for shared ownership
    inner: Arc<InternalRepoManagerLogic>,
}

#[pymethods]
impl RepoManager {
    #[new]
    fn new(urls: Vec<String>, github_username: String, github_token: String) -> Self {
        let string_urls: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
        // Create the internal logic handler instance, wrapped in Arc
        Self {
            inner: Arc::new(InternalRepoManagerLogic::new(
                &string_urls,
                &github_username,
                &github_token,
            )),
        }
    }

    /// Clones all repositories configured in this manager instance asynchronously.
    #[pyo3(name = "clone_all")]
    fn clone_all<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner); // Clone Arc for the async block
                                             // Convert the async Rust future into a Python awaitable
        tokio::future_into_py(py, async move {
            inner.clone_all().await; // Delegate to internal logic
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    /// Fetches the current status of all cloning tasks asynchronously.
    /// Returns a dictionary mapping repository URLs to CloneTask objects.
    #[pyo3(name = "fetch_clone_tasks")]
    fn fetch_clone_tasks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner); // Clone Arc for the async block
        tokio::future_into_py(py, async move {
            // Get tasks in their internal representation
            let internal_tasks = inner.get_internal_tasks().await;
            // Convert internal tasks to the exposed task type
            let result: HashMap<String, ExposedCloneTask> = internal_tasks
                .into_iter()
                .map(|(k, v)| (k, v.into())) // Use From impl for conversion
                .collect();

            // Convert the Rust HashMap to a Python dictionary
            Python::with_gil(|py| {
                let dict = PyDict::new(py);
                for (k, v) in result {
                    // Insert each key-value pair, v is PyClass, directly convertible
                    dict.set_item(k, v)?;
                }
                // Return the dictionary as a Python object
                Ok(dict.into_py(py))
            })
        })
    }

    /// Clones a single repository specified by URL asynchronously.
    #[pyo3(name = "clone")]
    fn clone<'py>(&self, py: Python<'py>, url: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner); // Clone Arc for the async block
        let url_clone = url.clone(); // Clone the URL for the closure
        tokio::future_into_py(py, async move {
            // Call the clone method on InternalRepoManagerLogic through deref()
            inner.deref().clone(url_clone).await;
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    /// Performs 'git blame' on multiple files within a cloned repository asynchronously.
    #[pyo3(name = "bulk_blame")]
    fn bulk_blame<'py>(
        &self,
        py: Python<'py>,
        target_repo_url: String,
        file_paths: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner); // Clone Arc for the async block
        tokio::future_into_py(py, async move {
            // Call the internal logic method
            let result_map = inner.bulk_blame(&target_repo_url, file_paths).await;

            // Convert the Rust result HashMap into a Python dictionary
            Python::with_gil(|py| {
                match result_map {
                    // Outer Ok: The bulk operation itself didn't fail early (e.g., repo found)
                    Ok(blame_results_map) => {
                        let py_result_dict = PyDict::new(py);

                        // Iterate through results for each file
                        for (file_path, blame_result) in blame_results_map {
                            match blame_result {
                                // Inner Ok: Blame for this file succeeded
                                Ok(blame_lines) => {
                                    let py_blame_list = PyList::empty(py);
                                    // Convert each BlameLineInfo struct to a PyDict
                                    for line_info in blame_lines {
                                        let line_dict = PyDict::new(py);
                                        // Using &line_info.* passes slices for Strings, avoiding clone
                                        line_dict.set_item("commit_id", &line_info.commit_id)?;
                                        line_dict
                                            .set_item("author_name", &line_info.author_name)?;
                                        line_dict
                                            .set_item("author_email", &line_info.author_email)?;
                                        line_dict
                                            .set_item("orig_line_no", line_info.orig_line_no)?;
                                        line_dict
                                            .set_item("final_line_no", line_info.final_line_no)?;
                                        line_dict
                                            .set_item("line_content", &line_info.line_content)?;
                                        py_blame_list.append(line_dict)?;
                                    }
                                    // Add the list of dicts to the main result dict
                                    py_result_dict.set_item(file_path, py_blame_list)?;
                                }
                                // Inner Err: Blame for this file failed
                                Err(err_string) => {
                                    // Add the error string directly as the value for this file
                                    py_result_dict.set_item(file_path, err_string)?;
                                }
                            }
                        }
                        // Return the final Python dictionary {file: [lines] | error}
                        Ok(py_result_dict.into_py(py))
                    }
                    // Outer Err: The bulk operation failed before processing files (e.g., repo not found)
                    Err(err_string) => {
                        // Raise a Python exception for overall failures
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            }) // End Python::with_gil
        }) // End future_into_py
    }
}

// --- Exposed Python Function: setup_async ---
/// Initializes the asynchronous runtime environment needed for manager operations.
#[pyfunction]
fn setup_async(_py: Python) -> PyResult<()> {
    // Initialize the tokio runtime for pyo3-async-runtimes
    let mut builder = ::tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    tokio::init(builder);
    Ok(())
}

// --- Python Module Definition ---
// Ensure this function name matches the library name in Cargo.toml ('gradelib')
#[pymodule]
fn gradelib(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(setup_async, m)?)?;
    m.add_class::<RepoManager>()?; // Exposes RepoManager
    m.add_class::<ExposedCloneTask>()?; // Exposes CloneTask
    m.add_class::<ExposedCloneStatus>()?; // Exposes CloneStatus
                                          // BlameLineInfo is not exposed as a class, only as dicts within bulk_blame result
    Ok(())
}
