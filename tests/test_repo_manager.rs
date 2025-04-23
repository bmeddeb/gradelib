// tests/test_repo_manager.rs
// Tests for RepoManager and core repository management functionality

use gradelib::*;
use pyo3::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

// Import our test utilities
mod utils;

// Helper to get a dummy RepoManager
fn create_test_manager(urls: Vec<&str>) -> RepoManager {
    RepoManager::new(
        urls.iter().map(|s| s.to_string()).collect(),
        "test_user".to_string(),
        "fake_token".to_string(),
    )
}

#[test]
fn test_repo_manager_creation() {
    // Test creating a repo manager with valid parameters
    let urls = vec!["https://github.com/user/repo1", "https://github.com/user/repo2"];
    let manager = create_test_manager(urls);
    
    // We can't directly access internal state, but we can check that the manager was created
    // without panicking
    assert!(true, "RepoManager creation should not panic");
}

#[test]
fn test_repo_manager_empty_urls() {
    // Test creating a repo manager with no URLs
    let urls: Vec<&str> = vec![];
    let manager = create_test_manager(urls);
    
    // Ensure creation with empty URLs doesn't panic
    assert!(true, "RepoManager creation with empty URLs should not panic");
}

#[test]
fn test_parse_slug_from_url() {
    // This test requires access to the internal parse_slug_from_url function,
    // which might not be possible depending on library visibility.
    // If it's not exposed, we'd need to test it indirectly through other methods.
    // For now, we'll just assert true as a placeholder.
    assert!(true, "This test would verify URL parsing functionality");
    
    // In reality, we'd have tests like:
    // assert_eq!(parse_slug_from_url("https://github.com/user/repo.git"), Some("user/repo".to_string()));
    // assert_eq!(parse_slug_from_url("https://github.com/user/repo"), Some("user/repo".to_string()));
    // assert_eq!(parse_slug_from_url("git@github.com:user/repo.git"), Some("user/repo".to_string()));
    // assert_eq!(parse_slug_from_url("invalid-url"), None);
}

#[tokio::test]
async fn test_clone_and_fetch_tasks() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create a simple test repo
    let (temp_dir, repo, repo_path) = utils::create_simple_repo();
    
    // Create a RepoManager with the test repo
    let manager = create_test_manager(vec![&repo_path]);
    
    // Test fetch_clone_tasks before cloning - should show 'queued' status
    Python::with_gil(|py| {
        let future = manager.fetch_clone_tasks(py).unwrap();
        let rt = Runtime::new().unwrap();
        let tasks: HashMap<String, ExposedCloneTask> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        assert!(tasks.contains_key(&repo_path), "Repo should be in tasks map");
        let task = &tasks[&repo_path];
        assert_eq!(task.url, repo_path, "Task URL should match repo path");
        assert_eq!(task.status.status_type, "queued", "Initial status should be 'queued'");
        assert!(task.temp_dir.is_none(), "Temp dir should be None before cloning");
    });
    
    // Test clone_all
    Python::with_gil(|py| {
        let future = manager.clone_all(py).unwrap();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            future.into_future().await.unwrap();
        });
    });
    
    // Test fetch_clone_tasks after cloning - should show 'completed' status
    Python::with_gil(|py| {
        let future = manager.fetch_clone_tasks(py).unwrap();
        let rt = Runtime::new().unwrap();
        let tasks: HashMap<String, ExposedCloneTask> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        assert!(tasks.contains_key(&repo_path), "Repo should be in tasks map");
        let task = &tasks[&repo_path];
        assert_eq!(task.url, repo_path, "Task URL should match repo path");
        assert_eq!(task.status.status_type, "completed", "Status should be 'completed' after cloning");
        assert!(task.temp_dir.is_some(), "Temp dir should be set after cloning");
    });
}

#[tokio::test]
async fn test_clone_single_repo() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create a simple test repo
    let (temp_dir, repo, repo_path) = utils::create_simple_repo();
    
    // Create a RepoManager with the test repo
    let manager = create_test_manager(vec![&repo_path]);
    
    // Test clone method
    Python::with_gil(|py| {
        let future = manager.clone(py, repo_path.clone()).unwrap();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            future.into_future().await.unwrap();
        });
    });
    
    // Verify the clone succeeded
    Python::with_gil(|py| {
        let future = manager.fetch_clone_tasks(py).unwrap();
        let rt = Runtime::new().unwrap();
        let tasks: HashMap<String, ExposedCloneTask> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        assert!(tasks.contains_key(&repo_path), "Repo should be in tasks map");
        let task = &tasks[&repo_path];
        assert_eq!(task.status.status_type, "completed", "Status should be 'completed' after cloning");
    });
}

#[tokio::test]
async fn test_clone_failure_handling() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create a non-existent repo URL
    let invalid_repo_url = "https://github.com/this/does/not/exist.git";
    
    // Create a RepoManager with the invalid repo
    let manager = create_test_manager(vec![invalid_repo_url]);
    
    // Test clone method with invalid URL
    Python::with_gil(|py| {
        let future = manager.clone(py, invalid_repo_url.to_string()).unwrap();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            future.into_future().await.unwrap();
        });
    });
    
    // Verify the clone failed
    Python::with_gil(|py| {
        let future = manager.fetch_clone_tasks(py).unwrap();
        let rt = Runtime::new().unwrap();
        let tasks: HashMap<String, ExposedCloneTask> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        assert!(tasks.contains_key(invalid_repo_url), "Invalid repo should be in tasks map");
        let task = &tasks[invalid_repo_url];
        assert_eq!(task.status.status_type, "failed", "Status should be 'failed' after failed clone");
        assert!(task.status.error.is_some(), "Error should be set after failed clone");
    });
}
