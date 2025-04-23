// tests/test_github_api.rs
// Tests for GitHub API integration functionality

use gradelib::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::env;
use tokio::runtime::Runtime;

// Import our mock utilities
mod mocks;
mod utils;

// Helper function to set up a RepoManager with our mock GitHub API
fn setup_manager_with_mock_api(mock_server_url: &str) -> RepoManager {
    // Create a manager with a fake repo pointing to our mock server
    let repo_urls = vec![
        format!("https://github.com/test-owner/test-repo")
    ];
    
    // Set environment variables to use our mock server
    env::set_var("GITHUB_API_BASE_URL", mock_server_url);
    
    RepoManager::new(
        repo_urls,
        "test-user".to_string(),
        "fake-token".to_string(),
    )
}

#[tokio::test]
async fn test_fetch_collaborators() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock the collaborators endpoint
    let _mock = mocks::github_api::mock_collaborators(&server, "test-owner", "test-repo");
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_collaborators
    Python::with_gil(|py| {
        let future = manager.fetch_collaborators(
            py,
            vec!["https://github.com/test-owner/test-repo".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, Vec<HashMap<String, pyo3::PyObject>>> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains our repo
        assert!(result.contains_key("https://github.com/test-owner/test-repo"), 
               "Result should contain our test repo");
        
        // Verify we have collaborator data
        let collaborators = &result["https://github.com/test-owner/test-repo"];
        assert_eq!(collaborators.len(), 2, "Should have 2 collaborators");
        
        // Check first collaborator data
        let collaborator1 = &collaborators[0];
        let login: String = collaborator1["login"].extract(py).unwrap();
        let github_id: u64 = collaborator1["github_id"].extract(py).unwrap();
        assert_eq!(login, "test-user1", "First collaborator should be test-user1");
        assert_eq!(github_id, 12345, "First collaborator should have ID 12345");
        
        // Check if full_name and email are correctly extracted for first collaborator
        let full_name: Option<String> = collaborator1["full_name"].extract(py).unwrap();
        let email: Option<String> = collaborator1["email"].extract(py).unwrap();
        assert_eq!(full_name, Some("Test User 1".to_string()), "First collaborator should have name 'Test User 1'");
        assert_eq!(email, Some("test-user1@example.com".to_string()), "First collaborator should have email 'test-user1@example.com'");
        
        // Check second collaborator data
        let collaborator2 = &collaborators[1];
        let login2: String = collaborator2["login"].extract(py).unwrap();
        assert_eq!(login2, "test-user2", "Second collaborator should be test-user2");
        
        // Check if null values are correctly extracted as None
        let full_name2: Option<String> = collaborator2["full_name"].extract(py).unwrap();
        let email2: Option<String> = collaborator2["email"].extract(py).unwrap();
        assert_eq!(full_name2, None, "Second collaborator should have null name");
        assert_eq!(email2, None, "Second collaborator should have null email");
    });
}

#[tokio::test]
async fn test_fetch_issues() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock the issues endpoint
    let _mock = mocks::github_api::mock_issues(&server, "test-owner", "test-repo", None);
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_issues
    Python::with_gil(|py| {
        let future = manager.fetch_issues(
            py,
            vec!["https://github.com/test-owner/test-repo".to_string()],
            None
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, pyo3::PyObject> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains our repo
        assert!(result.contains_key("https://github.com/test-owner/test-repo"), 
               "Result should contain our test repo");
        
        // Verify we have issue data
        let issues_obj = &result["https://github.com/test-owner/test-repo"];
        assert!(issues_obj.is_instance_of::<PyList>(py).unwrap(), 
               "Issues should be a list");
        
        let issues = issues_obj.downcast::<PyList>(py).unwrap();
        assert_eq!(issues.len(), 2, "Should have 2 issues");
        
        // Check first issue data
        let issue1 = issues.get_item(0).unwrap();
        let number: u32 = issue1.getattr("number").unwrap().extract().unwrap();
        let title: String = issue1.getattr("title").unwrap().extract().unwrap();
        let state: String = issue1.getattr("state").unwrap().extract().unwrap();
        
        assert_eq!(number, 1, "First issue should have number 1");
        assert_eq!(title, "Test issue 1", "First issue should have correct title");
        assert_eq!(state, "open", "First issue should be open");
        
        // Check if labels are correctly extracted
        let labels_obj = issue1.getattr("labels").unwrap();
        let labels: Vec<String> = labels_obj.extract().unwrap();
        assert_eq!(labels.len(), 1, "First issue should have 1 label");
        assert_eq!(labels[0], "bug", "First issue should have 'bug' label");
        
        // Check second issue data - verify it's closed
        let issue2 = issues.get_item(1).unwrap();
        let number2: u32 = issue2.getattr("number").unwrap().extract().unwrap();
        let state2: String = issue2.getattr("state").unwrap().extract().unwrap();
        let closed_at: Option<String> = issue2.getattr("closed_at").unwrap().extract().unwrap();
        
        assert_eq!(number2, 2, "Second issue should have number 2");
        assert_eq!(state2, "closed", "Second issue should be closed");
        assert!(closed_at.is_some(), "Second issue should have closed_at date");
    });
}

#[tokio::test]
async fn test_fetch_pull_requests() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock the pull requests endpoint
    let _mock = mocks::github_api::mock_pull_requests(&server, "test-owner", "test-repo", None);
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_pull_requests
    Python::with_gil(|py| {
        let future = manager.fetch_pull_requests(
            py,
            vec!["https://github.com/test-owner/test-repo".to_string()],
            None
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, pyo3::PyObject> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains our repo
        assert!(result.contains_key("https://github.com/test-owner/test-repo"), 
               "Result should contain our test repo");
        
        // Verify we have PR data
        let prs_obj = &result["https://github.com/test-owner/test-repo"];
        assert!(prs_obj.is_instance_of::<PyList>(py).unwrap(), 
               "PRs should be a list");
        
        let prs = prs_obj.downcast::<PyList>(py).unwrap();
        assert_eq!(prs.len(), 2, "Should have 2 PRs");
        
        // Check first PR data - should be open
        let pr1 = prs.get_item(0).unwrap();
        let number: u32 = pr1.getattr("number").unwrap().extract().unwrap();
        let title: String = pr1.getattr("title").unwrap().extract().unwrap();
        let state: String = pr1.getattr("state").unwrap().extract().unwrap();
        let merged: bool = pr1.getattr("merged").unwrap().extract().unwrap();
        
        assert_eq!(number, 3, "First PR should have number 3");
        assert_eq!(title, "Test PR 1", "First PR should have correct title");
        assert_eq!(state, "open", "First PR should be open");
        assert_eq!(merged, false, "First PR should not be merged");
        
        // Check second PR data - should be closed and merged
        let pr2 = prs.get_item(1).unwrap();
        let number2: u32 = pr2.getattr("number").unwrap().extract().unwrap();
        let state2: String = pr2.getattr("state").unwrap().extract().unwrap();
        let merged2: bool = pr2.getattr("merged").unwrap().extract().unwrap();
        let merged_at: Option<String> = pr2.getattr("merged_at").unwrap().extract().unwrap();
        
        assert_eq!(number2, 4, "Second PR should have number 4");
        assert_eq!(state2, "closed", "Second PR should be closed");
        assert_eq!(merged2, true, "Second PR should be merged");
        assert!(merged_at.is_some(), "Second PR should have merged_at date");
    });
}

#[tokio::test]
async fn test_fetch_code_reviews() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock the pull requests endpoint first (to get PR numbers)
    let _mock_prs = mocks::github_api::mock_pull_requests(&server, "test-owner", "test-repo", None);
    
    // Mock the code reviews endpoint
    let _mock_reviews = mocks::github_api::mock_code_reviews(&server, "test-owner", "test-repo", 3);
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_code_reviews
    Python::with_gil(|py| {
        let future = manager.fetch_code_reviews(
            py,
            vec!["https://github.com/test-owner/test-repo".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, pyo3::PyObject> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains our repo
        assert!(result.contains_key("https://github.com/test-owner/test-repo"), 
               "Result should contain our test repo");
        
        // Verify we have code review data as a dictionary of PR numbers
        let reviews_obj = &result["https://github.com/test-owner/test-repo"];
        assert!(reviews_obj.is_instance_of::<PyDict>(py).unwrap(), 
               "Reviews should be a dictionary");
        
        let reviews_dict = reviews_obj.downcast::<PyDict>(py).unwrap();
        
        // Check reviews for PR #3
        let pr3_reviews_obj = reviews_dict.get_item("3").unwrap();
        assert!(pr3_reviews_obj.is_instance_of::<PyList>(py).unwrap(), 
               "PR #3 reviews should be a list");
        
        let pr3_reviews = pr3_reviews_obj.downcast::<PyList>(py).unwrap();
        assert_eq!(pr3_reviews.len(), 2, "PR #3 should have 2 reviews");
        
        // Check first review data
        let review1 = pr3_reviews.get_item(0).unwrap();
        let id: u64 = review1.getattr("id").unwrap().extract().unwrap();
        let state: String = review1.getattr("state").unwrap().extract().unwrap();
        let user_login: String = review1.getattr("user_login").unwrap().extract().unwrap();
        
        assert_eq!(id, 12345, "First review should have ID 12345");
        assert_eq!(state, "APPROVED", "First review should be APPROVED");
        assert_eq!(user_login, "test-user2", "First review should be from test-user2");
        
        // Check second review data
        let review2 = pr3_reviews.get_item(1).unwrap();
        let state2: String = review2.getattr("state").unwrap().extract().unwrap();
        let user_login2: String = review2.getattr("user_login").unwrap().extract().unwrap();
        
        assert_eq!(state2, "CHANGES_REQUESTED", "Second review should be CHANGES_REQUESTED");
        assert_eq!(user_login2, "test-user1", "Second review should be from test-user1");
    });
}

#[tokio::test]
async fn test_fetch_comments() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock the various comments endpoints
    let _mock_issue_comments = mocks::github_api::mock_issue_comments(&server, "test-owner", "test-repo");
    let _mock_pr_comments = mocks::github_api::mock_pull_request_comments(&server, "test-owner", "test-repo");
    let _mock_commit_comments = mocks::github_api::mock_commit_comments(&server, "test-owner", "test-repo", "abcdef1234567890");
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_comments with all comment types
    Python::with_gil(|py| {
        let future = manager.fetch_comments(
            py,
            vec!["https://github.com/test-owner/test-repo".to_string()],
            Some(vec!["issue".to_string(), "commit".to_string(), "pullrequest".to_string(), "reviewcomment".to_string()])
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, pyo3::PyObject> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains our repo
        assert!(result.contains_key("https://github.com/test-owner/test-repo"), 
               "Result should contain our test repo");
        
        // Verify we have comment data
        let comments_obj = &result["https://github.com/test-owner/test-repo"];
        assert!(comments_obj.is_instance_of::<PyList>(py).unwrap(), 
               "Comments should be a list");
        
        let comments = comments_obj.downcast::<PyList>(py).unwrap();
        
        // We should have comments of different types
        let mut issue_comments = 0;
        let mut pr_comments = 0;
        let mut review_comments = 0;
        let mut commit_comments = 0;
        
        for i in 0..comments.len() {
            let comment = comments.get_item(i).unwrap();
            let comment_type: String = comment.getattr("comment_type").unwrap().extract().unwrap();
            
            match comment_type.as_str() {
                "issue" => issue_comments += 1,
                "pull_request" => pr_comments += 1,
                "review_comment" => review_comments += 1,
                "commit" => commit_comments += 1,
                _ => ()
            }
            
            // Check basic comment properties
            assert!(comment.getattr("id").unwrap().extract::<u64>().is_ok(), "Comment should have ID");
            assert!(comment.getattr("body").unwrap().extract::<String>().is_ok(), "Comment should have body");
            assert!(comment.getattr("user_login").unwrap().extract::<String>().is_ok(), "Comment should have user_login");
            assert!(comment.getattr("created_at").unwrap().extract::<String>().is_ok(), "Comment should have created_at");
        }
        
        // We should have at least some comments of each type
        // The exact counts would depend on our mock implementations
        assert!(issue_comments > 0, "Should have issue comments");
        assert!(pr_comments > 0, "Should have PR comments");
        assert!(review_comments > 0, "Should have review comments");
        assert!(commit_comments > 0, "Should have commit comments");
    });
}

#[tokio::test]
async fn test_error_handling() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock a 404 Not Found response
    let _mock = mocks::github_api::mock_not_found(&server, "/repos/test-owner/nonexistent-repo/collaborators");
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_collaborators with a nonexistent repo
    Python::with_gil(|py| {
        let future = manager.fetch_collaborators(
            py,
            vec!["https://github.com/test-owner/nonexistent-repo".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, pyo3::PyObject> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains our repo
        assert!(result.contains_key("https://github.com/test-owner/nonexistent-repo"), 
               "Result should contain our test repo");
        
        // The value should be an error string
        let value = &result["https://github.com/test-owner/nonexistent-repo"];
        assert!(value.is_instance_of::<pyo3::types::PyString>(py).unwrap(), 
               "Result for nonexistent repo should be an error string");
        
        let error_msg: String = value.extract(py).unwrap();
        assert!(error_msg.contains("Not Found"), "Error message should indicate not found");
    });
}

#[tokio::test]
async fn test_unauthorized_handling() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock a 401 Unauthorized response
    let _mock = mocks::github_api::mock_unauthorized(&server, "/repos/test-owner/test-repo/collaborators");
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_collaborators with invalid credentials
    Python::with_gil(|py| {
        let future = manager.fetch_collaborators(
            py,
            vec!["https://github.com/test-owner/test-repo".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, pyo3::PyObject> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // The value should be an error string
        let value = &result["https://github.com/test-owner/test-repo"];
        assert!(value.is_instance_of::<pyo3::types::PyString>(py).unwrap(), 
               "Result for unauthorized request should be an error string");
        
        let error_msg: String = value.extract(py).unwrap();
        assert!(error_msg.contains("Bad credentials"), "Error message should indicate bad credentials");
    });
}

#[tokio::test]
async fn test_rate_limit_handling() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up the mock server
    let server = mocks::github_api::setup_mock_server();
    let mock_url = server.url();
    
    // Mock a 403 Rate Limited response
    let _mock = mocks::github_api::mock_rate_limited(&server, "/repos/test-owner/test-repo/collaborators");
    
    // Create a RepoManager configured to use our mock server
    let manager = setup_manager_with_mock_api(&mock_url);
    
    // Test fetch_collaborators with rate limiting
    Python::with_gil(|py| {
        let future = manager.fetch_collaborators(
            py,
            vec!["https://github.com/test-owner/test-repo".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, pyo3::PyObject> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // The value should be an error string
        let value = &result["https://github.com/test-owner/test-repo"];
        assert!(value.is_instance_of::<pyo3::types::PyString>(py).unwrap(), 
               "Result for rate limited request should be an error string");
        
        let error_msg: String = value.extract(py).unwrap();
        assert!(error_msg.contains("API rate limit exceeded"), "Error message should indicate rate limiting");
    });
}
