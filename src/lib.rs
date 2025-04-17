use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// Use pyo3-async-runtimes
use pyo3_async_runtimes::tokio;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc; // Needed for calling method via Arc

// --- Declare modules ---
pub(crate) mod blame;
pub(crate) mod branch;
pub(crate) mod clone;
pub(crate) mod collaborators;
pub(crate) mod commits;
pub(crate) mod issues;
pub(crate) mod pull_requests;
pub(crate) mod repo;

// --- Import necessary items from modules ---
// Import directly from source modules
use crate::clone::{InternalCloneStatus, InternalRepoCloneTask};
use repo::InternalRepoManagerLogic; // Keep this as it's defined in repo // Import from clone module
                                                                         // use crate::commits::CommitInfo; // Remove unused import

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
            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                let dict = PyDict::new(py);
                for (k, v) in result {
                    dict.set_item(k, v)?;
                }
                Ok(dict.into())
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
            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result_map {
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
                        Ok(py_result_dict.into())
                    }
                    // Outer Err: The bulk operation failed before processing files (e.g., repo not found)
                    Err(err_string) => {
                        // Raise a Python exception for overall failures
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    /// Analyzes the commit history of a cloned repository asynchronously.
    #[pyo3(name = "analyze_commits")]
    fn analyze_commits<'py>(
        &self,
        py: Python<'py>,
        target_repo_url: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let url_clone = target_repo_url.clone(); // Clone for the async block

        tokio::future_into_py(py, async move {
            // Call the internal (now synchronous) logic method
            // We still block the current tokio thread managed by pyo3-async, which is acceptable
            // if the rayon work takes significant time, but alternatives exist if needed.
            let result_vec = inner.get_commit_analysis(&url_clone);

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result_vec {
                    Ok(commit_infos) => {
                        let py_commit_list = PyList::empty(py);
                        for info in commit_infos {
                            let commit_dict = PyDict::new(py);
                            commit_dict.set_item("sha", &info.sha)?;
                            commit_dict.set_item("repo_name", &info.repo_name)?; // Add repo_name
                            commit_dict.set_item("message", &info.message)?;
                            commit_dict.set_item("author_name", &info.author_name)?;
                            commit_dict.set_item("author_email", &info.author_email)?;
                            commit_dict.set_item("author_timestamp", info.author_timestamp)?;
                            commit_dict.set_item("author_offset", info.author_offset)?;
                            commit_dict.set_item("committer_name", &info.committer_name)?;
                            commit_dict.set_item("committer_email", &info.committer_email)?;
                            commit_dict
                                .set_item("committer_timestamp", info.committer_timestamp)?;
                            commit_dict.set_item("committer_offset", info.committer_offset)?;
                            commit_dict.set_item("additions", info.additions)?;
                            commit_dict.set_item("deletions", info.deletions)?;
                            commit_dict.set_item("is_merge", info.is_merge)?;
                            // commit_dict.set_item("url", &info.url)?; // URL moved out of CommitInfo struct
                            py_commit_list.append(commit_dict)?;
                        }
                        Ok(py_commit_list.into())
                    }
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    /// Fetches collaborator information for multiple repositories.
    #[pyo3(name = "fetch_collaborators")]
    fn fetch_collaborators<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use the existing credentials from the RepoManager
        let github_username = self.inner.github_username.clone();
        let github_token = self.inner.github_token.clone();

        tokio::future_into_py(py, async move {
            let result = collaborators::fetch_collaborators(
                repo_urls,
                &github_username, // Even though prefixed with underscore in the implementation,
                &github_token,    // we still need to pass it here
            )
            .await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(collab_map) => {
                        let py_result_dict = PyDict::new(py);

                        for (repo_url, collaborators) in collab_map {
                            let py_collab_list = PyList::empty(py);

                            for collab in collaborators {
                                let collab_dict = PyDict::new(py);
                                collab_dict.set_item("login", &collab.login)?;
                                collab_dict.set_item("github_id", collab.github_id)?;

                                if let Some(name) = &collab.full_name {
                                    collab_dict.set_item("full_name", name)?;
                                } else {
                                    collab_dict.set_item("full_name", py.None())?;
                                }

                                if let Some(email) = &collab.email {
                                    collab_dict.set_item("email", email)?;
                                } else {
                                    collab_dict.set_item("email", py.None())?;
                                }

                                if let Some(avatar) = &collab.avatar_url {
                                    collab_dict.set_item("avatar_url", avatar)?;
                                } else {
                                    collab_dict.set_item("avatar_url", py.None())?;
                                }

                                py_collab_list.append(collab_dict)?;
                            }

                            py_result_dict.set_item(repo_url, py_collab_list)?;
                        }

                        Ok(py_result_dict.into())
                    }
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    /// Fetches issue information for multiple repositories.
    #[pyo3(name = "fetch_issues")]
    fn fetch_issues<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        state: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use the existing credentials from the RepoManager
        let github_username = self.inner.github_username.clone();
        let github_token = self.inner.github_token.clone();
        
        tokio::future_into_py(py, async move {
            let result = issues::fetch_issues(
                repo_urls,
                &github_username,
                &github_token,
                state.as_deref()
            ).await;
            
            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(issue_map) => {
                        let py_result_dict = PyDict::new(py);
                        
                        for (repo_url, result) in issue_map {
                            match result {
                                Ok(issues) => {
                                    let py_issue_list = PyList::empty(py);
                                    
                                    for issue in issues {
                                        let issue_dict = PyDict::new(py);
                                        issue_dict.set_item("id", issue.id)?;
                                        issue_dict.set_item("number", issue.number)?;
                                        issue_dict.set_item("title", &issue.title)?;
                                        issue_dict.set_item("state", &issue.state)?;
                                        issue_dict.set_item("created_at", &issue.created_at)?;
                                        issue_dict.set_item("updated_at", &issue.updated_at)?;
                                        
                                        if let Some(closed_at) = &issue.closed_at {
                                            issue_dict.set_item("closed_at", closed_at)?;
                                        } else {
                                            issue_dict.set_item("closed_at", py.None())?;
                                        }
                                        
                                        issue_dict.set_item("user_login", &issue.user_login)?;
                                        issue_dict.set_item("user_id", issue.user_id)?;
                                        
                                        if let Some(body) = &issue.body {
                                            issue_dict.set_item("body", body)?;
                                        } else {
                                            issue_dict.set_item("body", py.None())?;
                                        }
                                        
                                        issue_dict.set_item("comments_count", issue.comments_count)?;
                                        issue_dict.set_item("is_pull_request", issue.is_pull_request)?;
                                        issue_dict.set_item("labels", &issue.labels)?;
                                        issue_dict.set_item("assignees", &issue.assignees)?;
                                        
                                        if let Some(milestone) = &issue.milestone {
                                            issue_dict.set_item("milestone", milestone)?;
                                        } else {
                                            issue_dict.set_item("milestone", py.None())?;
                                        }
                                        
                                        issue_dict.set_item("locked", issue.locked)?;
                                        issue_dict.set_item("html_url", &issue.html_url)?;
                                        
                                        py_issue_list.append(issue_dict)?;
                                    }
                                    
                                    py_result_dict.set_item(repo_url, py_issue_list)?;
                                },
                                Err(error) => {
                                    // Store error message
                                    py_result_dict.set_item(repo_url, error)?;
                                }
                            }
                        }
                        
                        Ok(py_result_dict.into_py(py))
                    },
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    /// Fetches pull request information for multiple repositories.
    #[pyo3(name = "fetch_pull_requests")]
    fn fetch_pull_requests<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        state: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use the existing credentials from the RepoManager
        let github_username = self.inner.github_username.clone();
        let github_token = self.inner.github_token.clone();

        tokio::future_into_py(py, async move {
            let result = pull_requests::fetch_pull_requests(
                repo_urls,
                &github_username,
                &github_token,
                state.as_deref(),
            )
            .await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(pr_map) => {
                        let py_result_dict = PyDict::new(py);

                        for (repo_url, result) in pr_map {
                            match result {
                                Ok(prs) => {
                                    let py_pr_list = PyList::empty(py);

                                    for pr in prs {
                                        let pr_dict = PyDict::new(py);
                                        pr_dict.set_item("id", pr.id)?;
                                        pr_dict.set_item("number", pr.number)?;
                                        pr_dict.set_item("title", &pr.title)?;
                                        pr_dict.set_item("state", &pr.state)?;
                                        pr_dict.set_item("created_at", &pr.created_at)?;
                                        pr_dict.set_item("updated_at", &pr.updated_at)?;

                                        if let Some(closed_at) = &pr.closed_at {
                                            pr_dict.set_item("closed_at", closed_at)?;
                                        } else {
                                            pr_dict.set_item("closed_at", py.None())?;
                                        }

                                        if let Some(merged_at) = &pr.merged_at {
                                            pr_dict.set_item("merged_at", merged_at)?;
                                        } else {
                                            pr_dict.set_item("merged_at", py.None())?;
                                        }

                                        pr_dict.set_item("user_login", &pr.user_login)?;
                                        pr_dict.set_item("user_id", pr.user_id)?;

                                        if let Some(body) = &pr.body {
                                            pr_dict.set_item("body", body)?;
                                        } else {
                                            pr_dict.set_item("body", py.None())?;
                                        }

                                        pr_dict.set_item("comments", pr.comments)?;
                                        pr_dict.set_item("commits", pr.commits)?;
                                        pr_dict.set_item("additions", pr.additions)?;
                                        pr_dict.set_item("deletions", pr.deletions)?;
                                        pr_dict.set_item("changed_files", pr.changed_files)?;

                                        if let Some(mergeable) = pr.mergeable {
                                            pr_dict.set_item("mergeable", mergeable)?;
                                        } else {
                                            pr_dict.set_item("mergeable", py.None())?;
                                        }

                                        pr_dict.set_item("labels", &pr.labels)?;
                                        pr_dict.set_item("is_draft", pr.draft)?;
                                        pr_dict.set_item("merged", pr.merged)?;

                                        if let Some(merged_by) = &pr.merged_by {
                                            pr_dict.set_item("merged_by", merged_by)?;
                                        } else {
                                            pr_dict.set_item("merged_by", py.None())?;
                                        }

                                        py_pr_list.append(pr_dict)?;
                                    }

                                    py_result_dict.set_item(repo_url, py_pr_list)?;
                                }
                                Err(error) => {
                                    // Store error message
                                    py_result_dict.set_item(repo_url, error)?;
                                }
                            }
                        }

                        Ok(py_result_dict.into())
                    }
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    /// Analyzes branches in cloned repositories.
    #[pyo3(name = "analyze_branches")]
    fn analyze_branches<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);

        tokio::future_into_py(py, async move {
            // Get paths for all requested repositories
            let mut repo_paths = Vec::new();

            {
                let tasks = inner.tasks.lock().unwrap();

                for url in &repo_urls {
                    if let Some(task) = tasks.get(url) {
                        match &task.status {
                            InternalCloneStatus::Completed => {
                                if let Some(path) = &task.temp_dir {
                                    repo_paths.push((url.clone(), path.clone()));
                                }
                            }
                            _ => {
                                // Skip repositories that aren't completed
                                eprintln!("Repository {} is not in completed state, skipping", url);
                            }
                        }
                    } else {
                        eprintln!("Repository {} is not managed, skipping", url);
                    }
                }
            }

            // Process branches in parallel (will be executed on a blocking thread)
            // Use ::tokio for direct access to the full tokio crate
            let result_map = ::tokio::task::spawn_blocking(move || {
                branch::extract_branches_parallel(repo_paths)
            })
            .await
            .unwrap_or_else(|e| {
                // Handle join error
                let mut error_map = HashMap::new();
                for url in repo_urls {
                    error_map.insert(url, Err(format!("Task execution failed: {}", e)));
                }
                error_map
            });

            // Convert results to Python objects
            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                let py_result_dict = PyDict::new(py);

                for (repo_url, result) in result_map {
                    match result {
                        Ok(branch_infos) => {
                            let py_branch_list = PyList::empty(py);

                            for info in branch_infos {
                                let branch_dict = PyDict::new(py);
                                branch_dict.set_item("name", &info.name)?;
                                branch_dict.set_item("is_remote", info.is_remote)?;
                                branch_dict.set_item("commit_id", &info.commit_id)?;
                                branch_dict.set_item("commit_message", &info.commit_message)?;
                                branch_dict.set_item("author_name", &info.author_name)?;
                                branch_dict.set_item("author_email", &info.author_email)?;
                                branch_dict.set_item("author_time", info.author_time)?;
                                branch_dict.set_item("is_head", info.is_head)?;

                                if let Some(remote) = &info.remote_name {
                                    branch_dict.set_item("remote_name", remote)?;
                                } else {
                                    branch_dict.set_item("remote_name", py.None())?;
                                }

                                py_branch_list.append(branch_dict)?;
                            }

                            py_result_dict.set_item(repo_url, py_branch_list)?;
                        }
                        Err(error) => {
                            // Store error message
                            py_result_dict.set_item(repo_url, error)?;
                        }
                    }
                }

                Ok(py_result_dict.into())
            })
        })
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
