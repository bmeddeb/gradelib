#![allow(dead_code)]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// Use pyo3-async-runtimes
use pyo3_async_runtimes::tokio;

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc; // Needed for calling method via Arc

// --- Declare modules ---
pub(crate) mod providers;

// Re-export GitHub provider modules
pub(crate) use providers::github::blame;
pub(crate) use providers::github::branch;
pub(crate) use providers::github::clone;
pub(crate) use providers::github::code_review;
pub(crate) use providers::github::collaborators;
pub(crate) use providers::github::comments;
pub(crate) use providers::github::commits;
pub(crate) use providers::github::issues;
pub(crate) use providers::github::oauth::GitHubOAuthClient;
pub(crate) use providers::github::pull_requests;
pub(crate) use providers::github::repo;

// Re-export Taiga provider modules
pub(crate) use providers::taiga::client;
pub(crate) use providers::taiga::orchestrator;

// --- Import necessary items from modules ---
// Import directly from source modules
use crate::clone::{InternalCloneStatus, InternalRepoCloneTask};
use repo::InternalRepoManagerLogic;
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
    #[pyo3(signature = (urls, github_token, github_username=None))]
    fn new(urls: Vec<String>, github_token: String, github_username: Option<String>) -> Self {
        let string_urls: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
        // Use an empty string if username is None
        let username = github_username.unwrap_or_default();
        // Create the internal logic handler with username and token
        Self {
            inner: Arc::new(InternalRepoManagerLogic::new(
                &string_urls,
                &username,
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
            let _ = inner.deref().clone(url_clone).await;
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    /// Performs 'git blame' on multiple files within a cloned repository asynchronously.
    #[pyo3(name = "bulk_blame")]
    fn bulk_blame<'py>(
        &self,
        py: Python<'py>,
        repo_path: String,
        file_paths: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner); // Clone Arc for the async block
        tokio::future_into_py(py, async move {
            let result_map = inner
                .bulk_blame(&PathBuf::from(repo_path), file_paths)
                .await;
            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result_map {
                    Ok(blame_results_map) => {
                        let py_result_dict = PyDict::new(py);
                        for (file_path, blame_result) in blame_results_map {
                            match blame_result {
                                Ok(blame_lines) => {
                                    let py_blame_list = PyList::empty(py);
                                    for line_info in blame_lines {
                                        let line_dict = PyDict::new(py);
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
                                    py_result_dict.set_item(file_path, py_blame_list)?;
                                }
                                Err(err_string) => {
                                    py_result_dict.set_item(file_path, err_string)?;
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

    /// Analyzes the commit history of a cloned repository asynchronously.
    #[pyo3(name = "analyze_commits")]
    fn analyze_commits<'py>(
        &self,
        py: Python<'py>,
        repo_path: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let repo_path_clone = repo_path.clone();
        tokio::future_into_py(py, async move {
            let result_vec = inner.get_commit_analysis(&PathBuf::from(repo_path_clone));
            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result_vec {
                    Ok(commit_infos) => {
                        let py_commit_list = PyList::empty(py);
                        for info in commit_infos {
                            let commit_dict = PyDict::new(py);
                            commit_dict.set_item("sha", &info.sha)?;
                            commit_dict.set_item("repo_name", &info.repo_name)?;
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
    /// Returns a dictionary mapping each repo URL to either a list of collaborators (on success)
    /// or an error string (on failure for that repo). No exceptions are raised for individual failures.
    #[pyo3(name = "fetch_collaborators")]
    fn fetch_collaborators<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        max_pages: Option<usize>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use the existing credentials from the RepoManager
        let github_username = self.inner.github_username.clone();
        let github_token = self.inner.github_token.clone();

        tokio::future_into_py(py, async move {
            let result = collaborators::fetch_collaborators(
                repo_urls,
                &github_username, // Even though prefixed with underscore in the implementation,
                &github_token,    // we still need to pass it here
                max_pages,
            )
            .await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(collab_map) => {
                        let py_result_dict = PyDict::new(py);

                        for (repo_url, result) in collab_map {
                            match result {
                                Ok(collaborators) => {
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
                                Err(error) => {
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

    /// Fetches issue information for multiple repositories.
    #[pyo3(name = "fetch_issues")]
    fn fetch_issues<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        state: Option<String>,
        max_pages: Option<usize>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use the existing credentials from the RepoManager
        let github_username = self.inner.github_username.clone();
        let github_token = self.inner.github_token.clone();

        tokio::future_into_py(py, async move {
            let result = issues::fetch_issues(
                repo_urls,
                &github_username,
                &github_token,
                state.as_deref(),
                max_pages,
            )
            .await;

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

                                        issue_dict
                                            .set_item("comments_count", issue.comments_count)?;
                                        issue_dict
                                            .set_item("is_pull_request", issue.is_pull_request)?;
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

    /// Fetches pull request information for multiple repositories.
    #[pyo3(name = "fetch_pull_requests")]
    fn fetch_pull_requests<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        state: Option<String>,
        max_pages: Option<usize>,
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
                max_pages,
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

    /// Fetches code review information for multiple repositories.
    #[pyo3(name = "fetch_code_reviews")]
    fn fetch_code_reviews<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        max_pages: Option<usize>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use the existing credentials from the RepoManager
        let github_username = self.inner.github_username.clone();
        let github_token = self.inner.github_token.clone();

        tokio::future_into_py(py, async move {
            let result = code_review::fetch_code_reviews(
                repo_urls,
                &github_username,
                &github_token,
                max_pages,
            )
            .await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(reviews_map) => {
                        let py_result_dict = PyDict::new(py);

                        for (repo_url, result) in reviews_map {
                            match result {
                                Ok(pr_reviews) => {
                                    let py_pr_reviews_dict = PyDict::new(py);

                                    for (pr_number, reviews) in pr_reviews {
                                        let py_reviews_list = PyList::empty(py);

                                        for review in reviews {
                                            let review_dict = PyDict::new(py);
                                            review_dict.set_item("id", review.id)?;
                                            review_dict.set_item("pr_number", review.pr_number)?;
                                            review_dict
                                                .set_item("user_login", &review.user_login)?;
                                            review_dict.set_item("user_id", review.user_id)?;

                                            if let Some(body) = &review.body {
                                                review_dict.set_item("body", body)?;
                                            } else {
                                                review_dict.set_item("body", py.None())?;
                                            }

                                            review_dict.set_item("state", &review.state)?;
                                            review_dict
                                                .set_item("submitted_at", &review.submitted_at)?;
                                            review_dict.set_item("commit_id", &review.commit_id)?;
                                            review_dict.set_item("html_url", &review.html_url)?;

                                            py_reviews_list.append(review_dict)?;
                                        }

                                        py_pr_reviews_dict
                                            .set_item(pr_number.to_string(), py_reviews_list)?;
                                    }

                                    py_result_dict.set_item(repo_url, py_pr_reviews_dict)?;
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

    /// Fetches comments of various types for multiple repositories.
    #[pyo3(name = "fetch_comments")]
    fn fetch_comments<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        comment_types: Option<Vec<String>>,
        max_pages: Option<usize>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Use the existing credentials from the RepoManager
        let github_username = self.inner.github_username.clone();
        let github_token = self.inner.github_token.clone();

        // Convert string comment types to CommentType enum if provided
        let types_enum = match comment_types {
            Some(types) => {
                let mut enum_types = Vec::new();
                for type_str in types {
                    match type_str.to_lowercase().as_str() {
                        "issue" => enum_types.push(comments::CommentType::Issue),
                        "commit" => enum_types.push(comments::CommentType::Commit),
                        "pullrequest" | "pull_request" => {
                            enum_types.push(comments::CommentType::PullRequest)
                        }
                        "reviewcomment" | "review_comment" => {
                            enum_types.push(comments::CommentType::ReviewComment)
                        }
                        _ => {
                            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                                format!("Invalid comment type: {}. Valid types are: issue, commit, pullrequest, reviewcomment", type_str)
                            ));
                        }
                    }
                }
                Some(enum_types)
            }
            None => None,
        };

        tokio::future_into_py(py, async move {
            let result = comments::fetch_comments(
                repo_urls,
                &github_username,
                &github_token,
                types_enum,
                max_pages,
            )
            .await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(comments_map) => {
                        let py_result_dict = PyDict::new(py);

                        for (repo_url, result) in comments_map {
                            match result {
                                Ok(comments) => {
                                    let py_comments_list = PyList::empty(py);

                                    for comment in comments {
                                        let comment_dict = PyDict::new(py);
                                        comment_dict.set_item("id", comment.id)?;

                                        // Convert enum to string for Python
                                        let comment_type = match comment.comment_type {
                                            comments::CommentType::Issue => "issue",
                                            comments::CommentType::Commit => "commit",
                                            comments::CommentType::PullRequest => "pull_request",
                                            comments::CommentType::ReviewComment => {
                                                "review_comment"
                                            }
                                        };
                                        comment_dict.set_item("comment_type", comment_type)?;

                                        comment_dict.set_item("user_login", &comment.user_login)?;
                                        comment_dict.set_item("user_id", comment.user_id)?;
                                        comment_dict.set_item("body", &comment.body)?;
                                        comment_dict.set_item("created_at", &comment.created_at)?;
                                        comment_dict.set_item("updated_at", &comment.updated_at)?;
                                        comment_dict.set_item("html_url", &comment.html_url)?;

                                        // Handle optional fields
                                        if let Some(issue_number) = comment.issue_number {
                                            comment_dict.set_item("issue_number", issue_number)?;
                                        } else {
                                            comment_dict.set_item("issue_number", py.None())?;
                                        }

                                        if let Some(pr_number) = comment.pull_request_number {
                                            comment_dict
                                                .set_item("pull_request_number", pr_number)?;
                                        } else {
                                            comment_dict
                                                .set_item("pull_request_number", py.None())?;
                                        }

                                        if let Some(commit_id) = &comment.commit_id {
                                            comment_dict.set_item("commit_id", commit_id)?;
                                        } else {
                                            comment_dict.set_item("commit_id", py.None())?;
                                        }

                                        if let Some(path) = &comment.path {
                                            comment_dict.set_item("path", path)?;
                                        } else {
                                            comment_dict.set_item("path", py.None())?;
                                        }

                                        if let Some(position) = comment.position {
                                            comment_dict.set_item("position", position)?;
                                        } else {
                                            comment_dict.set_item("position", py.None())?;
                                        }

                                        if let Some(line) = comment.line {
                                            comment_dict.set_item("line", line)?;
                                        } else {
                                            comment_dict.set_item("line", py.None())?;
                                        }

                                        if let Some(commit_sha) = &comment.commit_sha {
                                            comment_dict.set_item("commit_sha", commit_sha)?;
                                        } else {
                                            comment_dict.set_item("commit_sha", py.None())?;
                                        }

                                        py_comments_list.append(comment_dict)?;
                                    }

                                    py_result_dict.set_item(repo_url, py_comments_list)?;
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

// --- Exposed Python Class: TaigaClient ---
#[pyclass(name = "TaigaClient", module = "gradelib")]
#[derive(Debug, Clone)]
pub struct TaigaClient {
    pub inner: client::TaigaClient,
}

#[pymethods]
impl TaigaClient {
    #[new]
    #[pyo3(signature = (base_url, auth_token=None, username=None))]
    fn new(base_url: String, auth_token: Option<String>, username: Option<String>) -> Self {
        let config = client::TaigaClientConfig {
            base_url,
            // Use empty string if auth_token is None
            auth_token: auth_token.unwrap_or_default(),
            // Use empty string if username is None
            username: username.unwrap_or_default(),
        };
        let taiga_client = client::TaigaClient::new(config);
        Self {
            inner: taiga_client,
        }
    }

    fn fetch_project_data<'py>(
        &self,
        py: Python<'py>,
        slug: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();

        tokio::future_into_py(py, async move {
            let result = orchestrator::fetch_complete_project_data(&client, &slug).await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(project_data) => {
                        let py_result = PyDict::new(py);

                        // Project info
                        let project_dict = PyDict::new(py);
                        project_dict.set_item("id", project_data.project.id)?;
                        project_dict.set_item("name", &project_data.project.name)?;
                        project_dict.set_item("slug", &project_data.project.slug)?;
                        project_dict.set_item("description", &project_data.project.description)?;
                        project_dict
                            .set_item("created_date", &project_data.project.created_date)?;
                        project_dict
                            .set_item("modified_date", &project_data.project.modified_date)?;
                        py_result.set_item("project", project_dict)?;

                        // Members
                        let members_list = PyList::empty(py);
                        for member in project_data.members {
                            let member_dict = PyDict::new(py);
                            member_dict.set_item("id", member.id)?;
                            member_dict.set_item("user", member.user)?;
                            member_dict.set_item("role", member.role)?;
                            member_dict.set_item("role_name", member.role_name)?;
                            member_dict.set_item("full_name", member.full_name)?;
                            members_list.append(member_dict)?;
                        }
                        py_result.set_item("members", members_list)?;

                        // Sprints
                        let sprints_list = PyList::empty(py);
                        for sprint in project_data.sprints {
                            let sprint_dict = PyDict::new(py);
                            sprint_dict.set_item("id", sprint.id)?;
                            sprint_dict.set_item("name", sprint.name)?;
                            sprint_dict.set_item("estimated_start", sprint.estimated_start)?;
                            sprint_dict.set_item("estimated_finish", sprint.estimated_finish)?;
                            sprint_dict.set_item("created_date", sprint.created_date)?;
                            sprint_dict.set_item("closed", sprint.closed)?;
                            sprints_list.append(sprint_dict)?;
                        }
                        py_result.set_item("sprints", sprints_list)?;

                        // User stories
                        let user_stories_dict = PyDict::new(py);
                        for (sprint_id, stories) in project_data.user_stories {
                            let stories_list = PyList::empty(py);
                            for story in stories {
                                let story_dict = PyDict::new(py);
                                story_dict.set_item("id", story.id)?;
                                story_dict.set_item("reference", story.reference)?;
                                story_dict.set_item("subject", story.subject)?;
                                story_dict.set_item("status", story.status)?;
                                stories_list.append(story_dict)?;
                            }
                            user_stories_dict.set_item(sprint_id.to_string(), stories_list)?;
                        }
                        py_result.set_item("user_stories", user_stories_dict)?;

                        // Tasks
                        let tasks_dict = PyDict::new(py);
                        for (sprint_id, tasks) in project_data.tasks {
                            let tasks_list = PyList::empty(py);
                            for task in tasks {
                                let task_dict = PyDict::new(py);
                                task_dict.set_item("id", task.id)?;
                                task_dict.set_item("reference", task.reference)?;
                                task_dict.set_item("subject", task.subject)?;
                                task_dict.set_item("is_closed", task.is_closed)?;
                                if let Some(assigned_to) = task.assigned_to {
                                    task_dict.set_item("assigned_to", assigned_to)?;
                                } else {
                                    task_dict.set_item("assigned_to", py.None())?;
                                }
                                tasks_list.append(task_dict)?;
                            }
                            tasks_dict.set_item(sprint_id.to_string(), tasks_list)?;
                        }
                        py_result.set_item("tasks", tasks_dict)?;

                        // Task histories
                        let histories_dict = PyDict::new(py);
                        for (task_id, events) in project_data.task_histories {
                            let events_list = PyList::empty(py);
                            for event in events {
                                let event_dict = PyDict::new(py);
                                event_dict.set_item("id", event.id)?;
                                event_dict.set_item("created_at", event.created_at)?;
                                event_dict.set_item("event_type", event.event_type)?;
                                events_list.append(event_dict)?;
                            }
                            histories_dict.set_item(task_id.to_string(), events_list)?;
                        }
                        py_result.set_item("task_histories", histories_dict)?;

                        Ok(py_result.into())
                    }
                    Err(e) => {
                        // Convert the error to a Python exception
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Failed to fetch Taiga project data: {}",
                            e
                        )))
                    }
                }
            })
        })
    }

    fn fetch_multiple_projects<'py>(
        &self,
        py: Python<'py>,
        slugs: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();

        tokio::future_into_py(py, async move {
            let result = orchestrator::fetch_taiga_data_concurrently(&client, slugs).await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(projects_map) => {
                        let py_result = PyDict::new(py);

                        for (slug, project_result) in projects_map {
                            match project_result {
                                Ok(_) => {
                                    // Success indicator
                                    py_result.set_item(&slug, true)?;
                                }
                                Err(e) => {
                                    // Error message
                                    py_result.set_item(&slug, format!("Error: {}", e))?;
                                }
                            }
                        }

                        Ok(py_result.into())
                    }
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to fetch Taiga projects: {}",
                        e
                    ))),
                }
            })
        })
    }
}

/// Registers the Taiga module
fn register_taiga_module(py: Python<'_>, parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new(py, "taiga")?;
    m.add_class::<TaigaClient>()?;
    parent_module.add_submodule(&m)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("gradelib.taiga", m)?;
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

    // Also expose TaigaClient directly in the root module
    m.add_class::<TaigaClient>()?;

    // Register the Taiga module
    register_taiga_module(_py, m)?;

    m.add_class::<GitHubOAuthClient>()?;

    Ok(())
}
