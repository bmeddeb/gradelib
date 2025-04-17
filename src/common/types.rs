use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Provider credential types
#[derive(Clone)]
pub enum ProviderCredentials {
    Basic { username: String, token: String },
    OAuth { token: String },
    None,
}

// Common type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub repo_name: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_timestamp: i64,
    pub author_offset: i32,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_timestamp: i64,
    pub committer_offset: i32,
    pub additions: usize,
    pub deletions: usize,
    pub is_merge: bool,
}

#[derive(Debug, Clone)]
pub struct BlameLineInfo {
    pub commit_id: String,
    pub author_name: String,
    pub author_email: String,
    pub orig_line_no: usize,
    pub final_line_no: usize,
    pub line_content: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    pub login: String,
    pub github_id: i64,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueInfo {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub user_login: String,
    pub user_id: i64,
    pub body: Option<String>,
    pub comments_count: i32,
    pub is_pull_request: bool,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub milestone: Option<String>,
    pub locked: bool,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub merged_at: Option<String>,
    pub user_login: String,
    pub user_id: i64,
    pub body: Option<String>,
    pub comments: i32,
    pub commits: i32,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub mergeable: Option<bool>,
    pub labels: Vec<String>,
    pub draft: bool,
    pub merged: bool,
    pub merged_by: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CloneStatus {
    Queued,
    Cloning(u8), // percent complete
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct RepoCloneTask {
    pub url: String,
    pub status: CloneStatus,
    pub temp_dir: Option<PathBuf>,
}
