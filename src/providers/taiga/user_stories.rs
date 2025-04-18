use crate::providers::taiga::client::TaigaClient;
use crate::providers::taiga::common::TaigaError;
use futures::future::join_all;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use tokio::task;

/// Helper function to deserialize status field that could be either a string or an integer
fn deserialize_status<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StatusValue {
        Int(i64),
        Str(String),
    }

    let value = StatusValue::deserialize(deserializer)?;
    Ok(match value {
        StatusValue::Int(i) => i.to_string(),
        StatusValue::Str(s) => s,
    })
}

/// User Story response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStoryResponse {
    pub id: i64,
    #[serde(rename = "ref")]
    pub reference: i64,
    pub subject: String,
    pub description: Option<String>,
    pub created_date: String,
    pub modified_date: String,
    #[serde(deserialize_with = "deserialize_status")]
    pub status: String,
    pub is_closed: bool,
    pub project: i64,
    pub milestone: Option<i64>, // Sprint ID
    pub points: Option<HashMap<String, f32>>,
    pub total_comments: i32,
    pub total_attachments: i32,
}

/// Fetch a single user story by ID
pub async fn fetch_user_story(
    client: &TaigaClient,
    user_story_id: i64,
) -> Result<UserStoryResponse, TaigaError> {
    let endpoint = format!("userstories/{}", user_story_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch user stories for a specific sprint
pub async fn fetch_user_stories_for_sprint(
    client: &TaigaClient,
    sprint_id: i64,
) -> Result<Vec<UserStoryResponse>, TaigaError> {
    let endpoint = format!("userstories?milestone={}", sprint_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch user stories for a specific project
pub async fn fetch_user_stories_for_project(
    client: &TaigaClient,
    project_id: i64,
) -> Result<Vec<UserStoryResponse>, TaigaError> {
    let endpoint = format!("userstories?project={}", project_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch user stories for multiple sprints concurrently
pub async fn fetch_user_stories_for_sprints_concurrently(
    client: &TaigaClient,
    sprint_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<UserStoryResponse>>, TaigaError> {
    let mut futures = Vec::new();

    for sprint_id in sprint_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_user_stories_for_sprint(&client_clone, sprint_id).await;
            (sprint_id, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut sprint_stories_map = HashMap::new();

    for result in results {
        match result {
            Ok((sprint_id, Ok(stories))) => {
                sprint_stories_map.insert(sprint_id, stories);
            }
            Ok((sprint_id, Err(e))) => {
                return Err(TaigaError::ApiError(format!(
                    "Failed to fetch user stories for sprint {}: {}",
                    sprint_id, e
                )));
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(sprint_stories_map)
}

/// Fetch user stories for multiple projects concurrently
pub async fn fetch_user_stories_for_projects_concurrently(
    client: &TaigaClient,
    project_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<UserStoryResponse>>, TaigaError> {
    let mut futures = Vec::new();

    for project_id in project_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_user_stories_for_project(&client_clone, project_id).await;
            (project_id, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut project_stories_map = HashMap::new();

    for result in results {
        match result {
            Ok((project_id, Ok(stories))) => {
                project_stories_map.insert(project_id, stories);
            }
            Ok((project_id, Err(e))) => {
                return Err(TaigaError::ApiError(format!(
                    "Failed to fetch user stories for project {}: {}",
                    project_id, e
                )));
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(project_stories_map)
}
