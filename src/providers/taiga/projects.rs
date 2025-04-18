use crate::providers::taiga::client::TaigaClient;
use crate::providers::taiga::common::{TaigaError, TaigaProject};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

/// Response structure for project members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: i64,
    pub user: i64,
    pub role: i64,
    pub role_name: String,
    pub full_name: String,
    pub is_active: Option<bool>,
    pub email: Option<String>,
    pub photo: Option<String>,
    pub username: Option<String>,
}

/// Fetch a project by its slug
pub async fn fetch_project_by_slug(
    client: &TaigaClient,
    slug: &str,
) -> Result<TaigaProject, TaigaError> {
    let endpoint = format!("projects/by_slug?slug={}", slug);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch project members by project ID
pub async fn fetch_project_members(
    client: &TaigaClient,
    project_id: i64,
) -> Result<Vec<MemberResponse>, TaigaError> {
    let endpoint = format!("memberships?project={}", project_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch multiple projects by their slugs concurrently
pub async fn fetch_projects_by_slugs_concurrently(
    client: &TaigaClient,
    slugs: Vec<String>,
) -> Result<HashMap<String, Result<TaigaProject, TaigaError>>, TaigaError> {
    let mut futures = Vec::new();

    for slug in slugs.clone() {
        let client_clone = client.clone();
        let slug_clone = slug.clone();
        let future = task::spawn(async move {
            let result = fetch_project_by_slug(&client_clone, &slug_clone).await;
            (slug_clone, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut projects_map = HashMap::new();

    for result in results {
        match result {
            Ok((slug, project_result)) => {
                projects_map.insert(slug, project_result);
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(projects_map)
}

/// Fetch project members for multiple projects concurrently
pub async fn fetch_project_members_concurrently(
    client: &TaigaClient,
    project_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<MemberResponse>>, TaigaError> {
    let mut futures = Vec::new();

    for project_id in project_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_project_members(&client_clone, project_id).await;
            (project_id, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut members_map = HashMap::new();

    for result in results {
        match result {
            Ok((project_id, Ok(members))) => {
                members_map.insert(project_id, members);
            }
            Ok((project_id, Err(e))) => {
                return Err(TaigaError::ApiError(format!(
                    "Failed to fetch members for project {}: {}",
                    project_id, e
                )));
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(members_map)
}

/// Fetch a project with all its members by slug
pub async fn fetch_project_with_members(
    client: &TaigaClient,
    slug: &str,
) -> Result<(TaigaProject, Vec<MemberResponse>), TaigaError> {
    let project = fetch_project_by_slug(client, slug).await?;
    let members = fetch_project_members(client, project.id).await?;

    Ok((project, members))
}
