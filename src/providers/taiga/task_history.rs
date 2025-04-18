use crate::providers::taiga::client::TaigaClient;
use crate::providers::taiga::common::TaigaError;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

/// User information from history events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryUserInfo {
    pub pk: i64, // User ID
    pub username: String,
    pub name: Option<String>,
    pub photo: Option<String>,
    pub is_active: Option<bool>,
}

/// Values that were changed in a history event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuesDiff {
    pub status: Option<Vec<String>>,
    pub assigned_to: Option<Vec<String>>,
    pub subject: Option<Vec<String>>,
    pub taskboard_order: Option<Vec<String>>,
    // Other possible fields that might change
}

/// Single history event for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryEvent {
    pub id: i64,
    pub user: HistoryUserInfo,
    pub created_at: String,
    pub comment: Option<String>,
    pub delete_comment_date: Option<String>,
    pub edit_comment_date: Option<String>,
    pub comment_versions: Option<Vec<String>>,
    pub values_diff: ValuesDiff,
    #[serde(rename = "type")]
    pub event_type: i32, // Event type
    pub is_hidden: bool,
    pub is_snapshot: bool,
}

/// Enum to represent event types for task history
pub enum TaskHistoryEventType {
    StatusChange = 1,
    AssignmentChange = 2,
    SubjectChange = 3,
    Other = 4,
}

impl From<i32> for TaskHistoryEventType {
    fn from(value: i32) -> Self {
        match value {
            1 => TaskHistoryEventType::StatusChange,
            2 => TaskHistoryEventType::AssignmentChange,
            3 => TaskHistoryEventType::SubjectChange,
            _ => TaskHistoryEventType::Other,
        }
    }
}

/// Fetch history for a specific task
pub async fn fetch_task_history(
    client: &TaigaClient,
    task_id: i64,
) -> Result<Vec<TaskHistoryEvent>, TaigaError> {
    let endpoint = format!("history/task/{}", task_id);
    let response = client
        .get(&endpoint)
        .await
        .map_err(|e| TaigaError::ApiError(e))?;

    serde_json::from_str(&response).map_err(|e| TaigaError::ParseError(e.to_string()))
}

/// Fetch history for multiple tasks concurrently
pub async fn fetch_task_histories_concurrently(
    client: &TaigaClient,
    task_ids: Vec<i64>,
) -> Result<HashMap<i64, Vec<TaskHistoryEvent>>, TaigaError> {
    let mut futures = Vec::new();

    for task_id in task_ids {
        let client_clone = client.clone();
        let future = task::spawn(async move {
            let result = fetch_task_history(&client_clone, task_id).await;
            (task_id, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut task_histories_map = HashMap::new();

    for result in results {
        match result {
            Ok((task_id, Ok(history))) => {
                task_histories_map.insert(task_id, history);
            }
            Ok((task_id, Err(_))) => {
                // Skip errors for individual task histories
                task_histories_map.insert(task_id, Vec::new());
            }
            Err(_) => {
                // Skip task join errors
                continue;
            }
        }
    }

    Ok(task_histories_map)
}
