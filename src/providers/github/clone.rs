use std::path::PathBuf;

/// Internal representation of the status of a cloning operation.
#[derive(Debug, Clone)]
pub enum InternalCloneStatus {
    Queued,
    Cloning(u8), // percent complete
    Completed,
    Failed(String),
}

/// Internal representation of a repository cloning task.
#[derive(Debug, Clone)]
pub struct InternalRepoCloneTask {
    pub url: String,
    pub status: InternalCloneStatus,
    pub temp_dir: Option<PathBuf>, // Stores the path to the temporary directory if clone is successful
} 