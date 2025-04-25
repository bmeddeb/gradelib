use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

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

/// Fetches pull request information for multiple repositories concurrently
///
/// For each input repo URL, returns either a list of pull requests or an error string.
/// If the GitHub client cannot be created, all URLs are mapped to the error string.
pub async fn fetch_pull_requests(
    repo_urls: Vec<String>,
    _github_username: &str, // Prefix with underscore to indicate intentional non-use
    github_token: &str,
    state: Option<&str>, // "open", "closed", "all"
) -> Result<HashMap<String, Result<Vec<PullRequestInfo>, String>>, String> {
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

    // Fetch pull requests for all repositories concurrently
    let mut tasks = Vec::new();

    for repo_url in repo_urls {
        let client = client.clone();
        let token = github_token.to_string();
        let url = repo_url.clone();
        let state_param = state.map(|s| s.to_string());

        let task = task::spawn(async move {
            let result =
                fetch_repo_pull_requests(&client, &url, &token, state_param.as_deref()).await;
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
                // We could insert an error result here if needed
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
    use crate::repo::parse_slug_from_url;

    let slug = parse_slug_from_url(repo_url)
        .ok_or_else(|| format!("Invalid repository URL format: {}", repo_url))?;

    let parts: Vec<&str> = slug.split('/').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid repository slug format: {}", slug));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Fetches pull requests for a single repository
async fn fetch_repo_pull_requests(
    client: &reqwest::Client,
    repo_url: &str,
    _token: &str,        // Prefixed with underscore to indicate intentional non-use
    state: Option<&str>, // "open", "closed", "all"
) -> Result<Vec<PullRequestInfo>, String> {
    // Parse owner/repo from URL
    let (owner, repo) = parse_repo_parts(repo_url)?;

    // Build the URL with optional state parameter
    let mut pr_url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);

    if let Some(state_val) = state {
        pr_url = format!("{}?state={}", pr_url, state_val);
    } else {
        // Default to all pull requests
        pr_url = format!("{}?state=all", pr_url);
    }

    // Basic pull request information from /pulls endpoint
    #[derive(Deserialize)]
    struct PullRequestBasic {
        id: i64,
        number: i32,
        title: String,
        state: String,
        created_at: String,
        updated_at: String,
        closed_at: Option<String>,
        merged_at: Option<String>,
        user: User,
        body: Option<String>,
        draft: bool,
        labels: Vec<Label>,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
        id: i64,
    }

    #[derive(Deserialize)]
    struct Label {
        name: String,
    }

    // Fetch the list of pull requests
    let prs_response = client
        .get(&pr_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch pull requests: {}", e))?;

    if !prs_response.status().is_success() {
        return Err(format!("GitHub API error: {}", prs_response.status()));
    }

    let basic_prs: Vec<PullRequestBasic> = prs_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse pull requests response: {}", e))?;

    // Fetch additional details for each PR
    let mut detailed_prs = Vec::new();
    for basic_pr in basic_prs {
        // Convert labels to Vec<String> for the function call
        let label_names: Vec<String> = basic_pr.labels.iter().map(|l| l.name.clone()).collect();

        match fetch_pr_details(
            client,
            &owner,
            &repo,
            basic_pr.number,
            basic_pr.id,
            &basic_pr.title,
            &basic_pr.state,
            &basic_pr.created_at,
            &basic_pr.updated_at,
            &basic_pr.closed_at,
            &basic_pr.merged_at,
            &basic_pr.user.login,
            basic_pr.user.id,
            &basic_pr.body,
            basic_pr.draft,
            &label_names, // Pass Vec<String> instead of Vec<Label>
        )
        .await
        {
            Ok(pr_info) => detailed_prs.push(pr_info),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to fetch details for PR #{}: {}",
                    basic_pr.number, e
                );
                // Add basic info with defaults for missing fields
                let labels = basic_pr.labels.iter().map(|l| l.name.clone()).collect();
                let is_merged = basic_pr.merged_at.is_some();

                detailed_prs.push(PullRequestInfo {
                    id: basic_pr.id,
                    number: basic_pr.number,
                    title: basic_pr.title,
                    state: basic_pr.state,
                    created_at: basic_pr.created_at,
                    updated_at: basic_pr.updated_at,
                    closed_at: basic_pr.closed_at,
                    merged_at: basic_pr.merged_at,
                    user_login: basic_pr.user.login,
                    user_id: basic_pr.user.id,
                    body: basic_pr.body,
                    comments: 0,
                    commits: 0,
                    additions: 0,
                    deletions: 0,
                    changed_files: 0,
                    mergeable: None,
                    labels,
                    draft: basic_pr.draft,
                    merged: is_merged,
                    merged_by: None,
                });
            }
        }
    }

    Ok(detailed_prs)
}

/// Fetches detailed information for a single pull request
async fn fetch_pr_details(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    pr_number: i32,
    pr_id: i64,
    title: &str,
    state: &str,
    created_at: &str,
    updated_at: &str,
    closed_at: &Option<String>,
    merged_at: &Option<String>,
    user_login: &str,
    user_id: i64,
    body: &Option<String>,
    draft: bool,
    labels: &Vec<String>, // Update parameter type to Vec<String>
) -> Result<PullRequestInfo, String> {
    // API URL for detailed PR information
    let pr_detail_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );

    #[derive(Deserialize)]
    struct PullRequestDetail {
        mergeable: Option<bool>,
        merged: bool,
        merged_by: Option<User>,
        comments: i32,
        commits: i32,
        additions: i32,
        deletions: i32,
        changed_files: i32,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
    }

    // Fetch PR details
    let pr_response = client
        .get(&pr_detail_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch PR details: {}", e))?;

    if !pr_response.status().is_success() {
        return Err(format!("GitHub API error: {}", pr_response.status()));
    }

    let pr_detail: PullRequestDetail = pr_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse PR detail response: {}", e))?;

    // No need to convert labels since they're already strings
    let label_names = labels.clone();

    // Build the final PR info object
    Ok(PullRequestInfo {
        id: pr_id,
        number: pr_number,
        title: title.to_string(),
        state: state.to_string(),
        created_at: created_at.to_string(),
        updated_at: updated_at.to_string(),
        closed_at: closed_at.clone(),
        merged_at: merged_at.clone(),
        user_login: user_login.to_string(),
        user_id,
        body: body.clone(),
        comments: pr_detail.comments,
        commits: pr_detail.commits,
        additions: pr_detail.additions,
        deletions: pr_detail.deletions,
        changed_files: pr_detail.changed_files,
        mergeable: pr_detail.mergeable,
        labels: label_names,
        draft,
        merged: pr_detail.merged,
        merged_by: pr_detail.merged_by.map(|user| user.login),
    })
}
