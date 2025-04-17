use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::traits::CollaboratorOperations;
use crate::common::types::CollaboratorInfo;
use crate::github::provider::GitHubProvider;

#[derive(Debug, Deserialize, Serialize)]
struct GitHubCollaborator {
    login: String,
    id: i64,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[async_trait]
impl CollaboratorOperations for GitHubProvider {
    async fn fetch_collaborators(
        &self,
        repo_urls: Vec<String>,
    ) -> Result<HashMap<String, Vec<CollaboratorInfo>>, String> {
        let client = reqwest::Client::new();
        let mut results = HashMap::new();

        for url in repo_urls {
            let repo_name = extract_repo_name(&url)?;
            let collaborators =
                fetch_repo_collaborators(&client, &repo_name, &self.username, &self.token).await?;
            results.insert(url, collaborators);
        }

        Ok(results)
    }
}

async fn fetch_repo_collaborators(
    client: &Client,
    repo_name: &str,
    username: &str,
    token: &str,
) -> Result<Vec<CollaboratorInfo>, String> {
    let url = format!("https://api.github.com/repos/{}/collaborators", repo_name);

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

    let gh_collaborators: Vec<GitHubCollaborator> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Convert to our internal type
    let collaborators = gh_collaborators
        .into_iter()
        .map(|collab| CollaboratorInfo {
            login: collab.login,
            github_id: collab.id,
            full_name: collab.name,
            email: collab.email,
            avatar_url: collab.avatar_url,
        })
        .collect();

    Ok(collaborators)
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
