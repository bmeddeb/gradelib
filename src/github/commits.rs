use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tokio::task;

use crate::common::traits::CommitOperations;
use crate::common::types::{CloneStatus, CommitInfo};
use crate::github::provider::GitHubProvider;
use crate::utils;

#[async_trait]
impl CommitOperations for GitHubProvider {
    async fn analyze_commits(&self, target_repo_url: &str) -> Result<Vec<CommitInfo>, String> {
        // Get the repository path and repo name inside a block to release the MutexGuard
        let (repo_path, repo_name) = {
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
                        // Extract repo name here while we have the lock
                        let repo_name = utils::extract_repo_name(target_repo_url)?;
                        Ok((path.clone(), repo_name))
                    } else {
                        Err("Repository path not found".to_string())
                    }
                }
                _ => Err(format!(
                    "Repository is not in a completed state: {:?}",
                    task.status
                )),
            }
        }?; // MutexGuard is dropped here

        // Process commits in a blocking task without holding the lock
        task::spawn_blocking(move || extract_commits(&repo_path, &repo_name))
            .await
            .map_err(|e| format!("Task execution failed: {}", e))?
    }
}

fn extract_repo_name(url: &str) -> Result<String, String> {
    // Extract repo name from URL (e.g., "owner/repo" from https://github.com/owner/repo)
    if url.starts_with("https://github.com/") {
        let parts: Vec<&str> = url
            .strip_prefix("https://github.com/")
            .unwrap()
            .split('/')
            .collect();
        if parts.len() >= 2 {
            Ok(format!(
                "{}/{}",
                parts[0],
                parts[1].trim_end_matches(".git")
            ))
        } else {
            Err("Invalid GitHub URL format".to_string())
        }
    } else if url.starts_with("git@github.com:") {
        let parts: Vec<&str> = url
            .strip_prefix("git@github.com:")
            .unwrap()
            .split('/')
            .collect();
        if parts.len() >= 2 {
            Ok(format!(
                "{}/{}",
                parts[0],
                parts[1].trim_end_matches(".git")
            ))
        } else {
            Err("Invalid GitHub URL format".to_string())
        }
    } else {
        Err(format!("Unsupported URL format: {}", url))
    }
}

fn extract_commits(repo_path: &Path, repo_name: &str) -> Result<Vec<CommitInfo>, String> {
    // Get all commits with their details
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(&[
            "log",
            "--pretty=format:%H|%an|%ae|%at|%cn|%ce|%ct|%s|%p",
            "--numstat",
        ])
        .output()
        .map_err(|e| format!("Failed to execute git log: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Parse the output
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_git_log_output(&output_str, repo_name)
}

fn parse_git_log_output(output: &str, repo_name: &str) -> Result<Vec<CommitInfo>, String> {
    let mut commits = Vec::new();
    let mut lines = output.lines().peekable();

    while let Some(line) = lines.next() {
        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Parse commit info
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 9 {
            continue; // Not a commit line in expected format
        }

        let sha = parts[0].to_string();
        let author_name = parts[1].to_string();
        let author_email = parts[2].to_string();
        let author_timestamp = parts[3].parse::<i64>().unwrap_or(0);
        let committer_name = parts[4].to_string();
        let committer_email = parts[5].to_string();
        let committer_timestamp = parts[6].parse::<i64>().unwrap_or(0);
        let message = parts[7].to_string();
        let parent_hashes = parts[8].to_string();
        let is_merge = parent_hashes.contains(' '); // Merges have multiple parents

        // Timezone offset - default to 0 since we don't have it in the log format
        let author_offset = 0;
        let committer_offset = 0;

        // Process stats lines
        let mut additions = 0;
        let mut deletions = 0;

        // Keep reading lines until we hit the next commit or end of output
        while let Some(line) = lines.peek() {
            if line.is_empty() || line.contains('|') {
                break;
            }

            // Parse numstat line (additions, deletions, filename)
            let stats_parts: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
            if stats_parts.len() >= 2 {
                if let Ok(add) = stats_parts[0].parse::<usize>() {
                    additions += add;
                }
                if let Ok(del) = stats_parts[1].parse::<usize>() {
                    deletions += del;
                }
            }
        }

        // Create commit info
        commits.push(CommitInfo {
            sha,
            repo_name: repo_name.to_string(),
            message,
            author_name,
            author_email,
            author_timestamp,
            author_offset,
            committer_name,
            committer_email,
            committer_timestamp,
            committer_offset,
            additions,
            deletions,
            is_merge,
        });
    }

    Ok(commits)
}
