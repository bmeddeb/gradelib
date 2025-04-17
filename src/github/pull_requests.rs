use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::traits::PullRequestOperations;
use crate::common::types::PullRequestInfo;
use crate::github::provider::GitHubProvider;

#[derive(Debug, Deserialize, Serialize)]
struct GitHubUser {
    login: String,
    id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubLabel {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubPullRequest {
    id: i64,
    number: i32,
    title: String,
    state: String,
    created_at: String,
    updated_at: String,
    closed_at: Option<String>,
    merged_at: Option<String>,
    user: GitHubUser,
    body: Option<String>,
    comments: i32,
    commits: i32,
    additions: i32,
    deletions: i32,
    changed_files: i32,
    mergeable: Option<bool>,
    labels: Vec<GitHubLabel>,
    draft: bool,
    merged: bool,
    merged_by: Option<GitHubUser>,
}

#[async_trait]
impl PullRequestOperations for GitHubProvider {
    async fn fetch_pull_requests(
        &self,
        repo_urls: Vec<String>,
        state: Option<&str>,
    ) -> Result<HashMap<String, Result<Vec<PullRequestInfo>, String>>, String> {
        let client = reqwest::Client::new();
        let mut results = HashMap::new();

        for url in repo_urls {
            let repo_name = match extract_repo_name(&url) {
                Ok(name) => name,
                Err(e) => {
                    results.insert(url, Err(e));
                    continue;
                }
            };

            match fetch_repo_pull_requests(&client, &repo_name, &self.username, &self.token, state)
                .await
            {
                Ok(pull_requests) => {
                    results.insert(url, Ok(pull_requests));
                }
                Err(e) => {
                    results.insert(url, Err(e));
                }
            }
        }

        Ok(results)
    }
}

async fn fetch_repo_pull_requests(
    client: &Client,
    repo_name: &str,
    username: &str,
    token: &str,
    state: Option<&str>,
) -> Result<Vec<PullRequestInfo>, String> {
    // Build the query URL with optional state parameter
    let mut url = format!("https://api.github.com/repos/{}/pulls", repo_name);
    if let Some(state_val) = state {
        url = format!("{}?state={}", url, state_val);
    } else {
        url = format!("{}?state=all", url);
    }

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("gradelib-agent"),
    );

    let response = client
        .get(&url)
        .basic_auth(username, Some(token))
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API error: {} - {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }

    let gh_prs: Vec<GitHubPullRequest> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Convert to our internal type
    let pull_requests = gh_prs
        .into_iter()
        .map(|pr| PullRequestInfo {
            id: pr.id,
            number: pr.number,
            title: pr.title,
            state: pr.state,
            created_at: pr.created_at,
            updated_at: pr.updated_at,
            closed_at: pr.closed_at,
            merged_at: pr.merged_at,
            user_login: pr.user.login,
            user_id: pr.user.id,
            body: pr.body,
            comments: pr.comments,
            commits: pr.commits,
            additions: pr.additions,
            deletions: pr.deletions,
            changed_files: pr.changed_files,
            mergeable: pr.mergeable,
            labels: pr.labels.into_iter().map(|l| l.name).collect(),
            draft: pr.draft,
            merged: pr.merged,
            merged_by: pr.merged_by.map(|user| user.login),
        })
        .collect();

    Ok(pull_requests)
}

fn extract_repo_name(url: &str) -> Result<String, String> {
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
