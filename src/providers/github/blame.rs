use git2::{BlameOptions, Repository};
use std::path::Path;
use std::{
    fs,                  // For reading file content
    io::{self, BufRead}, // For reading file content efficiently
};

/// Represents information about a single line from a git blame operation.
#[derive(Clone, Debug)]
pub struct BlameLineInfo {
    pub commit_id: String,    // Full commit hash
    pub author_name: String,
    pub author_email: String,
    pub orig_line_no: usize,  // 1-based original line number in the commit
    pub final_line_no: usize, // 1-based final line number in the file
    pub line_content: String,
}

/// Performs git blame on a single file within a repository.
/// Designed to be run synchronously, intended for use with `tokio::task::spawn_blocking`.
pub fn get_blame_for_file(
    repo_path: &Path,
    file_path_relative: &str,
) -> Result<Vec<BlameLineInfo>, String> {
    // 1. Open the repository
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository at {:?}: {}", repo_path, e))?;

    let file_path_repo = Path::new(file_path_relative);

    // 2. Read the actual file content for context
    let full_file_path = repo_path.join(file_path_repo);
    let file_lines = match fs::File::open(&full_file_path) {
        Ok(file) => io::BufReader::new(file)
            .lines()
            .collect::<Result<Vec<String>, _>>(),
        // Handle file not found specifically
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(format!("File not found at path: {:?}", full_file_path));
        }
        Err(e) => {
            return Err(format!(
                "Failed to open/read file {:?}: {}",
                full_file_path, e
            ))
        }
    }
    .map_err(|e| format!("Failed to read lines from file {:?}: {}", full_file_path, e))?;

    // 3. Perform git blame using git2-rs
    let mut blame_opts = BlameOptions::new();
    // Configure options if needed (e.g., track copies, specific commit)

    let blame = match repo.blame_file(file_path_repo, Some(&mut blame_opts)) {
        Ok(b) => b,
        // Handle case where file isn't in the repository index / history
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            return Err(format!(
                "File {:?} not found in repository history.",
                file_path_relative
            ));
        }
        Err(e) => {
            return Err(format!(
                "Failed to blame file {:?}: {}",
                file_path_relative, e
            ))
        }
    };

    // 4. Process hunks and lines into BlameLineInfo structs
    let mut blame_results: Vec<BlameLineInfo> = Vec::with_capacity(file_lines.len());
    for hunk in blame.iter() {
        let commit_id = hunk.final_commit_id().to_string(); // Full commit hash
        let signature = hunk.orig_signature();
        // Use empty strings as fallback for potentially missing signature info
        let author_name = signature.name().unwrap_or("").to_string();
        let author_email = signature.email().unwrap_or("").to_string();
        let start_line_no = hunk.final_start_line(); // 1-based line number in final file
        let orig_start_line_no = hunk.orig_start_line(); // 1-based line number in original commit

        // Iterate through each line within the current blame hunk
        for i in 0..hunk.lines_in_hunk() {
            let final_line_no = start_line_no + i; // Calculate 1-based final line number
            let orig_line_no = orig_start_line_no + i; // Calculate 1-based original line number

            // Get corresponding content from the pre-read file lines (using 0-based index)
            let line_content = file_lines
                .get(final_line_no - 1) // Use 0-based index
                .cloned()
                .unwrap_or_else(String::new); // Use empty string if line index is out of bounds

            blame_results.push(BlameLineInfo {
                commit_id: commit_id.clone(), // Clone commit_id for each line
                author_name: author_name.clone(),
                author_email: author_email.clone(),
                orig_line_no,
                final_line_no,
                line_content,
            });
        }
    }

    Ok(blame_results)
} 