use git2::{Branch, BranchType, Repository};
use std::path::Path;
use std::collections::HashMap;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub remote_name: Option<String>,
    pub is_remote: bool,
    pub commit_id: String,
    pub commit_message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
    pub is_head: bool,
}

/// Extracts branch information from a cloned repository.
pub fn extract_branches(repo_path: &Path) -> Result<Vec<BranchInfo>, String> {
    // Open the repository
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository at {:?}: {}", repo_path, e))?;

    // Fetch all remote branches to ensure we have the latest information
    // This is equivalent to 'git fetch --all'
    let mut remote_names = Vec::new();
    
    // Get remote names - StringArray isn't directly iterable, need to use indices
    let remotes = repo.remotes().map_err(|e| format!("Failed to get remotes: {}", e))?;
    for i in 0..remotes.len() {
        if let Some(remote_name) = remotes.get(i) {
            remote_names.push(remote_name.to_string());
        }
    }

    // Fetch all remotes
    for remote_name in &remote_names {
        match repo.find_remote(remote_name) {
            Ok(mut remote) => {
                let fetch_result = remote.fetch(&[] as &[&str], None, None);
                if let Err(e) = fetch_result {
                    eprintln!("Warning: Failed to fetch from remote '{}': {}", remote_name, e);
                    // Continue with other remotes even if one fails
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to find remote '{}': {}", remote_name, e);
                // Continue with other remotes
            }
        }
    }

    // Get HEAD reference to identify the current branch
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(_) => None, // Repository might be empty or HEAD might be detached
    };

    // Process local branches
    let mut branch_infos = Vec::new();
    if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
        for branch_result in branches {
            if let Ok((branch, _)) = branch_result {
                if let Some(branch_info) = process_branch(&repo, branch, &head, false) {
                    branch_infos.push(branch_info);
                }
            }
        }
    }

    // Process remote branches
    if let Ok(branches) = repo.branches(Some(BranchType::Remote)) {
        for branch_result in branches {
            if let Ok((branch, _)) = branch_result {
                if let Some(branch_info) = process_branch(&repo, branch, &head, true) {
                    branch_infos.push(branch_info);
                }
            }
        }
    }

    Ok(branch_infos)
}

/// Processes a single branch to extract its information.
fn process_branch(
    repo: &Repository,
    branch: Branch,
    head: &Option<git2::Reference>,
    is_remote: bool,
) -> Option<BranchInfo> {
    // Get branch name
    let branch_name = match branch.name() {
        Ok(Some(name)) => name.to_string(),
        _ => return None, // Skip branches with invalid names
    };

    // For remote branches, extract the remote name
    let remote_name = if is_remote {
        branch_name
            .split('/')
            .next()
            .map(|remote| remote.to_string())
    } else {
        None
    };

    // Get the target commit
    let oid = match branch.get().target() {
        Some(oid) => oid,
        None => return None, // Skip if no target
    };

    // Find the commit
    let commit = match repo.find_commit(oid) {
        Ok(commit) => commit,
        Err(_) => return None, // Skip if commit not found
    };

    // Check if this branch is the current HEAD
    let is_head = if let Some(ref head_ref) = head {
        head_ref.target() == Some(oid)
    } else {
        false
    };

    // Extract commit details
    let message = commit.message().unwrap_or("").to_string();
    let author = commit.author();
    let author_name = author.name().unwrap_or("").to_string();
    let author_email = author.email().unwrap_or("").to_string();
    let author_time = author.when().seconds();

    Some(BranchInfo {
        name: branch_name,
        remote_name,
        is_remote,
        commit_id: oid.to_string(),
        commit_message: message,
        author_name,
        author_email,
        author_time,
        is_head,
    })
}

/// Extracts branch information from multiple repositories in parallel.
pub fn extract_branches_parallel(
    repo_paths: Vec<(String, std::path::PathBuf)>,
) -> HashMap<String, Result<Vec<BranchInfo>, String>> {
    repo_paths
        .par_iter() // Process repositories in parallel using Rayon
        .map(|(repo_url, path)| {
            let result = extract_branches(path);
            (repo_url.clone(), result)
        })
        .collect()
}