use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;

/// Status of a GitHub API task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task has not started or is in queue
    Idle,
    
    /// Task is currently in progress
    InProgress { 
        /// Percentage of completion (0-100)
        completion_percentage: f32 
    },
    
    /// Task has completed successfully
    Completed {
        /// When the task was completed (Unix timestamp)
        completed_at: u64
    },
    
    /// Task failed with an error
    Failed { 
        /// Error message
        error: String,
        /// When the task failed (Unix timestamp) 
        failed_at: u64
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Idle
    }
}

/// Information about a GitHub API task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Unique ID for the task
    pub id: String,
    
    /// Type of the task (issues, pull_requests, etc.)
    pub task_type: String,
    
    /// Repository URL or other identifier
    pub resource: String,
    
    /// Current status of the task
    pub status: TaskStatus,
    
    /// When the task was created (Unix timestamp)
    pub created_at: u64,
    
    /// When the task was last updated (Unix timestamp)
    pub updated_at: u64,
}

// Global registry of all tasks
static TASK_REGISTRY: Lazy<Mutex<HashMap<String, TaskInfo>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// Creates a new task ID for the given task type and resource
pub fn create_task_id(task_type: &str, resource: &str, operation: &str) -> String {
    format!("{}:{}:{}", task_type, resource, operation)
}

/// Registers a new task with Idle status
pub fn register_task(task_type: &str, resource: &str, operation: &str) -> String {
    let task_id = create_task_id(task_type, resource, operation);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let task_info = TaskInfo {
        id: task_id.clone(),
        task_type: task_type.to_string(),
        resource: resource.to_string(),
        status: TaskStatus::Idle,
        created_at: now,
        updated_at: now,
    };
    
    let mut registry = TASK_REGISTRY.lock().unwrap();
    registry.insert(task_id.clone(), task_info);
    
    task_id
}

/// Updates the status of an existing task
pub fn update_task_status(task_id: &str, status: TaskStatus) -> bool {
    let mut registry = TASK_REGISTRY.lock().unwrap();
    
    if let Some(task) = registry.get_mut(task_id) {
        task.status = status;
        task.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        true
    } else {
        false
    }
}

/// Gets information about a task
pub fn get_task_info(task_id: &str) -> Option<TaskInfo> {
    let registry = TASK_REGISTRY.lock().unwrap();
    registry.get(task_id).cloned()
}

/// Sets a task to In Progress with the given completion percentage
pub fn set_task_in_progress(task_id: &str, completion_percentage: f32) -> bool {
    update_task_status(
        task_id, 
        TaskStatus::InProgress { 
            completion_percentage: completion_percentage.max(0.0).min(100.0) 
        }
    )
}

/// Sets a task to Completed
pub fn set_task_completed(task_id: &str) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
        
    update_task_status(task_id, TaskStatus::Completed { completed_at: now })
}

/// Sets a task to Failed with the given error message
pub fn set_task_failed(task_id: &str, error: &str) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
        
    update_task_status(
        task_id, 
        TaskStatus::Failed { 
            error: error.to_string(),
            failed_at: now
        }
    )
}

/// Lists all tasks of a specific type
pub fn list_tasks_by_type(task_type: &str) -> Vec<TaskInfo> {
    let registry = TASK_REGISTRY.lock().unwrap();
    
    registry
        .values()
        .filter(|task| task.task_type == task_type)
        .cloned()
        .collect()
}

/// Lists all tasks for a specific resource
pub fn list_tasks_for_resource(resource: &str) -> Vec<TaskInfo> {
    let registry = TASK_REGISTRY.lock().unwrap();
    
    registry
        .values()
        .filter(|task| task.resource == resource)
        .cloned()
        .collect()
}

/// Clears all completed tasks older than the specified duration in seconds
pub fn clear_old_completed_tasks(older_than_seconds: u64) -> usize {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
        
    let mut registry = TASK_REGISTRY.lock().unwrap();
    
    let to_remove: Vec<String> = registry
        .iter()
        .filter_map(|(id, task)| {
            if let TaskStatus::Completed { completed_at } = task.status {
                if now - completed_at > older_than_seconds {
                    return Some(id.clone());
                }
            }
            None
        })
        .collect();
        
    let count = to_remove.len();
    
    for id in to_remove {
        registry.remove(&id);
    }
    
    count
}
