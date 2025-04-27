use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

/// Enum to represent different types of GitHub comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentType {
    Issue,         // General comments on issues
    Commit,        // Comments on specific commits
    PullRequest,   // General comments on pull requests
    ReviewComment, // Line-specific comments on pull requests
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentInfo {
    pub id: i64,
    pub comment_type: CommentType,
    pub user_login: String,
    pub user_id: i64,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,

    // For issue comments
    pub issue_number: Option<i32>,

    // For PR review comments
    pub pull_request_number: Option<i32>,
    pub commit_id: Option<String>,
    pub path: Option<String>,
    pub position: Option<i32>,
    pub line: Option<i32>,

    // For commit comments
    pub commit_sha: Option<String>,
}

/// Fetches comments of various types for multiple repositories concurrently
///
/// For each input repo URL, returns either a list of comments or an error string.
/// If the GitHub client cannot be created, all URLs are mapped to the error string.
pub async fn fetch_comments(
    repo_urls: Vec<String>,
    _github_username: &str, // Prefix with underscore to indicate intentional non-use
    github_token: &str,
    comment_types: Option<Vec<CommentType>>, // Optional filter for comment types
    max_pages: Option<usize>,
) -> Result<HashMap<String, Result<Vec<CommentInfo>, String>>, String> {
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

    // Fetch comments for all repositories concurrently
    let mut tasks = Vec::new();

    for repo_url in repo_urls {
        let client = client.clone();
        let token = github_token.to_string();
        let url = repo_url.clone();
        let types = comment_types.clone();
        let max_pages = max_pages.clone();
        let task = task::spawn(async move {
            let result = fetch_repo_comments(&client, &url, &token, types, max_pages).await;
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

/// Fetches comments for a single repository
async fn fetch_repo_comments(
    client: &reqwest::Client,
    repo_url: &str,
    _token: &str, // Prefixed with underscore to indicate intentional non-use
    comment_types: Option<Vec<CommentType>>,
    max_pages: Option<usize>,
) -> Result<Vec<CommentInfo>, String> {
    // Parse owner/repo from URL
    let (owner, repo) = parse_repo_parts(repo_url)?;

    // Determine which types of comments to fetch
    let types = comment_types.unwrap_or_else(|| {
        vec![
            CommentType::Issue,
            CommentType::Commit,
            CommentType::PullRequest,
            CommentType::ReviewComment,
        ]
    });

    // Use multiple concurrent tasks to fetch different comment types
    let mut tasks = Vec::new();

    if types.iter().any(|t| matches!(t, CommentType::Issue)) {
        let owner_clone = owner.clone();
        let repo_clone = repo.clone();
        let client_clone = client.clone();
        let max_pages = max_pages.clone();
        tasks.push(task::spawn(async move {
            fetch_issue_comments(&client_clone, &owner_clone, &repo_clone, max_pages).await
        }));
    }

    if types.iter().any(|t| matches!(t, CommentType::Commit)) {
        let owner_clone = owner.clone();
        let repo_clone = repo.clone();
        let client_clone = client.clone();
        let max_pages = max_pages.clone();
        tasks.push(task::spawn(async move {
            fetch_commit_comments(&client_clone, &owner_clone, &repo_clone, max_pages).await
        }));
    }

    if types.iter().any(|t| matches!(t, CommentType::PullRequest)) {
        let owner_clone = owner.clone();
        let repo_clone = repo.clone();
        let client_clone = client.clone();
        let max_pages = max_pages.clone();
        tasks.push(task::spawn(async move {
            fetch_pr_comments(&client_clone, &owner_clone, &repo_clone, max_pages).await
        }));
    }

    if types
        .iter()
        .any(|t| matches!(t, CommentType::ReviewComment))
    {
        let owner_clone = owner.clone();
        let repo_clone = repo.clone();
        let client_clone = client.clone();
        let max_pages = max_pages.clone();
        tasks.push(task::spawn(async move {
            fetch_review_comments(&client_clone, &owner_clone, &repo_clone, max_pages).await
        }));
    }

    // Collect results from all tasks
    let mut combined_comments = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => match result {
                Ok(comments) => combined_comments.extend(comments),
                Err(e) => eprintln!("Warning: Failed to fetch some comments: {}", e),
            },
            Err(e) => eprintln!("Task execution failed: {}", e),
        }
    }

    Ok(combined_comments)
}

/// Fetches issue comments for a repository
async fn fetch_issue_comments(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    max_pages: Option<usize>,
) -> Result<Vec<CommentInfo>, String> {
    #[derive(Deserialize)]
    struct IssueBasic {
        number: i32,
    }
    let mut all_comments = Vec::new();
    let mut page = 1;
    loop {
        let issues_url = format!(
            "https://api.github.com/repos/{}/{}/issues?state=all&per_page=100&page={}",
            owner, repo, page
        );
        let issues_response = client
            .get(&issues_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch issues: {}", e))?;
        if !issues_response.status().is_success() {
            return Err(format!(
                "GitHub API error for issues: {}",
                issues_response.status()
            ));
        }
        let issues: Vec<IssueBasic> = issues_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse issues response: {}", e))?;
        if issues.is_empty() {
            break;
        }
        for issue in issues {
            let comments_url = format!(
                "https://api.github.com/repos/{}/{}/issues/{}/comments",
                owner, repo, issue.number
            );
            match fetch_issue_comments_for_number(client, &comments_url, issue.number, max_pages)
                .await
            {
                Ok(comments) => all_comments.extend(comments),
                Err(e) => eprintln!(
                    "Warning: Failed to fetch comments for issue #{}: {}",
                    issue.number, e
                ),
            }
        }
        page += 1;
        if let Some(max) = max_pages {
            if page > max {
                break;
            }
        }
    }
    Ok(all_comments)
}

/// Fetches comments for a specific issue number
async fn fetch_issue_comments_for_number(
    client: &reqwest::Client,
    url: &str,
    issue_number: i32,
    max_pages: Option<usize>,
) -> Result<Vec<CommentInfo>, String> {
    #[derive(Deserialize)]
    struct IssueComment {
        id: i64,
        user: User,
        body: String,
        created_at: String,
        updated_at: String,
        html_url: String,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
        id: i64,
    }

    let mut all_comments = Vec::new();
    let mut page = 1;
    loop {
        let paged_url = format!("{}?per_page=100&page={}", url, page);
        let response = client
            .get(&paged_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch issue comments: {}", e))?;
        if !response.status().is_success() {
            return Err(format!(
                "GitHub API error for issue comments: {}",
                response.status()
            ));
        }
        let comments: Vec<IssueComment> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse issue comments response: {}", e))?;
        if comments.is_empty() {
            break;
        }
        for comment in comments {
            all_comments.push(CommentInfo {
                id: comment.id,
                comment_type: CommentType::Issue,
                user_login: comment.user.login,
                user_id: comment.user.id,
                body: comment.body,
                created_at: comment.created_at,
                updated_at: comment.updated_at,
                html_url: comment.html_url,
                issue_number: Some(issue_number),
                pull_request_number: None,
                commit_id: None,
                path: None,
                position: None,
                line: None,
                commit_sha: None,
            });
        }
        page += 1;
        if let Some(max) = max_pages {
            if page > max {
                break;
            }
        }
    }
    Ok(all_comments)
}

/// Fetches pull request general comments for a repository
async fn fetch_pr_comments(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    max_pages: Option<usize>,
) -> Result<Vec<CommentInfo>, String> {
    #[derive(Deserialize)]
    struct PullRequestBasic {
        number: i32,
    }
    let mut all_comments = Vec::new();
    let mut page = 1;
    loop {
        let prs_url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state=all&per_page=100&page={}",
            owner, repo, page
        );
        let prs_response = client
            .get(&prs_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch pull requests: {}", e))?;
        if !prs_response.status().is_success() {
            return Err(format!(
                "GitHub API error for PRs: {}",
                prs_response.status()
            ));
        }
        let pull_requests: Vec<PullRequestBasic> = prs_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse PRs response: {}", e))?;
        if pull_requests.is_empty() {
            break;
        }
        for pr in pull_requests {
            let comments_url = format!(
                "https://api.github.com/repos/{}/{}/issues/{}/comments",
                owner, repo, pr.number
            );
            match fetch_pr_comments_for_number(client, &comments_url, pr.number, max_pages).await {
                Ok(comments) => all_comments.extend(comments),
                Err(e) => eprintln!(
                    "Warning: Failed to fetch comments for PR #{}: {}",
                    pr.number, e
                ),
            }
        }
        page += 1;
        if let Some(max) = max_pages {
            if page > max {
                break;
            }
        }
    }
    Ok(all_comments)
}

/// Fetches general comments for a specific pull request number
async fn fetch_pr_comments_for_number(
    client: &reqwest::Client,
    url: &str,
    pr_number: i32,
    max_pages: Option<usize>,
) -> Result<Vec<CommentInfo>, String> {
    #[derive(Deserialize)]
    struct PullRequestComment {
        id: i64,
        user: User,
        body: String,
        created_at: String,
        updated_at: String,
        html_url: String,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
        id: i64,
    }

    let mut all_comments = Vec::new();
    let mut page = 1;
    loop {
        let paged_url = format!("{}?per_page=100&page={}", url, page);
        let response = client
            .get(&paged_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch PR comments: {}", e))?;
        if !response.status().is_success() {
            return Err(format!(
                "GitHub API error for PR comments: {}",
                response.status()
            ));
        }
        let comments: Vec<PullRequestComment> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse PR comments response: {}", e))?;
        if comments.is_empty() {
            break;
        }
        for comment in comments {
            all_comments.push(CommentInfo {
                id: comment.id,
                comment_type: CommentType::PullRequest,
                user_login: comment.user.login,
                user_id: comment.user.id,
                body: comment.body,
                created_at: comment.created_at,
                updated_at: comment.updated_at,
                html_url: comment.html_url,
                issue_number: None,
                pull_request_number: Some(pr_number),
                commit_id: None,
                path: None,
                position: None,
                line: None,
                commit_sha: None,
            });
        }
        page += 1;
        if let Some(max) = max_pages {
            if page > max {
                break;
            }
        }
    }
    Ok(all_comments)
}

/// Fetches pull request review comments (inline code comments) for a repository
async fn fetch_review_comments(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    max_pages: Option<usize>,
) -> Result<Vec<CommentInfo>, String> {
    #[derive(Deserialize)]
    struct ReviewComment {
        id: i64,
        user: User,
        body: String,
        created_at: String,
        updated_at: String,
        html_url: String,
        pull_request_url: String,
        commit_id: String,
        path: String,
        position: Option<i32>,
        line: Option<i32>,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
        id: i64,
    }

    let mut all_comments = Vec::new();
    let mut page = 1;
    loop {
        let review_comments_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/comments?per_page=100&page={}",
            owner, repo, page
        );
        let response = client
            .get(&review_comments_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch review comments: {}", e))?;
        if !response.status().is_success() {
            return Err(format!(
                "GitHub API error for review comments: {}",
                response.status()
            ));
        }
        let comments: Vec<ReviewComment> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse review comments response: {}", e))?;
        if comments.is_empty() {
            break;
        }
        for comment in comments {
            all_comments.push(CommentInfo {
                id: comment.id,
                comment_type: CommentType::ReviewComment,
                user_login: comment.user.login,
                user_id: comment.user.id,
                body: comment.body,
                created_at: comment.created_at,
                updated_at: comment.updated_at,
                html_url: comment.html_url,
                issue_number: None,
                pull_request_number: None,
                commit_id: Some(comment.commit_id.clone()),
                path: Some(comment.path.clone()),
                position: comment.position,
                line: comment.line,
                commit_sha: None,
            });
        }
        page += 1;
        if let Some(max) = max_pages {
            if page > max {
                break;
            }
        }
    }
    Ok(all_comments)
}

/// Fetches commit comments for a repository
async fn fetch_commit_comments(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    max_pages: Option<usize>,
) -> Result<Vec<CommentInfo>, String> {
    #[derive(Deserialize)]
    struct CommitComment {
        id: i64,
        user: User,
        body: String,
        created_at: String,
        updated_at: String,
        html_url: String,
        commit_id: String,
        path: Option<String>,
        position: Option<i32>,
        line: Option<i32>,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
        id: i64,
    }

    let mut all_comments = Vec::new();
    let mut page = 1;
    loop {
        let commit_comments_url = format!(
            "https://api.github.com/repos/{}/{}/comments?per_page=100&page={}",
            owner, repo, page
        );
        let response = client
            .get(&commit_comments_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch commit comments: {}", e))?;
        if !response.status().is_success() {
            return Err(format!(
                "GitHub API error for commit comments: {}",
                response.status()
            ));
        }
        let comments: Vec<CommitComment> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse commit comments response: {}", e))?;
        if comments.is_empty() {
            break;
        }
        for comment in comments {
            all_comments.push(CommentInfo {
                id: comment.id,
                comment_type: CommentType::Commit,
                user_login: comment.user.login,
                user_id: comment.user.id,
                body: comment.body,
                created_at: comment.created_at,
                updated_at: comment.updated_at,
                html_url: comment.html_url,
                issue_number: None,
                pull_request_number: None,
                commit_id: None,
                path: comment.path.clone(),
                position: comment.position,
                line: comment.line,
                commit_sha: Some(comment.commit_id.clone()),
            });
        }
        page += 1;
        if let Some(max) = max_pages {
            if page > max {
                break;
            }
        }
    }
    Ok(all_comments)
}
