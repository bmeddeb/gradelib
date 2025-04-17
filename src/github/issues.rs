use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::traits::IssueOperations;
use crate::common::types::IssueInfo;
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
struct GitHubMilestone {
    title: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubIssue {
    id: i64,
    number: i32,
    title: String,
    state: String,
    created_at: String,
    updated_at: String,
    closed_at: Option<String>,
    user: GitHubUser,
    body: Option<String>,
    comments: i32,
    labels: Vec<GitHubLabel>,
    assignees: Vec<GitHubUser>,
    milestone: Option<GitHubMilestone>,
    locked: bool,
    html_url: String,
    pull_request: Option<serde_json::Value>,
}

#[async_trait]
impl IssueOperations for GitHubProvider {
    async fn fetch_issues(
        &self,
        repo_urls: Vec<String>,
        state: Option<&str>,
    ) -> Result<HashMap<String, Result<Vec<IssueInfo>, String>>, String> {
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

            match fetch_repo_issues(&client, &repo_name, &self.username, &self.token, state).await {
                Ok(issues) => {
                    results.insert(url, Ok(issues));
                }
                Err(e) => {
                    results.insert(url, Err(e));
                }
            }
        }

        Ok(results)
    }
}

async fn fetch_repo_issues(
    client: &Client,
    repo_name: &str,
    username: &str,
    token: &str,
    state: Option<&str>,
) -> Result<Vec<IssueInfo>, String> {
    // Build the query URL with optional state parameter
    let mut url = format!("https://api.github.com/repos/{}/issues", repo_name);
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

    let gh_issues: Vec<GitHubIssue> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Convert to our internal type
    let issues = gh_issues
        .into_iter()
        .map(|issue| IssueInfo {
            id: issue.id,
            number: issue.number,
            title: issue.title,
            state: issue.state,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            closed_at: issue.closed_at,
            user_login: issue.user.login,
            user_id: issue.user.id,
            body: issue.body,
            comments_count: issue.comments,
            is_pull_request: issue.pull_request.is_some(),
            labels: issue.labels.into_iter().map(|l| l.name).collect(),
            assignees: issue.assignees.into_iter().map(|a| a.login).collect(),
            milestone: issue.milestone.map(|m| m.title),
            locked: issue.locked,
            html_url: issue.html_url,
        })
        .collect();

    Ok(issues)
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
