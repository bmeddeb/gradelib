use crate::common::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

#[async_trait]
pub trait Provider {
    fn name(&self) -> &str;
    fn get_credentials(&self) -> ProviderCredentials;
}

#[async_trait]
pub trait RepoOperations {
    async fn clone_repo(&self, url: &str) -> Result<(), String>;
    async fn clone_all(&self, urls: &[&str]) -> Result<(), String>;
    async fn get_clone_tasks(&self) -> HashMap<String, RepoCloneTask>;
    async fn bulk_blame(
        &self,
        target_repo_url: &str,
        file_paths: Vec<String>,
    ) -> Result<HashMap<String, Result<Vec<BlameLineInfo>, String>>, String>;
}

#[async_trait]
pub trait CommitOperations {
    async fn analyze_commits(&self, target_repo_url: &str) -> Result<Vec<CommitInfo>, String>;
}

#[async_trait]
pub trait BranchOperations {
    async fn analyze_branches(
        &self,
        repo_urls: Vec<String>,
    ) -> HashMap<String, Result<Vec<BranchInfo>, String>>;
}

#[async_trait]
pub trait CollaboratorOperations {
    async fn fetch_collaborators(
        &self,
        repo_urls: Vec<String>,
    ) -> Result<HashMap<String, Vec<CollaboratorInfo>>, String>;
}

#[async_trait]
pub trait IssueOperations {
    async fn fetch_issues(
        &self,
        repo_urls: Vec<String>,
        state: Option<&str>,
    ) -> Result<HashMap<String, Result<Vec<IssueInfo>, String>>, String>;
}

#[async_trait]
pub trait PullRequestOperations {
    async fn fetch_pull_requests(
        &self,
        repo_urls: Vec<String>,
        state: Option<&str>,
    ) -> Result<HashMap<String, Result<Vec<PullRequestInfo>, String>>, String>;
}
