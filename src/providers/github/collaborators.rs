use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

use crate::repo::parse_slug_from_url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    pub login: String,
    pub github_id: i64,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Fetches collaborator information for multiple repositories concurrently
///
/// For each input repo URL, returns either a list of collaborators or an error string.
/// If the GitHub client cannot be created, all URLs are mapped to the error string.
pub async fn fetch_collaborators(
    repo_urls: Vec<String>,
    _github_username: &str, // Prefix with underscore to indicate intentional non-use
    github_token: &str,
    max_pages: Option<usize>,
) -> Result<HashMap<String, Result<Vec<CollaboratorInfo>, String>>, String> {
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

    // Fetch collaborators for all repositories concurrently
    let mut tasks = Vec::new();

    for repo_url in repo_urls {
        let client = client.clone();
        let token = github_token.to_string();
        let url = repo_url.clone();

        let task = task::spawn(async move {
            let result = fetch_repo_collaborators(&client, &url, &token, max_pages).await;
            (url, result)
        });

        tasks.push(task);
    }

    // Collect results
    let mut results = HashMap::new();
    for task in tasks {
        match task.await {
            Ok((repo_url, Ok(collaborators))) => {
                results.insert(repo_url, Ok(collaborators));
            }
            Ok((repo_url, Err(e))) => {
                eprintln!(
                    "Warning: Failed to fetch collaborators for {}: {}",
                    repo_url, e
                );
                results.insert(repo_url, Err(e));
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

/// Fetches collaborators for a single repository
async fn fetch_repo_collaborators(
    client: &reqwest::Client,
    repo_url: &str,
    _token: &str, // Prefix with underscore to indicate intentional non-use
    max_pages: Option<usize>,
) -> Result<Vec<CollaboratorInfo>, String> {
    // Parse owner/repo from URL using existing function
    let slug = parse_slug_from_url(repo_url)
        .ok_or_else(|| format!("Invalid repository URL format: {}", repo_url))?;
    let parts: Vec<&str> = slug.split('/').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid repository slug format: {}", slug));
    }
    let owner = parts[0];
    let repo = parts[1];
    let mut page = 1;
    let mut all_collaborators = Vec::new();
    loop {
        let collaborators_url = format!(
            "https://api.github.com/repos/{}/{}/collaborators?per_page=100&page={}",
            owner, repo, page
        );
        #[derive(Deserialize)]
        struct CollaboratorBasic {
            login: String,
        }
        let collaborators_response = client
            .get(&collaborators_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch collaborators: {}", e))?;
        if !collaborators_response.status().is_success() {
            return Err(format!(
                "GitHub API error: {}",
                collaborators_response.status()
            ));
        }
        let collaborators: Vec<CollaboratorBasic> = collaborators_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse collaborators response: {}", e))?;
        let len = collaborators.len();
        if len == 0 {
            break;
        }
        let mut should_break = false;
        if let Some(max) = max_pages {
            if page >= max {
                should_break = true;
            }
        }
        if len < 100 {
            should_break = true;
        }
        all_collaborators.extend(collaborators);
        if should_break {
            break;
        }
        page += 1;
    }
    // Now fetch detailed information for each collaborator
    let mut detailed_collaborators = Vec::new();
    for collab in all_collaborators {
        match fetch_user_details(client, &collab.login).await {
            Ok(user_info) => detailed_collaborators.push(user_info),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to fetch details for {}: {}",
                    collab.login, e
                );
                // Add basic info anyway
                detailed_collaborators.push(CollaboratorInfo {
                    login: collab.login,
                    github_id: 0, // Default/unknown
                    full_name: None,
                    email: None,
                    avatar_url: None,
                });
            }
        }
    }
    Ok(detailed_collaborators)
}

/// Fetches detailed information for a single user
async fn fetch_user_details(
    client: &reqwest::Client,
    username: &str,
) -> Result<CollaboratorInfo, String> {
    let user_url = format!("https://api.github.com/users/{}", username);

    #[derive(Deserialize)]
    struct UserResponse {
        login: String,
        id: i64,
        name: Option<String>,
        email: Option<String>,
        avatar_url: Option<String>,
    }

    let user_response = client
        .get(&user_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user details: {}", e))?;

    if !user_response.status().is_success() {
        return Err(format!("GitHub API error: {}", user_response.status()));
    }

    let user: UserResponse = user_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user response: {}", e))?;

    Ok(CollaboratorInfo {
        login: user.login,
        github_id: user.id,
        full_name: user.name,
        email: user.email,
        avatar_url: user.avatar_url,
    })
}
