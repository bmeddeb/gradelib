use futures::future::join_all;
use git2::{Cred, FetchOptions, Progress, RemoteCallbacks};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tempfile::TempDir;
use tokio::task::JoinHandle; // For spawn_blocking handle type // Keep regex crate

// --- Import from new modules ---
use crate::blame::{get_blame_for_file, BlameLineInfo};
use crate::clone::{InternalCloneStatus, InternalRepoCloneTask};
use crate::commits::{extract_commits_parallel, CommitInfo}; // Use the new parallel function

// --- Internal Data Structures ---

// Main struct holding the application state and logic (internal)
#[derive(Clone)] // Derives the Clone trait method clone(&self) -> Self
pub struct InternalRepoManagerLogic {
    // Stores clone tasks, keyed by repository URL
    pub tasks: Arc<Mutex<HashMap<String, InternalRepoCloneTask>>>,
    // GitHub credentials used for cloning
    pub github_username: String,
    pub github_token: String,
}

// --- Helper Functions ---

lazy_static! {
    // Regex for HTTPS: captures 'owner/repo' from https://github.com/owner/repo.git or https://host.com/owner/repo
    static ref RE_HTTPS: Regex = Regex::new(r"https?://[^/]+/(?P<slug>[^/]+/[^/.]+?)(\.git)?/?$").unwrap();
    // Regex for SSH: captures 'owner/repo' from git@github.com:owner/repo.git or user@host:owner/repo
    static ref RE_SSH: Regex = Regex::new(r"^(?:ssh://)?git@.*?:(?P<slug>[^/]+/[^/.]+?)(\.git)?$").unwrap();
}

/// Parses a repository slug (e.g., "owner/repo") from common Git URLs.
/// Moved outside the impl block.
pub fn parse_slug_from_url(url: &str) -> Option<String> {
    if let Some(caps) = RE_HTTPS.captures(url) {
        caps.name("slug").map(|m| m.as_str().to_string())
    } else if let Some(caps) = RE_SSH.captures(url) {
        caps.name("slug").map(|m| m.as_str().to_string())
    } else {
        None // URL format not recognized
    }
}

// --- Core Logic Implementation for InternalRepoManagerLogic ---

impl InternalRepoManagerLogic {
    /// Creates a new instance of the internal manager logic.
    pub fn new(urls: &[&str], github_username: &str, github_token: &str) -> Self {
        // Initialize lazy_static regexes here if not already done
        lazy_static::initialize(&RE_HTTPS);
        lazy_static::initialize(&RE_SSH);

        let tasks = urls
            .iter()
            .map(|&url| {
                (
                    url.to_string(),
                    InternalRepoCloneTask {
                        url: url.to_string(),
                        status: InternalCloneStatus::Queued,
                        temp_dir: None,
                    },
                )
            })
            .collect();

        Self {
            tasks: Arc::new(Mutex::new(tasks)),
            github_username: github_username.to_string(),
            github_token: github_token.to_string(),
        }
    }

    /// Initiates cloning for all repositories managed by this instance.
    pub async fn clone_all(&self) -> HashMap<String, Result<PathBuf, String>> {
        let task_urls = {
            let tasks_guard = self.tasks.lock().unwrap();
            tasks_guard.keys().cloned().collect::<Vec<_>>()
        };
        let results = join_all(task_urls.iter().cloned().map(|url| self.clone(url))).await;
        let mut map = HashMap::new();
        for ((result, _url), original_url) in results.into_iter().zip(task_urls.into_iter()) {
            map.insert(original_url, result);
        }
        map
    }

