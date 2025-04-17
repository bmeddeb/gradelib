use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3_async_runtimes::tokio;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::common::traits::*;
use crate::common::types::*;

pub struct GitHubProvider {
    pub username: String,
    pub token: String,
    pub tasks: Arc<Mutex<HashMap<String, RepoCloneTask>>>,
}

impl GitHubProvider {
    pub fn new(username: String, token: String) -> Self {
        Self {
            username,
            token,
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_repos(username: String, token: String, urls: &[&str]) -> Self {
        let mut tasks = HashMap::new();
        for &url in urls {
            tasks.insert(
                url.to_string(),
                RepoCloneTask {
                    url: url.to_string(),
                    status: CloneStatus::Queued,
                    temp_dir: None,
                },
            );
        }

        Self {
            username,
            token,
            tasks: Arc::new(Mutex::new(tasks)),
        }
    }
}

#[async_trait]
impl Provider for GitHubProvider {
    fn name(&self) -> &str {
        "github"
    }

    fn get_credentials(&self) -> ProviderCredentials {
        ProviderCredentials::Basic {
            username: self.username.clone(),
            token: self.token.clone(),
        }
    }
}

// Python bindings for GitHubProvider
#[pyclass(name = "GitHubProvider")]
#[derive(Clone)]
pub struct GitHubProviderPy {
    inner: Arc<GitHubProvider>,
}

#[pymethods]
impl GitHubProviderPy {
    #[new]
    fn new(username: String, token: String, urls: Option<Vec<String>>) -> Self {
        if let Some(urls) = urls {
            let string_urls: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
            Self {
                inner: Arc::new(GitHubProvider::with_repos(username, token, &string_urls)),
            }
        } else {
            Self {
                inner: Arc::new(GitHubProvider::new(username, token)),
            }
        }
    }

    #[pyo3(name = "clone_all")]
    fn clone_all<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            match inner.clone_all(&[]).await {
                Ok(_) => Python::with_gil(|py| Ok(py.None())),
                Err(e) => {
                    Python::with_gil(|_py| Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)))
                }
            }
        })
    }

    #[pyo3(name = "clone")]
    fn clone_repo<'py>(&self, py: Python<'py>, url: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        let url_clone = url.clone();
        tokio::future_into_py(py, async move {
            let result = inner.clone_repo(&url_clone).await;
            match result {
                Ok(_) => Python::with_gil(|py| Ok(py.None())),
                Err(e) => {
                    Python::with_gil(|_py| Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)))
                }
            }
        })
    }

    #[pyo3(name = "fetch_clone_tasks")]
    fn fetch_clone_tasks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            let tasks = inner.get_clone_tasks().await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                let dict = PyDict::new(py);
                for (k, v) in tasks {
                    let task_dict = PyDict::new(py);

                    // Add task properties to the dictionary
                    task_dict.set_item("url", k.clone())?;

                    // Convert status to Python-friendly format
                    match v.status {
                        CloneStatus::Queued => {
                            task_dict.set_item("status_type", "queued")?;
                            task_dict.set_item("progress", py.None())?;
                            task_dict.set_item("error", py.None())?;
                        }
                        CloneStatus::Cloning(progress) => {
                            task_dict.set_item("status_type", "cloning")?;
                            task_dict.set_item("progress", progress)?;
                            task_dict.set_item("error", py.None())?;
                        }
                        CloneStatus::Completed => {
                            task_dict.set_item("status_type", "completed")?;
                            task_dict.set_item("progress", py.None())?;
                            task_dict.set_item("error", py.None())?;
                        }
                        CloneStatus::Failed(ref error) => {
                            task_dict.set_item("status_type", "failed")?;
                            task_dict.set_item("progress", py.None())?;
                            task_dict.set_item("error", error)?;
                        }
                    }

                    // Add temp_dir if available
                    if let Some(dir) = &v.temp_dir {
                        task_dict.set_item("temp_dir", dir.to_string_lossy().to_string())?;
                    } else {
                        task_dict.set_item("temp_dir", py.None())?;
                    }

                    dict.set_item(k, task_dict)?;
                }

                Ok(dict.into_py(py))
            })
        })
    }

    #[pyo3(name = "analyze_commits")]
    fn analyze_commits<'py>(
        &self,
        py: Python<'py>,
        target_repo_url: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            let result = inner.analyze_commits(&target_repo_url).await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(commits) => {
                        let py_list = PyList::empty(py);

                        for commit in commits {
                            let commit_dict = PyDict::new(py);
                            commit_dict.set_item("sha", commit.sha)?;
                            commit_dict.set_item("repo_name", commit.repo_name)?;
                            commit_dict.set_item("message", commit.message)?;
                            commit_dict.set_item("author_name", commit.author_name)?;
                            commit_dict.set_item("author_email", commit.author_email)?;
                            commit_dict.set_item("author_timestamp", commit.author_timestamp)?;
                            commit_dict.set_item("author_offset", commit.author_offset)?;
                            commit_dict.set_item("committer_name", commit.committer_name)?;
                            commit_dict.set_item("committer_email", commit.committer_email)?;
                            commit_dict
                                .set_item("committer_timestamp", commit.committer_timestamp)?;
                            commit_dict.set_item("committer_offset", commit.committer_offset)?;
                            commit_dict.set_item("additions", commit.additions)?;
                            commit_dict.set_item("deletions", commit.deletions)?;
                            commit_dict.set_item("is_merge", commit.is_merge)?;
                            py_list.append(commit_dict)?;
                        }

                        Ok(py_list.into_py(py))
                    }
                    Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err)),
                }
            })
        })
    }

    #[pyo3(name = "bulk_blame")]
    fn bulk_blame<'py>(
        &self,
        py: Python<'py>,
        target_repo_url: String,
        file_paths: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            let result = inner.bulk_blame(&target_repo_url, file_paths).await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(blame_results_map) => {
                        let py_result_dict = PyDict::new(py);

                        for (file_path, blame_result) in blame_results_map {
                            match blame_result {
                                Ok(blame_lines) => {
                                    let py_blame_list = PyList::empty(py);
                                    for line_info in blame_lines {
                                        let line_dict = PyDict::new(py);
                                        line_dict.set_item("commit_id", line_info.commit_id)?;
                                        line_dict.set_item("author_name", line_info.author_name)?;
                                        line_dict
                                            .set_item("author_email", line_info.author_email)?;
                                        line_dict
                                            .set_item("orig_line_no", line_info.orig_line_no)?;
                                        line_dict
                                            .set_item("final_line_no", line_info.final_line_no)?;
                                        line_dict
                                            .set_item("line_content", line_info.line_content)?;
                                        py_blame_list.append(line_dict)?;
                                    }
                                    py_result_dict.set_item(file_path, py_blame_list)?;
                                }
                                Err(err_string) => {
                                    py_result_dict.set_item(file_path, err_string)?;
                                }
                            }
                        }

                        Ok(py_result_dict.into_py(py))
                    }
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    #[pyo3(name = "analyze_branches")]
    fn analyze_branches<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            let result = inner.analyze_branches(repo_urls.clone()).await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                let py_result_dict = PyDict::new(py);

                for (repo_url, branch_result) in result {
                    match branch_result {
                        Ok(branches) => {
                            let py_branch_list = PyList::empty(py);

                            for branch in branches {
                                let branch_dict = PyDict::new(py);
                                branch_dict.set_item("name", branch.name)?;
                                branch_dict.set_item("is_remote", branch.is_remote)?;
                                branch_dict.set_item("commit_id", branch.commit_id)?;
                                branch_dict.set_item("commit_message", branch.commit_message)?;
                                branch_dict.set_item("author_name", branch.author_name)?;
                                branch_dict.set_item("author_email", branch.author_email)?;
                                branch_dict.set_item("author_time", branch.author_time)?;
                                branch_dict.set_item("is_head", branch.is_head)?;

                                if let Some(remote) = branch.remote_name {
                                    branch_dict.set_item("remote_name", remote)?;
                                } else {
                                    branch_dict.set_item("remote_name", py.None())?;
                                }

                                py_branch_list.append(branch_dict)?;
                            }

                            py_result_dict.set_item(repo_url, py_branch_list)?;
                        }
                        Err(error) => {
                            py_result_dict.set_item(repo_url, error)?;
                        }
                    }
                }

                Ok(py_result_dict.into_py(py))
            })
        })
    }

    #[pyo3(name = "fetch_collaborators")]
    fn fetch_collaborators<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            let result = inner.fetch_collaborators(repo_urls).await;

            Python::with_gil(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Ok(collab_map) => {
                        let py_result_dict = PyDict::new(py);

                        for (repo_url, collaborators) in collab_map {
                            let py_collab_list = PyList::empty(py);

                            for collab in collaborators {
                                let collab_dict = PyDict::new(py);
                                collab_dict.set_item("login", collab.login)?;
                                collab_dict.set_item("github_id", collab.github_id)?;

                                if let Some(name) = collab.full_name {
                                    collab_dict.set_item("full_name", name)?;
                                } else {
                                    collab_dict.set_item("full_name", py.None())?;
                                }

                                if let Some(email) = collab.email {
                                    collab_dict.set_item("email", email)?;
                                } else {
                                    collab_dict.set_item("email", py.None())?;
                                }

                                if let Some(avatar) = collab.avatar_url {
                                    collab_dict.set_item("avatar_url", avatar)?;
                                } else {
                                    collab_dict.set_item("avatar_url", py.None())?;
                                }

                                py_collab_list.append(collab_dict)?;
                            }

                            py_result_dict.set_item(repo_url, py_collab_list)?;
                        }

                        Ok(py_result_dict.into_py(py))
                    }
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    #[pyo3(name = "fetch_issues")]
    fn fetch_issues<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        state: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            let result = inner.fetch_issues(repo_urls, state.as_deref()).await;

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
                                        issue_dict.set_item("title", issue.title)?;
                                        issue_dict.set_item("state", issue.state)?;
                                        issue_dict.set_item("created_at", issue.created_at)?;
                                        issue_dict.set_item("updated_at", issue.updated_at)?;

                                        if let Some(closed_at) = issue.closed_at {
                                            issue_dict.set_item("closed_at", closed_at)?;
                                        } else {
                                            issue_dict.set_item("closed_at", py.None())?;
                                        }

                                        issue_dict.set_item("user_login", issue.user_login)?;
                                        issue_dict.set_item("user_id", issue.user_id)?;

                                        if let Some(body) = issue.body {
                                            issue_dict.set_item("body", body)?;
                                        } else {
                                            issue_dict.set_item("body", py.None())?;
                                        }

                                        issue_dict
                                            .set_item("comments_count", issue.comments_count)?;
                                        issue_dict
                                            .set_item("is_pull_request", issue.is_pull_request)?;
                                        issue_dict.set_item("labels", issue.labels)?;
                                        issue_dict.set_item("assignees", issue.assignees)?;

                                        if let Some(milestone) = issue.milestone {
                                            issue_dict.set_item("milestone", milestone)?;
                                        } else {
                                            issue_dict.set_item("milestone", py.None())?;
                                        }

                                        issue_dict.set_item("locked", issue.locked)?;
                                        issue_dict.set_item("html_url", issue.html_url)?;

                                        py_issue_list.append(issue_dict)?;
                                    }

                                    py_result_dict.set_item(repo_url, py_issue_list)?;
                                }
                                Err(error) => {
                                    py_result_dict.set_item(repo_url, error)?;
                                }
                            }
                        }

                        Ok(py_result_dict.into_py(py))
                    }
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }

    #[pyo3(name = "fetch_pull_requests")]
    fn fetch_pull_requests<'py>(
        &self,
        py: Python<'py>,
        repo_urls: Vec<String>,
        state: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::clone(&self.inner);
        tokio::future_into_py(py, async move {
            let result = inner.fetch_pull_requests(repo_urls, state.as_deref()).await;

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
                                        pr_dict.set_item("title", pr.title)?;
                                        pr_dict.set_item("state", pr.state)?;
                                        pr_dict.set_item("created_at", pr.created_at)?;
                                        pr_dict.set_item("updated_at", pr.updated_at)?;

                                        if let Some(closed_at) = pr.closed_at {
                                            pr_dict.set_item("closed_at", closed_at)?;
                                        } else {
                                            pr_dict.set_item("closed_at", py.None())?;
                                        }

                                        if let Some(merged_at) = pr.merged_at {
                                            pr_dict.set_item("merged_at", merged_at)?;
                                        } else {
                                            pr_dict.set_item("merged_at", py.None())?;
                                        }

                                        pr_dict.set_item("user_login", pr.user_login)?;
                                        pr_dict.set_item("user_id", pr.user_id)?;

                                        if let Some(body) = pr.body {
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

                                        pr_dict.set_item("labels", pr.labels)?;
                                        pr_dict.set_item("is_draft", pr.draft)?;
                                        pr_dict.set_item("merged", pr.merged)?;

                                        if let Some(merged_by) = pr.merged_by {
                                            pr_dict.set_item("merged_by", merged_by)?;
                                        } else {
                                            pr_dict.set_item("merged_by", py.None())?;
                                        }

                                        py_pr_list.append(pr_dict)?;
                                    }

                                    py_result_dict.set_item(repo_url, py_pr_list)?;
                                }
                                Err(error) => {
                                    py_result_dict.set_item(repo_url, error)?;
                                }
                            }
                        }

                        Ok(py_result_dict.into_py(py))
                    }
                    Err(err_string) => {
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(err_string))
                    }
                }
            })
        })
    }
}
