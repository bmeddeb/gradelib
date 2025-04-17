use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tokio::task;

use crate::common::traits::BranchOperations;
use crate::common::types::{BranchInfo, CloneStatus};
use crate::github::provider::GitHubProvider;

#[async_trait]
impl BranchOperations for GitHubProvider {
    async fn analyze_branches(
        &self,
        repo_urls: Vec<String>,
    ) -> HashMap<String, Result<Vec<BranchInfo>, String>> {
        let mut results = HashMap::new();

        // Collect paths for all completed repositories
        let repo_paths = {
            let mut paths = Vec::new();
            let tasks = self.tasks.lock().unwrap();

            for url in &repo_urls {
                if let Some(task) = tasks.get(url) {
                    match &task.status {
                        CloneStatus::Completed => {
                            if let Some(path) = &task.temp_dir {
                                paths.push((url.clone(), path.clone()));
                            }
                        }
                        _ => {
                            // Skip repositories that aren't completed
                            results.insert(
                                url.clone(),
                                Err(format!("Repository {} is not in completed state", url)),
                            );
                        }
                    }
                } else {
                    results.insert(
                        url.clone(),
                        Err(format!("Repository {} is not managed", url)),
                    );
                }
            }

            paths
        }; // MutexGuard is dropped here

        // Process branches in parallel without holding the lock
        let branches_results = task::spawn_blocking(move || {
            let mut results = HashMap::new();
            for (url, path) in repo_paths {
                results.insert(url, extract_branches(&path));
            }
            results
        })
        .await
        .unwrap_or_else(|e| {
            let mut error_map = HashMap::new();
            for url in repo_urls.clone() {
                if !results.contains_key(&url) {
                    error_map.insert(url, Err(format!("Task execution failed: {}", e)));
                }
            }
            error_map
        });

        // Merge the results
        for (url, result) in branches_results {
            results.insert(url, result);
        }

        results
    }
}

fn extract_branches(repo_path: &Path) -> Result<Vec<BranchInfo>, String> {
    // Get all branches with their details
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(&[
            "branch",
            "-a",
            "--format=%(refname)|%(objectname)|%(subject)|%(authorname)|%(authoremail)|%(authordate:unix)|%(HEAD)",
        ])
        .output()
        .map_err(|e| format!("Failed to execute git branch: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Git branch failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Parse the output
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 7 {
            continue;
        }

        let refname = parts[0];
        let commit_id = parts[1].to_string();
        let commit_message = parts[2].to_string();
        let author_name = parts[3].to_string();
        let author_email = parts[4].to_string();
        let author_time = parts[5].parse::<i64>().unwrap_or(0);
        let is_head = parts[6] == "*";

        // Determine if it's a remote branch and get the name
        let (name, is_remote, remote_name) = if refname.starts_with("refs/remotes/") {
            let ref_parts: Vec<&str> = refname
                .strip_prefix("refs/remotes/")
                .unwrap()
                .split('/')
                .collect();
            if ref_parts.len() >= 2 {
                (
                    ref_parts[1..].join("/"),
                    true,
                    Some(ref_parts[0].to_string()),
                )
            } else {
                (refname.to_string(), true, None)
            }
        } else if refname.starts_with("refs/heads/") {
            (
                refname.strip_prefix("refs/heads/").unwrap().to_string(),
                false,
                None,
            )
        } else {
            (refname.to_string(), false, None)
        };

        branches.push(BranchInfo {
            name,
            remote_name,
            is_remote,
            commit_id,
            commit_message,
            author_name,
            author_email,
            author_time,
            is_head,
        });
    }

    Ok(branches)
}
