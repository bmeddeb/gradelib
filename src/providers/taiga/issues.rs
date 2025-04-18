use crate::providers::taiga::client::TaigaClient;
use crate::providers::taiga::common::TaigaError;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

/// Represents an issue in the Taiga API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueResponse {
    pub id: i64,
    #[serde(rename = "ref")]
    pub reference: i64,
    pub subject: String,
    pub description: String,
    pub created_date: String,
    pub modified_date: String,
    pub status: String,
    pub severity: String,
    pub priority: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub project: i64,
    pub milestone: Option<i64>, // Sprint ID if assigned
    pub is_closed: bool,
    pub assigned_to: Option<i64>,
}

/// Fetch issues for a specific project
pub async fn fetch_issues_for_project(
    client: &TaigaClient,
    project_id: i64,
) -> Result<Vec<IssueResponse>, TaigaError> {
    let endpoint = format!("issues?project={}", project_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch issues for a specific sprint
pub async fn fetch_issues_for_sprint(
    client: &TaigaClient,
    sprint_id: i64,
) -> Result<Vec<IssueResponse>, TaigaError> {
    let endpoint = format!("issues?milestone={}", sprint_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch a single issue by ID
pub async fn fetch_issue(client: &TaigaClient, issue_id: i64) -> Result<IssueResponse, TaigaError> {
    let endpoint = format!("issues/{}", issue_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch issues for multiple projects concurrently
pub async fn fetch_issues_for_projects_concurrently(
    client: &TaigaClient,
    project_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<IssueResponse>>, TaigaError> {
    let mut futures = Vec::new();

    for project_id in project_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_issues_for_project(&client_clone, project_id).await;
            (project_id, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut project_issues_map = HashMap::new();

    for result in results {
        match result {
            Ok((project_id, Ok(issues))) => {
                project_issues_map.insert(project_id, issues);
            }
            Ok((project_id, Err(e))) => {
                return Err(TaigaError::ApiError(format!(
                    "Failed to fetch issues for project {}: {}",
                    project_id, e
                )));
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(project_issues_map)
}

/// Fetch issues for multiple sprints concurrently
pub async fn fetch_issues_for_sprints_concurrently(
    client: &TaigaClient,
    sprint_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<IssueResponse>>, TaigaError> {
    let mut futures = Vec::new();

    for sprint_id in sprint_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_issues_for_sprint(&client_clone, sprint_id).await;
            (sprint_id, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut sprint_issues_map = HashMap::new();

    for result in results {
        match result {
            Ok((sprint_id, Ok(issues))) => {
                sprint_issues_map.insert(sprint_id, issues);
            }
            Ok((sprint_id, Err(e))) => {
                return Err(TaigaError::ApiError(format!(
                    "Failed to fetch issues for sprint {}: {}",
                    sprint_id, e
                )));
            }
            Err(e) => {
                return Err(TaigaError::ApiError(format!("Task join error: {}", e)));
            }
        }
    }

    Ok(sprint_issues_map)
}
