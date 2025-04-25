use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

use crate::repo::parse_slug_from_url;

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

#[derive(Deserialize)]
struct Label {
    name: String,
}

#[derive(Deserialize)]
struct User {
    login: String,
    id: i64,
}

#[derive(Deserialize)]
struct Milestone {
    title: String,
}

/// Fetches issue information for multiple repositories concurrently
///
/// For each input repo URL, returns either a list of issues or an error string.
/// If the GitHub client cannot be created, all URLs are mapped to the error string.
pub async fn fetch_issues(
    repo_urls: Vec<String>,
    github_username: &str,
    github_token: &str,
    state: Option<&str>, // "open", "closed", "all"
) -> Result<HashMap<String, Result<Vec<IssueInfo>, String>>, String> {
    // Create a GitHub client
    let client = match create_github_client(github_token) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to create GitHub client: {}", e);
            let mut results = HashMap::new();
            for url in repo_urls {
                results.insert(url, Err(err_msg.clone()));
            }
            return Ok(results);
        }
    };

    // Fetch issues for all repositories concurrently
    let mut tasks = Vec::new();

    for repo_url in repo_urls {
        let client = client.clone();
        let token = github_token.to_string();
        let username = github_username.to_string();
        let url = repo_url.clone();
        let state_param = state.map(|s| s.to_string());

        let task = task::spawn(async move {
            let result =
                fetch_repo_issues(&client, &url, &username, &token, state_param.as_deref()).await;
            (url, result)
        });

        tasks.push(task);
    }

    // Collect results
    let mut results = HashMap::new();
    for task in tasks {
        match task.await {
            Ok((repo_url, result)) => {
                results.insert(repo_url, result);
            }
            Err(e) => {
                eprintln!("Task failed: {}", e);
                // Could insert an error result here if needed
            }
        }
    }

    Ok(results)
}

/// Creates a GitHub API client with proper authentication
fn create_github_client(token: &str) -> Result<reqwest::Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    // Standard GitHub API headers
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token)).unwrap(),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("gradelib-github-client/0.1.0"),
    );

    reqwest::Client::builder().default_headers(headers).build()
}

/// Parses owner and repo name from GitHub URL
fn parse_repo_parts(repo_url: &str) -> Result<(String, String), String> {
    let slug = parse_slug_from_url(repo_url)
        .ok_or_else(|| format!("Invalid repository URL format: {}", repo_url))?;

    let parts: Vec<&str> = slug.split('/').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid repository slug format: {}", slug));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Fetches issues for a single repository
async fn fetch_repo_issues(
    client: &reqwest::Client,
    repo_url: &str,
    _github_username: &str, // Prefixed with underscore to indicate intentional non-use
    _github_token: &str,    // Prefixed with underscore to indicate intentional non-use
    state: Option<&str>,    // "open", "closed", "all"
) -> Result<Vec<IssueInfo>, String> {
    // Parse owner/repo from URL
    let (owner, repo) = parse_repo_parts(repo_url)?;

    // Build the URL with optional state parameter
    let mut issues_url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);

    // Build query parameters
    let mut query_params = Vec::new();

    // Add state parameter if provided, otherwise default to "all"
    if let Some(state_val) = state {
        query_params.push(format!("state={}", state_val));
    } else {
        query_params.push("state=all".to_string());
    }

    // Always include closed issues
    query_params.push("direction=desc".to_string());

    // Sort by updated to get most recent issues first
    query_params.push("sort=updated".to_string());

    // Get as many issues as possible per page
    query_params.push("per_page=100".to_string());

    // Append query parameters to URL
    if !query_params.is_empty() {
        issues_url = format!("{}?{}", issues_url, query_params.join("&"));
    }

    // GitHub API returns both issues and pull requests from the issues endpoint
    // So we need to filter out pull requests if we only want issues
    #[derive(Deserialize)]
    struct IssueResponse {
        id: i64,
        number: i32,
        title: String,
        state: String,
        created_at: String,
        updated_at: String,
        closed_at: Option<String>,
        user: User,
        body: Option<String>,
        comments: i32,
        pull_request: Option<PullRequest>,
        labels: Vec<Label>,
        assignees: Vec<User>,
        milestone: Option<Milestone>,
        locked: bool,
        html_url: String,
    }

    #[derive(Deserialize)]
    struct PullRequest {
        // The presence of this field indicates this issue is actually a pull request
        // We don't need any fields, just checking if it exists
        #[allow(dead_code)]
        url: String,
    }

    // Fetch issues
    let issues_response = client
        .get(&issues_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch issues: {}", e))?;

    if !issues_response.status().is_success() {
        return Err(format!("GitHub API error: {}", issues_response.status()));
    }

    let issue_responses: Vec<IssueResponse> = issues_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse issues response: {}", e))?;

    // Process the issues
    let mut issues = Vec::new();
    for issue in issue_responses {
        // Convert labels to list of strings
        let label_names = issue.labels.iter().map(|l| l.name.clone()).collect();

        // Convert assignees to list of logins
        let assignee_logins = issue.assignees.iter().map(|a| a.login.clone()).collect();

        // Get milestone title if present
        let milestone_title = issue.milestone.map(|m| m.title);

        // Create the issue info
        let issue_info = IssueInfo {
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
            labels: label_names,
            assignees: assignee_logins,
            milestone: milestone_title,
            locked: issue.locked,
            html_url: issue.html_url,
        };

        issues.push(issue_info);
    }

    Ok(issues)
}
