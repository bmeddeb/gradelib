use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use tokio::task;
use serde::{Deserialize, Serialize};

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
pub async fn fetch_collaborators(
    repo_urls: Vec<String>,
    _github_username: &str, // Prefix with underscore to indicate intentional non-use
    github_token: &str,
) -> Result<HashMap<String, Vec<CollaboratorInfo>>, String> {
    // Create a GitHub client
    let client = match create_github_client(github_token) {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to create GitHub client: {}", e)),
    };

    // Fetch collaborators for all repositories concurrently
    let mut tasks = Vec::new();
    
    for repo_url in repo_urls {
        let client = client.clone();
        let token = github_token.to_string();
        let url = repo_url.clone();
        
        let task = task::spawn(async move {
            let result = fetch_repo_collaborators(&client, &url, &token).await;
            (url, result)
        });
        
        tasks.push(task);
    }
    
    // Collect results
    let mut results = HashMap::new();
    for task in tasks {
        match task.await {
            Ok((repo_url, Ok(collaborators))) => {
                results.insert(repo_url, collaborators);
            }
            Ok((repo_url, Err(e))) => {
                eprintln!("Warning: Failed to fetch collaborators for {}: {}", repo_url, e);
                results.insert(repo_url, Vec::new());
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
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token)).unwrap(),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("gradelib-github-client/0.1.0"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
}

/// Fetches collaborators for a single repository
async fn fetch_repo_collaborators(
    client: &reqwest::Client,
    repo_url: &str,
    token: &str,
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

    // First, get the list of collaborators
    let collaborators_url = format!(
        "https://api.github.com/repos/{}/{}/collaborators",
        owner, repo
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

    // Now fetch detailed information for each collaborator
    let mut detailed_collaborators = Vec::new();
    for collab in collaborators {
        match fetch_user_details(client, &collab.login).await {
            Ok(user_info) => detailed_collaborators.push(user_info),
            Err(e) => {
                eprintln!("Warning: Failed to fetch details for {}: {}", collab.login, e);
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