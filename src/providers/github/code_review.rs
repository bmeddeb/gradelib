use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewInfo {
    pub id: i64,
    pub pr_number: i32,
    pub user_login: String,
    pub user_id: i64,
    pub body: Option<String>,
    pub state: String, // "APPROVED", "COMMENTED", "CHANGES_REQUESTED", etc.
    pub submitted_at: String,
    pub commit_id: String,
    pub html_url: String,
}

/// Fetches code review information for multiple repositories concurrently
///
/// For each input repo URL, returns either a map of PR numbers to reviews or an error string.
/// If the GitHub client cannot be created, all URLs are mapped to the error string.
pub async fn fetch_code_reviews(
    repo_urls: Vec<String>,
    _github_username: &str, // Prefix with underscore to indicate intentional non-use
    github_token: &str,
) -> Result<HashMap<String, Result<HashMap<i32, Vec<ReviewInfo>>, String>>, String> {
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

    // Fetch code reviews for all repositories concurrently
    let mut tasks = Vec::new();

    for repo_url in repo_urls {
        let client = client.clone();
        let token = github_token.to_string();
        let url = repo_url.clone();

        let task = task::spawn(async move {
            let result = fetch_repo_code_reviews(&client, &url, &token).await;
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

/// Fetches code reviews for a single repository
async fn fetch_repo_code_reviews(
    client: &reqwest::Client,
    repo_url: &str,
    _token: &str, // Prefixed with underscore to indicate intentional non-use
) -> Result<HashMap<i32, Vec<ReviewInfo>>, String> {
    // Parse owner/repo from URL
    let (owner, repo) = parse_repo_parts(repo_url)?;

    // Fetch open and recently closed pull requests first
    let pr_url = format!(
        "https://api.github.com/repos/{}/{}/pulls?state=all&sort=updated&direction=desc&per_page=100",
        owner, repo
    );

    #[derive(Deserialize)]
    struct PullRequestBasic {
        number: i32,
        html_url: String,
    }

    let prs_response = client
        .get(&pr_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch pull requests: {}", e))?;

    if !prs_response.status().is_success() {
        return Err(format!("GitHub API error: {}", prs_response.status()));
    }

    let pull_requests: Vec<PullRequestBasic> = prs_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse pull requests response: {}", e))?;

    // Fetch reviews for each PR
    let mut result_map = HashMap::new();

    for pr in pull_requests {
        match fetch_pr_reviews(client, &owner, &repo, pr.number, &pr.html_url).await {
            Ok(reviews) => {
                if !reviews.is_empty() {
                    result_map.insert(pr.number, reviews);
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to fetch reviews for PR #{}: {}",
                    pr.number, e
                );
            }
        }
    }

    Ok(result_map)
}

/// Fetches review information for a single pull request
async fn fetch_pr_reviews(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    pr_number: i32,
    _pr_html_url: &str,
) -> Result<Vec<ReviewInfo>, String> {
    let reviews_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
        owner, repo, pr_number
    );

    #[derive(Deserialize)]
    struct ReviewResponse {
        id: i64,
        user: User,
        body: Option<String>,
        state: String,
        submitted_at: String,
        commit_id: String,
        html_url: String,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
        id: i64,
    }

    let reviews_response = client
        .get(&reviews_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch reviews: {}", e))?;

    if !reviews_response.status().is_success() {
        return Err(format!("GitHub API error: {}", reviews_response.status()));
    }

    let reviews: Vec<ReviewResponse> = reviews_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse reviews response: {}", e))?;

    let mut result = Vec::new();
    for review in reviews {
        result.push(ReviewInfo {
            id: review.id,
            pr_number,
            user_login: review.user.login,
            user_id: review.user.id,
            body: review.body,
            state: review.state,
            submitted_at: review.submitted_at,
            commit_id: review.commit_id,
            html_url: review.html_url,
        });
    }

    Ok(result)
}
