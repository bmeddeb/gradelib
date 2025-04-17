use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use tokio::task;

use crate::common::traits::RepoOperations;
use crate::common::types::{BlameLineInfo, CloneStatus, RepoCloneTask};
use crate::github::provider::GitHubProvider;

#[async_trait]
impl RepoOperations for GitHubProvider {
    async fn clone_repo(&self, url: &str) -> Result<(), String> {
        // Check if we're already tracking this repo
        {
            let mut tasks = self.tasks.lock().unwrap();
            if !tasks.contains_key(url) {
                tasks.insert(
                    url.to_string(),
                    RepoCloneTask {
                        url: url.to_string(),
                        status: CloneStatus::Queued,
                        temp_dir: None,
                    },
                );
            } else {
                // If we already have this repo and it's completed, we don't need to clone again
                if let CloneStatus::Completed = tasks.get(url).unwrap().status {
                    return Ok(());
                }
            }
        }

        // Start clone process in a background task
        let url_owned = url.to_string();
        let tasks_clone = Arc::clone(&self.tasks);
        let github_username = self.username.clone();
        let github_token = self.token.clone();

        task::spawn(async move {
            // Create a temporary directory to clone into
            let temp_dir = match tempdir() {
                Ok(dir) => dir,
                Err(e) => {
                    let mut tasks = tasks_clone.lock().unwrap();
                    if let Some(task) = tasks.get_mut(&url_owned) {
                        task.status =
                            CloneStatus::Failed(format!("Failed to create temp dir: {}", e));
                    }
                    return;
                }
            };

            // Update status to cloning
            {
                let mut tasks = tasks_clone.lock().unwrap();
                if let Some(task) = tasks.get_mut(&url_owned) {
                    task.status = CloneStatus::Cloning(0);
                    task.temp_dir = Some(temp_dir.path().to_path_buf());
                }
            }

            // Prepare URL with authentication
            let mut authenticated_url = String::new();
            if url_owned.starts_with("https://github.com/") {
                authenticated_url = format!(
                    "https://{}:{}@github.com/{}",
                    github_username,
                    github_token,
                    url_owned.strip_prefix("https://github.com/").unwrap()
                );
            } else {
                authenticated_url = url_owned.clone();
            }

            // Execute git clone command
            let output = Command::new("git")
                .arg("clone")
                .arg("--progress")
                .arg(&authenticated_url)
                .arg(temp_dir.path())
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        // Clone succeeded
                        let mut tasks = tasks_clone.lock().unwrap();
                        if let Some(task) = tasks.get_mut(&url_owned) {
                            task.status = CloneStatus::Completed;
                            // Keep the temp_dir field so we don't drop the tempdir
                        }
                    } else {
                        // Clone failed
                        let error = String::from_utf8_lossy(&output.stderr).to_string();
                        let mut tasks = tasks_clone.lock().unwrap();
                        if let Some(task) = tasks.get_mut(&url_owned) {
                            task.status = CloneStatus::Failed(error);
                        }
                    }
                }
                Err(e) => {
                    // Failed to execute command
                    let mut tasks = tasks_clone.lock().unwrap();
                    if let Some(task) = tasks.get_mut(&url_owned) {
                        task.status =
                            CloneStatus::Failed(format!("Failed to execute git clone: {}", e));
                    }
                }
            }
        });

        Ok(())
    }

    async fn clone_all(&self, urls: &[&str]) -> Result<(), String> {
        for url in urls {
            self.clone_repo(url).await?;
        }
        Ok(())
    }

    async fn get_clone_tasks(&self) -> HashMap<String, RepoCloneTask> {
        let tasks = self.tasks.lock().unwrap();
        tasks.clone()
    }

    async fn bulk_blame(
        &self,
        target_repo_url: &str,
        file_paths: Vec<String>,
    ) -> Result<HashMap<String, Result<Vec<BlameLineInfo>, String>>, String> {
        // Get the repository path from tasks
        let repo_path = {
            let tasks = self.tasks.lock().unwrap();
            let task = tasks.get(target_repo_url).ok_or_else(|| {
                format!(
                    "Repository {} not found in managed repositories",
                    target_repo_url
                )
            })?;

            match &task.status {
                CloneStatus::Completed => {
                    if let Some(path) = &task.temp_dir {
                        Ok(path.clone())
                    } else {
                        Err("Repository path not found".to_string())
                    }
                }
                _ => Err(format!(
                    "Repository is not in a completed state: {:?}",
                    task.status
                )),
            }
        }?; // Release MutexGuard here

        // Now proceed with the blocking operation without holding the lock
        let path_buf = repo_path;
        let results = task::spawn_blocking(move || {
            let mut blame_results = HashMap::new();

            for file_path in file_paths {
                let absolute_file_path = path_buf.join(&file_path);
                blame_results.insert(file_path.clone(), blame_file(&absolute_file_path));
            }

            blame_results
        })
        .await
        .map_err(|e| format!("Task execution failed: {}", e))?;

        Ok(results)
    }
}

// Helper function to perform git blame on a single file
fn blame_file(file_path: &Path) -> Result<Vec<BlameLineInfo>, String> {
    // Check if file exists
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.display()));
    }

    // Get the repository root directory
    let repo_dir = find_git_repo(file_path)
        .ok_or_else(|| format!("Could not find git repository for {}", file_path.display()))?;

    // Make the path relative to the repository root
    let relative_path = file_path
        .strip_prefix(&repo_dir)
        .map_err(|e| format!("Failed to make path relative: {}", e))?;

    // Run git blame command
    let output = Command::new("git")
        .current_dir(&repo_dir)
        .arg("blame")
        .arg("--line-porcelain") // Detailed output format
        .arg(relative_path)
        .output()
        .map_err(|e| format!("Failed to execute git blame: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Git blame failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Parse the output
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_blame_output(&output_str)
}

// Helper function to parse git blame output
fn parse_blame_output(output: &str) -> Result<Vec<BlameLineInfo>, String> {
    let mut blame_lines = Vec::new();
    let mut current_commit_id = String::new();
    let mut current_author_name = String::new();
    let mut current_author_email = String::new();
    let mut current_orig_line = 0;
    let mut current_final_line = 0;
    let mut current_line_content = String::new();

    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("Commit ") {
            current_commit_id = line[7..].trim().to_string();
        } else if line.starts_with("author ") {
            current_author_name = line[7..].trim().to_string();
        } else if line.starts_with("author-mail ") {
            // Remove the angle brackets
            let email = line[12..].trim();
            current_author_email = email
                .strip_prefix('<')
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or(email)
                .to_string();
        } else if line.starts_with("original-line ") {
            current_orig_line = line[14..].trim().parse().unwrap_or(0);
        } else if line.starts_with("final-line ") {
            current_final_line = line[11..].trim().parse().unwrap_or(0);
        } else if line.starts_with('\t') {
            // Tab character indicates the content line
            current_line_content = line[1..].to_string();

            // Add the complete blame info to our collection
            blame_lines.push(BlameLineInfo {
                commit_id: current_commit_id.clone(),
                author_name: current_author_name.clone(),
                author_email: current_author_email.clone(),
                orig_line_no: current_orig_line,
                final_line_no: current_final_line,
                line_content: current_line_content.clone(),
            });
        }
        i += 1;
    }

    Ok(blame_lines)
}

// Helper function to find the git repository root
fn find_git_repo(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();

    // If the path is a file, start with its parent directory
    if current.is_file() {
        current = current.parent()?.to_path_buf();
    }

    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Some(current);
        }

        // Go up one directory
        if !current.pop() {
            break;
        }
    }

    None
}
