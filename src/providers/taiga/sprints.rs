use crate::providers::taiga::client::TaigaClient;
use crate::providers::taiga::common::TaigaError;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

/// Sprint/Milestone in Taiga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintResponse {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub project: i64,
    pub project_slug: Option<String>,
    pub project_name: Option<String>,
    pub estimated_start: String,
    pub estimated_finish: String,
    pub created_date: String,
    pub modified_date: String,
    pub closed: bool,
    pub disponibility: Option<f32>,
    pub total_points: Option<f32>,
    pub closed_points: Option<f32>,
}

/// Fetch both open and closed sprints for a project
pub async fn fetch_sprints(
    client: &TaigaClient,
    project_id: i64,
    closed: bool,
) -> Result<Vec<SprintResponse>, TaigaError> {
    let endpoint = format!("milestones?project={}&closed={}", project_id, closed);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch all sprints (both open and closed) for a project
pub async fn fetch_all_sprints(
    client: &TaigaClient,
    project_id: i64,
) -> Result<Vec<SprintResponse>, TaigaError> {
    // Fetch open sprints
    let open_sprints = fetch_sprints(client, project_id, false).await?;

    // Fetch closed sprints
    let closed_sprints = fetch_sprints(client, project_id, true).await?;

    // Combine results
    let mut all_sprints = Vec::with_capacity(open_sprints.len() + closed_sprints.len());
    all_sprints.extend(open_sprints);
    all_sprints.extend(closed_sprints);

    Ok(all_sprints)
}

/// Fetch a single sprint by ID
pub async fn fetch_sprint(
    client: &TaigaClient,
    sprint_id: i64,
) -> Result<SprintResponse, TaigaError> {
    let endpoint = format!("milestones/{}", sprint_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch all sprints for multiple projects concurrently
pub async fn fetch_all_sprints_concurrently(
    client: &TaigaClient,
    project_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<SprintResponse>>, TaigaError> {
    let mut futures = Vec::new();

    for project_id in project_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_all_sprints(&client_clone, project_id).await;
            (project_id, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut project_sprints_map = HashMap::new();

    for result in results {
        match result {
            Ok((project_id, Ok(sprints))) => {
                project_sprints_map.insert(project_id, sprints);
            }
            Ok((project_id, Err(e))) => {
                return Err(TaigaError::ApiError(format!(
                    "Failed to fetch sprints for project {}: {}",
                    project_id, e
                )));
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(project_sprints_map)
}
