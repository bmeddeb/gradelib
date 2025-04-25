use crate::providers::taiga::client::TaigaClient;
use crate::providers::taiga::common::{TaigaError, TaigaProject};
use crate::providers::taiga::projects::{
    fetch_project_by_slug, fetch_project_members, MemberResponse,
};
use crate::providers::taiga::sprints::{fetch_all_sprints, SprintResponse};
use crate::providers::taiga::task_history::{fetch_task_histories_concurrently, TaskHistoryEvent};
use crate::providers::taiga::tasks::{fetch_tasks_for_sprints_concurrently, TaskResponse};
use crate::providers::taiga::user_stories::{
    fetch_user_stories_for_sprints_concurrently, UserStoryResponse,
};
use futures::future::join_all;
use std::collections::HashMap;
use tokio::task;

/// Comprehensive structure containing all data for a Taiga project
#[derive(Debug)]
pub struct TaigaProjectData {
    pub project: TaigaProject,
    pub members: Vec<MemberResponse>,
    pub sprints: Vec<SprintResponse>,
    pub user_stories: HashMap<i64, Vec<UserStoryResponse>>,
    pub tasks: HashMap<i64, Vec<TaskResponse>>,
    pub task_histories: HashMap<i64, Vec<TaskHistoryEvent>>,
}

/// Fetch all data for a single Taiga project by slug
pub async fn fetch_complete_project_data(
    client: &TaigaClient,
    slug: &str,
) -> Result<TaigaProjectData, TaigaError> {
    // Fetch project and members
    let project = fetch_project_by_slug(client, slug).await?;
    let members = if client.is_authenticated() {
        fetch_project_members(client, project.id).await?
    } else {
        vec![] // Public project, skip fetching members
    };

    // Fetch all sprints for the project
    let sprints = fetch_all_sprints(client, project.id).await?;

    // Prepare data for fetching user stories and tasks for each sprint
    let sprint_ids: Vec<i64> = sprints.iter().map(|s| s.id).collect();
    let sprint_project_pairs: Vec<(i64, i64)> =
        sprints.iter().map(|s| (project.id, s.id)).collect();

    // Fetch user stories for all sprints
    let user_stories = if !sprint_ids.is_empty() {
        fetch_user_stories_for_sprints_concurrently(client, sprint_ids.clone()).await?
    } else {
        HashMap::new()
    };

    // Fetch tasks for all sprints
    let tasks = if !sprint_project_pairs.is_empty() {
        fetch_tasks_for_sprints_concurrently(client, sprint_project_pairs).await?
    } else {
        HashMap::new()
    };

    // Extract all task IDs for fetching task histories
    let mut task_ids = Vec::new();
    for (_, sprint_tasks) in &tasks {
        for task in sprint_tasks {
            task_ids.push(task.id);
        }
    }

    // Fetch task histories
    let task_histories = if !task_ids.is_empty() {
        fetch_task_histories_concurrently(client, task_ids).await?
    } else {
        HashMap::new()
    };

    Ok(TaigaProjectData {
        project,
        members,
        sprints,
        user_stories,
        tasks,
        task_histories,
    })
}

/// Fetch all data for multiple Taiga projects concurrently
///
/// For each input slug, returns either the project data or an error for that slug.
/// Task join errors or fetch errors for one slug do not abort the batch.
pub async fn fetch_taiga_data_concurrently(
    client: &TaigaClient,
    slugs: Vec<String>,
) -> Result<HashMap<String, Result<TaigaProjectData, TaigaError>>, TaigaError> {
    let mut futures = Vec::new();

    for slug in slugs.clone() {
        let client_clone = client.clone();
        let slug_clone = slug.clone();
        let future = task::spawn(async move {
            let result = fetch_complete_project_data(&client_clone, &slug_clone).await;
            (slug_clone, result)
        });
        futures.push(future);
    }

    let results = join_all(futures).await;
    let mut projects_map = HashMap::new();

    for (i, result) in results.into_iter().enumerate() {
        let slug = slugs[i].clone();
        match result {
            Ok((slug_from_task, project_result)) => {
                projects_map.insert(slug_from_task, project_result);
            }
            Err(e) => {
                // Insert an error for this slug only
                projects_map.insert(
                    slug,
                    Err(TaigaError::ApiError(format!("Task join error: {}", e))),
                );
            }
        }
    }

    Ok(projects_map)
}
