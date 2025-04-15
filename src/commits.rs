use git2::{Commit, DiffOptions, Oid, Repository, Sort};
use std::path::{Path, PathBuf};
use rayon::prelude::*; // Import Rayon traits

/// Represents information extracted for a single commit.
#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub sha: String,
    pub repo_name: String, // Name/slug of the repository (e.g., "owner/repo")
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_timestamp: i64, // Seconds since epoch
    pub author_offset: i32,    // Timezone offset in minutes
    pub committer_name: String,
    pub committer_email: String,
    pub committer_timestamp: i64,
    pub committer_offset: i32,
    pub additions: usize,
    pub deletions: usize,
    pub is_merge: bool,
    // pub branch: Option<String>, // Omitted for complexity/performance reasons
    // pub url: String, // URL construction moved to process_single_commit
}

/// Calculates additions and deletions for a commit by diffing against its first parent.
/// Handles the initial commit case (no parents).
fn calculate_diff_stats(
    repo: &Repository,
    commit: &Commit,
) -> Result<(usize, usize), git2::Error> {
    let commit_tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        let parent = commit.parent(0)?;
        Some(parent.tree()?)
    } else {
        None // Initial commit
    };

    let mut diff_opts = DiffOptions::new();
    diff_opts.ignore_submodules(true);
    diff_opts.ignore_whitespace(true);

    let diff =
        repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), Some(&mut diff_opts))?;
    let stats = diff.stats()?;
    Ok((stats.insertions(), stats.deletions()))
}

/// Extracts information for a single commit OID.
/// Designed to be called within a Rayon parallel iterator.
/// Opens its own repository handle for thread safety.
fn process_single_commit(
    repo_path: &Path,
    oid: Oid,
    repo_name: &str,
) -> Result<CommitInfo, String> {
    // Open repo handle specific to this thread/task
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repo in thread for {}: {}", oid, e))?;

    let commit = repo
        .find_commit(oid)
        .map_err(|e| format!("Failed to find commit {}: {}", oid, e))?;

    let (additions, deletions) = calculate_diff_stats(&repo, &commit)
        .map_err(|e| format!("Failed to calculate stats for commit {}: {}", oid, e))?;

    let author = commit.author();
    let committer = commit.committer();
    let author_time = author.when();
    let committer_time = committer.when();

    let commit_info = CommitInfo {
        sha: oid.to_string(),
        repo_name: repo_name.to_string(), // Include the repo name
        message: commit.message().unwrap_or("").trim().to_string(),
        author_name: author.name().unwrap_or("").to_string(),
        author_email: author.email().unwrap_or("").to_string(),
        author_timestamp: author_time.seconds(),
        author_offset: author_time.offset_minutes(),
        committer_name: committer.name().unwrap_or("").to_string(),
        committer_email: committer.email().unwrap_or("").to_string(),
        committer_timestamp: committer_time.seconds(),
        committer_offset: committer_time.offset_minutes(),
        additions,
        deletions,
        is_merge: commit.parent_count() > 1,
        // url: format!("https://github.com/{}/commit/{}", repo_name, oid), // Example URL
    };

    Ok(commit_info)
}

/// Extracts commit history information from a cloned repository using parallel processing.
/// This function is synchronous but performs work in parallel using Rayon.
pub fn extract_commits_parallel(
    repo_path: PathBuf,      // Take ownership of path
    repo_name: String, // Take ownership of name
) -> Result<Vec<CommitInfo>, String> {
    // --- Step 1: Get all commit OIDs (Sequential) ---
    let oids = {
        let repo = Repository::open(&repo_path)
            .map_err(|e| format!("Failed to open repository at {:?}: {}", repo_path, e))?;
        let mut revwalk = repo.revwalk().map_err(|e| format!("Failed to create revwalk: {}", e))?;
        revwalk.push_head().map_err(|e| format!("Failed to push HEAD: {}", e))?;
        // Consider adding other refs like all branches if needed: revwalk.push_glob("refs/heads/*")?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME).map_err(|e| format!("Failed to set revwalk sorting: {}", e))?;

        let oids: Result<Vec<Oid>, _> = revwalk.collect();
        oids.map_err(|e| format!("Failed during revwalk iteration: {}", e))?
    };

    // --- Step 2: Process commits in parallel using Rayon ---
    let results: Vec<Result<CommitInfo, String>> = oids
        .into_par_iter()
        .map(|oid| {
            // Clone repo_path and repo_name for the closure
            process_single_commit(&repo_path, oid, &repo_name)
        })
        .collect();

    // --- Step 3: Collect results, handling errors ---
    let mut commit_infos = Vec::with_capacity(results.len());
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(info) => commit_infos.push(info),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        // If any errors occurred, return a combined error message
        // You might want more sophisticated error reporting
        Err(format!("Errors encountered during commit processing: {}", errors.join("; ")))
    } else {
        Ok(commit_infos)
    }
}
