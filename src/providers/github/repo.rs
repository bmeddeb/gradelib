use futures::future::join_all;
use git2::{Cred, FetchOptions, Progress, RemoteCallbacks};
use std::{
    collections::HashMap,
    path::{PathBuf},
    sync::{Arc, Mutex},
};
use tempfile::TempDir;
use tokio::task::JoinHandle; // For spawn_blocking handle type
use lazy_static::lazy_static;
use regex::Regex; // Keep regex crate

// --- Import from new modules ---
use crate::blame::{get_blame_for_file, BlameLineInfo};
use crate::clone::{InternalCloneStatus, InternalRepoCloneTask};
use crate::commits::{CommitInfo, extract_commits_parallel}; // Use the new parallel function

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
    pub async fn clone_all(&self) {
        let task_urls = {
            // Clone URLs to avoid holding lock during async operations
            let tasks_guard = self.tasks.lock().unwrap();
            tasks_guard.keys().cloned().collect::<Vec<_>>()
        };
        // Create a future for each clone operation and run them concurrently
        join_all(task_urls.into_iter().map(|url| self.clone(url))).await;
    }

    /// Clones a single repository specified by URL.
    pub async fn clone(&self, url: String) {
        // Set status to Cloning(0) immediately
        self.update_status(&url, InternalCloneStatus::Cloning(0))
            .await;

        // Clone necessary data for the blocking task closure
        // Cloning the struct itself clones the Arc and Strings efficiently
        let manager_logic = Clone::clone(self);
        let username = self.github_username.clone();
        let token = self.github_token.clone();
        let url_clone = url.clone(); // Clone URL for final status updates

        // Spawn the potentially long-running git operation onto Tokio's blocking thread pool
        let result: Result<Result<PathBuf, String>, tokio::task::JoinError> =
            tokio::task::spawn_blocking(move || {
                // --- This closure runs on a blocking thread ---
                let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
                let temp_path = temp_dir.path().to_path_buf();

                // Setup remote callbacks for authentication and progress
                let mut callbacks = RemoteCallbacks::new();
                let username_cb = username.clone(); // Clone for credential closure
                let token_cb = token.clone(); // Clone for credential closure
                callbacks
                    .credentials(move |_, _, _| Cred::userpass_plaintext(&username_cb, &token_cb));

                // Progress reporting setup
                // Using a simple struct with atomic values for thread-safe communication
                let tasks = Arc::clone(&manager_logic.tasks);
                let url_str = url.clone();

                callbacks.transfer_progress(move |stats: Progress| {
                    let percent = ((stats.received_objects() as f32
                        / stats.total_objects().max(1) as f32)
                        * 100.0) as u8;
                    // Directly update the status in the shared tasks HashMap
                    if let Ok(mut tasks_guard) = tasks.lock() {
                        if let Some(task) = tasks_guard.get_mut(&url_str) {
                            task.status = InternalCloneStatus::Cloning(percent);
                        }
                    }
                    true // Continue transfer
                });

                // Configure fetch options with callbacks
                let mut fetch_options = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);

                // Configure and run the clone operation
                let mut builder = git2::build::RepoBuilder::new();
                builder.fetch_options(fetch_options);

                match builder.clone(&url, &temp_path) {
                    Ok(_repo) => Ok(temp_dir.into_path()), // Keep temp_dir alive by returning its path
                    Err(e) => Err(e.to_string()),
                }
                // --- End of blocking closure ---
            })
            .await; // Wait for the blocking task to complete

        // Process the result of the blocking task
        match result {
            Ok(Ok(path)) => {
                // spawn_blocking succeeded, clone operation succeeded
                // Set final status to 100% cloning before Completed, looks better
                self.update_status(&url_clone, InternalCloneStatus::Cloning(100))
                    .await;
                self.finalize_success(&url_clone, path).await;
            }
            Ok(Err(err_string)) => {
                // spawn_blocking succeeded, clone operation failed
                self.update_status(&url_clone, InternalCloneStatus::Failed(err_string))
                    .await;
            }
            Err(join_err) => {
                // spawn_blocking task itself failed (e.g., panicked)
                self.update_status(
                    &url_clone,
                    InternalCloneStatus::Failed(format!("Cloning task failed: {}", join_err)),
                )
                .await;
            }
        }
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
        target_repo_url: &str,
        file_paths: Vec<String>,
    ) -> Result<HashMap<String, Result<Vec<BlameLineInfo>, String>>, String> {
        // 1. Find the completed clone task and get its repository path
        let repo_path = {
            // Scope for the mutex guard
            let tasks_guard = self.tasks.lock().unwrap();
            let task = tasks_guard
                .get(target_repo_url)
                .ok_or_else(|| format!("Repository '{}' not managed.", target_repo_url))?;

            match &task.status {
                InternalCloneStatus::Completed => task.temp_dir.clone().ok_or_else(|| {
                    format!(
                        "Repository '{}' completed but temp_dir missing.",
                        target_repo_url
                    )
                })?,
                _ => {
                    return Err(format!(
                        "Repository '{}' not in 'Completed' state.",
                        target_repo_url
                    ))
                }
            }
        }; // Mutex guard dropped here

        // 2. Create futures for each file's blame operation run via spawn_blocking
        let mut blame_futures = Vec::new();
        for file_path in file_paths {
            // Iterate over original list to preserve association
            let repo_path_clone = repo_path.clone(); // Clone PathBuf for the closure
            let file_path_clone = file_path.clone(); // Clone file path for the closure

            // Spawn the blocking function, get a JoinHandle
            let handle: JoinHandle<Result<Vec<BlameLineInfo>, String>> =
                tokio::task::spawn_blocking(move || {
                    get_blame_for_file(&repo_path_clone, &file_path_clone)
                });

            // Create an async block to await the handle and pair it with the original file path
            blame_futures.push(async move {
                (file_path, handle.await) // Await the JoinHandle result here
            });
        }

        // 3. Run all blame operations concurrently and wait for results
        let joined_results = join_all(blame_futures).await;

        // 4. Process the results into the final HashMap
        let mut final_results: HashMap<String, Result<Vec<BlameLineInfo>, String>> = HashMap::new();
        for (file_path, join_result) in joined_results {
            match join_result {
                Ok(blame_result) => {
                    // spawn_blocking finished, inner result is Result<Vec<BlameLineInfo>, String>
                    final_results.insert(file_path, blame_result);
                }
                Err(join_error) => {
                    // spawn_blocking task itself panicked or was cancelled
                    final_results.insert(
                        file_path,
                        Err(format!("Blame task execution failed: {}", join_error)),
                    );
                }
            }
        }

        // 5. Return the map of results
        Ok(final_results)
    }

    /// Analyzes the commit history of a cloned repository using parallel processing.
    /// This method is synchronous internally but designed to be called from an async context.
    pub fn get_commit_analysis(
        &self,
        target_repo_url: &str,
    ) -> Result<Vec<CommitInfo>, String> {
        let repo_path = {
            let tasks = self.tasks.lock().unwrap();
            let task = tasks
                .get(target_repo_url)
                .ok_or_else(|| format!("Repository '{}' not managed.", target_repo_url))?;

            match &task.status {
                InternalCloneStatus::Completed => task.temp_dir.clone().ok_or_else(|| {
                    format!(
                        "Repository '{}' completed but temp_dir missing.",
                        target_repo_url
                    )
                })?,
                _ => {
                    return Err(format!(
                        "Repository '{}' is not successfully cloned for commit analysis.",
                        target_repo_url
                    ));
                }
            }
        };

        // 👇 Use full path or URL as display name; don't try to parse it
        extract_commits_parallel(repo_path, target_repo_url.to_string())
    }

}
