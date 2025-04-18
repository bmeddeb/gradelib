use crate::providers::taiga::client::TaigaClient;
use crate::providers::taiga::common::TaigaError;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

/// Response structure for fetching a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: i64,
    #[serde(rename = "ref")]
    pub reference: i64,
    pub subject: String,
    pub description: Option<String>,
    pub created_date: String,
    pub finished_date: Option<String>,
    pub is_closed: bool,
    pub assigned_to: Option<i64>,
    pub user_story: Option<i64>,
    pub milestone: Option<i64>,
    pub project: i64,
    pub user_story_extra_info: Option<UserStoryExtraInfo>,
}

/// Additional user story information included with task responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStoryExtraInfo {
    pub id: i64,
    pub subject: String,
    #[serde(rename = "ref")]
    pub reference: i64,
}

/// Fetch tasks for a specific sprint
pub async fn fetch_tasks_for_sprint(
    client: &TaigaClient,
    project_id: i64,
    sprint_id: i64,
) -> Result<Vec<TaskResponse>, TaigaError> {
    let endpoint = format!("tasks?project={}&milestone={}", project_id, sprint_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch user story details with full description
pub async fn fetch_user_story_description(
    client: &TaigaClient,
    user_story_id: i64,
) -> Result<HashMap<String, String>, TaigaError> {
    let endpoint = format!("userstories/{}", user_story_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch tasks for multiple sprints concurrently
pub async fn fetch_tasks_for_sprints_concurrently(
    client: &TaigaClient,
    sprints: Vec<(i64, i64)>, // Vector of (project_id, sprint_id) pairs
) -> Result<HashMap<i64, Vec<TaskResponse>>, TaigaError> {
    let mut tasks_futures = Vec::new();

    for (project_id, sprint_id) in sprints {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_tasks_for_sprint(&client_clone, project_id, sprint_id).await;
            (sprint_id, result)
        });
        tasks_futures.push(future);
    }

    let results = join_all(tasks_futures).await;
    let mut sprint_tasks_map = HashMap::new();

    for result in results {
        match result {
            Ok((sprint_id, Ok(tasks))) => {
                sprint_tasks_map.insert(sprint_id, tasks);
            }
            Ok((sprint_id, Err(e))) => {
                return Err(TaigaError::ApiError(format!(
                    "Failed to fetch tasks for sprint {}: {}",
                    sprint_id, e
                )));
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(sprint_tasks_map)
}

/// Fetch user story descriptions concurrently for multiple user stories
pub async fn fetch_user_story_descriptions_concurrently(
    client: &TaigaClient,
    user_story_ids: Vec<i64>,
) -> Result<HashMap<i64, HashMap<String, String>>, TaigaError> {
    let mut descriptions_futures = Vec::new();

    for us_id in user_story_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_user_story_description(&client_clone, us_id).await;
            (us_id, result)
        });
        descriptions_futures.push(future);
    }

    let results = join_all(descriptions_futures).await;
    let mut descriptions_map = HashMap::new();

    for result in results {
        match result {
            Ok((us_id, Ok(description))) => {
                descriptions_map.insert(us_id, description);
            }
            Ok((_, Err(_))) => {
                // Skip errors for individual user stories
                continue;
            }
            Err(_) => {
                // Skip task join errors
                continue;
            }
        }
    }

    Ok(descriptions_map)
}
