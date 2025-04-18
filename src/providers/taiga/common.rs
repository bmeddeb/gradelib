use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a Taiga project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaigaProject {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub created_date: String,
    pub modified_date: String,
    pub total_milestones: Option<i32>,
    pub total_userstories: Option<i32>,
    pub total_tasks: Option<i32>,
    pub total_issues: Option<i32>,
    pub is_private: bool,
}

/// Represents a Taiga user story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaigaUserStory {
    pub id: i64,
    #[serde(rename = "ref")]
    pub reference: i64,
    pub subject: String,
    pub description: String,
    pub created_date: String,
    pub modified_date: String,
    pub status: String,
    pub points: Option<f32>,
    pub total_comments: i32,
    pub total_attachments: i32,
    pub project: i64, // Project ID
}

/// Represents a Taiga task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaigaTask {
    pub id: i64,
    #[serde(rename = "ref")]
    pub reference: i64,
    pub subject: String,
    pub description: String,
    pub created_date: String,
    pub modified_date: String,
    pub status: String,
    pub user_story: Option<i64>, // User story ID
    pub project: i64,            // Project ID
}

/// Represents a Taiga issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaigaIssue {
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
    pub project: i64, // Project ID
}

/// Error type for Taiga operations
#[derive(Debug, Clone)]
pub enum TaigaError {
    ApiError(String),
    ParseError(String),
    NotFound(String),
    Unauthorized,
}

impl fmt::Display for TaigaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaigaError::ApiError(msg) => write!(f, "Taiga API error: {}", msg),
            TaigaError::ParseError(msg) => write!(f, "Taiga parse error: {}", msg),
            TaigaError::NotFound(msg) => write!(f, "Taiga resource not found: {}", msg),
            TaigaError::Unauthorized => write!(f, "Taiga unauthorized access"),
        }
    }
}