    /// Clones a single repository specified by URL.
    pub async fn clone(&self, url: String) -> (Result<PathBuf, String>, String) {
        self.update_status(&url, InternalCloneStatus::Cloning(0))
            .await;
        let manager_logic = Clone::clone(self);
        let username = self.github_username.clone();
        let token = self.github_token.clone();
        let url_clone = url.clone();
        let result: Result<Result<PathBuf, String>, tokio::task::JoinError> =
            tokio::task::spawn_blocking(move || {
                let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
                let temp_path = temp_dir.path().to_path_buf();
                let mut callbacks = RemoteCallbacks::new();
                let username_cb = username.clone();
                let token_cb = token.clone();
                callbacks.credentials(move |url, username_from_url, _allowed_types| {
                    // Log auth attempt for debugging
                    eprintln!("Git authentication attempt for URL: {}", url);
                    if let Some(user) = username_from_url {
                        eprintln!("Username from URL: {}", user);
                    }

                    // Determine which username to use
                    let effective_username = if username_cb.is_empty() {
                        // Use "git" as fallback username for GitHub URLs
                        if url.contains("github.com") {
                            "git"
                        } else {
                            // For non-GitHub URLs, try with the URL-provided username if available
                            username_from_url.unwrap_or("")
                        }
                    } else {
                        // Use the provided username
                        &username_cb
                    };

                    Cred::userpass_plaintext(effective_username, &token_cb)
                });
                let tasks = Arc::clone(&manager_logic.tasks);
                let url_str = url.clone();
                callbacks.transfer_progress(move |stats: Progress| {
                    let percent = ((stats.received_objects() as f32
                        / stats.total_objects().max(1) as f32)
                        * 100.0) as u8;
                    if let Ok(mut tasks_guard) = tasks.lock() {
                        if let Some(task) = tasks_guard.get_mut(&url_str) {
                            task.status = InternalCloneStatus::Cloning(percent);
                        }
                    }
                    true
                });
                let mut fetch_options = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);
                let mut builder = git2::build::RepoBuilder::new();
                builder.fetch_options(fetch_options);
                match builder.clone(&url, &temp_path) {
                    Ok(_repo) => Ok(temp_dir.into_path()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;
        let ret = match result {
            Ok(Ok(path)) => {
                self.update_status(&url_clone, InternalCloneStatus::Cloning(100))
                    .await;
                self.finalize_success(&url_clone, path.clone()).await;
                Ok(path)
            }
            Ok(Err(err_string)) => {
                self.update_status(&url_clone, InternalCloneStatus::Failed(err_string.clone()))
                    .await;
                Err(err_string)
            }
            Err(join_err) => {
                self.update_status(
                    &url_clone,
                    InternalCloneStatus::Failed(format!("Cloning task failed: {}", join_err)),
                )
                .await;
                Err(format!("Cloning task failed: {}", join_err))
            }
        };
        (ret, url_clone)
    }

    /// Updates the status of a specific clone task. Internal helper.
    async fn update_status(&self, url: &str, status: InternalCloneStatus) {
        let mut tasks_guard = self.tasks.lock().unwrap();
        if let Some(task) = tasks_guard.get_mut(url) {
            task.status = status;
        }
    }

    /// Marks a task as completed and stores its temporary directory path. Internal helper.
    async fn finalize_success(&self, url: &str, path: PathBuf) {
        let mut tasks_guard = self.tasks.lock().unwrap();
        if let Some(task) = tasks_guard.get_mut(url) {
            task.status = InternalCloneStatus::Completed;
            task.temp_dir = Some(path);
        }
    }

    /// Retrieves the current state of all managed clone tasks.
    pub async fn get_internal_tasks(&self) -> HashMap<String, InternalRepoCloneTask> {
        // Clone the HashMap to release the lock quickly
        self.tasks.lock().unwrap().clone()
    }

    /// Performs git blame concurrently on multiple files within a specified repository.
    pub async fn bulk_blame(
        &self,
        repo_path: &PathBuf,
        file_paths: Vec<String>,
    ) -> Result<HashMap<String, Result<Vec<BlameLineInfo>, String>>, String> {
        // 2. Create futures for each file's blame operation run via spawn_blocking
        let mut blame_futures = Vec::new();
        for file_path in file_paths {
            let repo_path_clone = repo_path.clone();
            let file_path_clone = file_path.clone();
            let handle: JoinHandle<Result<Vec<BlameLineInfo>, String>> =
                tokio::task::spawn_blocking(move || {
                    get_blame_for_file(&repo_path_clone, &file_path_clone)
                });
            blame_futures.push(async move { (file_path, handle.await) });
        }
        let joined_results = join_all(blame_futures).await;
        let mut final_results: HashMap<String, Result<Vec<BlameLineInfo>, String>> = HashMap::new();
        for (file_path, join_result) in joined_results {
            match join_result {
                Ok(blame_result) => {
                    final_results.insert(file_path, blame_result);
                }
                Err(join_error) => {
                    final_results.insert(
                        file_path,
                        Err(format!("Blame task execution failed: {}", join_error)),
                    );
                }
            }
        }
        Ok(final_results)
    }

    /// Analyzes the commit history of a cloned repository using parallel processing.
    /// This method is synchronous internally but designed to be called from an async context.
    pub fn get_commit_analysis(&self, repo_path: &PathBuf) -> Result<Vec<CommitInfo>, String> {
        extract_commits_parallel(repo_path.clone(), String::new())
    }
}
