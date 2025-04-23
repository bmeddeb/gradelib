// tests/test_blame.rs
// Tests for git blame functionality

use gradelib::*;
use pyo3::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

// Import our test utilities
mod utils;

// Helper to create a test manager with a cloned repo
async fn setup_test_manager_with_repo() -> (RepoManager, String) {
    // Create a repository with multiple files and commits
    let (temp_dir, repo, repo_path) = utils::create_complex_repo();
    
    // Create a manager and clone the repo
    let manager = RepoManager::new(
        vec![repo_path.clone()],
        "test_user".to_string(),
        "fake_token".to_string(),
    );
    
    // Clone the repo
    Python::with_gil(|py| {
        let future = manager.clone_all(py).unwrap();
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            future.into_future().await.unwrap();
        });
    });
    
    (manager, repo_path)
}

#[tokio::test]
async fn test_bulk_blame_single_file() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up manager with a cloned repo
    let (manager, repo_path) = setup_test_manager_with_repo().await;
    
    // Test bulk_blame on a single file
    Python::with_gil(|py| {
        let future = manager.bulk_blame(
            py, 
            repo_path.clone(), 
            vec!["README.md".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, Vec<HashMap<String, pyo3::PyObject>>> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains the file
        assert!(result.contains_key("README.md"), "Result should contain README.md");
        
        // Verify the blame results for README.md
        let blame_lines = &result["README.md"];
        assert!(!blame_lines.is_empty(), "Blame result should not be empty");
        
        // Check the structure of the first blame line
        let first_line = &blame_lines[0];
        assert!(first_line.contains_key("commit_id"), "Blame line should have commit_id");
        assert!(first_line.contains_key("author_name"), "Blame line should have author_name");
        assert!(first_line.contains_key("author_email"), "Blame line should have author_email");
        assert!(first_line.contains_key("line_content"), "Blame line should have line_content");
        assert!(first_line.contains_key("orig_line_no"), "Blame line should have orig_line_no");
        assert!(first_line.contains_key("final_line_no"), "Blame line should have final_line_no");
        
        // Extract some values to verify content
        let author_name: String = first_line["author_name"].extract(py).unwrap();
        let author_email: String = first_line["author_email"].extract(py).unwrap();
        let line_content: String = first_line["line_content"].extract(py).unwrap();
        
        assert_eq!(author_name, "Test User", "Author name should match");
        assert_eq!(author_email, "test@example.com", "Author email should match");
        assert!(line_content.contains("Complex Test Repository"), "Line content should match README content");
    });
}

#[tokio::test]
async fn test_bulk_blame_multiple_files() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up manager with a cloned repo
    let (manager, repo_path) = setup_test_manager_with_repo().await;
    
    // Test bulk_blame on multiple files
    Python::with_gil(|py| {
        let future = manager.bulk_blame(
            py, 
            repo_path.clone(), 
            vec!["README.md".to_string(), "src/main.rs".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, Vec<HashMap<String, pyo3::PyObject>>> = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Verify the result contains both files
        assert!(result.contains_key("README.md"), "Result should contain README.md");
        assert!(result.contains_key("src/main.rs"), "Result should contain src/main.rs");
        
        // Verify we have blame results for both files
        assert!(!result["README.md"].is_empty(), "README.md blame should not be empty");
        assert!(!result["src/main.rs"].is_empty(), "src/main.rs blame should not be empty");
        
        // Verify the content of the main.rs file's blame
        let main_rs_lines = &result["src/main.rs"];
        let line_content: String = main_rs_lines[0]["line_content"].extract(py).unwrap();
        assert!(line_content.contains("fn main()"), "main.rs should contain function declaration");
    });
}

#[tokio::test]
async fn test_blame_nonexistent_file() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up manager with a cloned repo
    let (manager, repo_path) = setup_test_manager_with_repo().await;
    
    // Test bulk_blame on a non-existent file
    Python::with_gil(|py| {
        let future = manager.bulk_blame(
            py, 
            repo_path.clone(), 
            vec!["nonexistent.txt".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            future.into_future().await.unwrap().extract::<HashMap<String, pyo3::PyObject>>(py)
        });
        
        // The extraction might fail if the result structure doesn't match HashMap<String, Vec<...>>
        // due to the error string being returned instead of a list of blame lines.
        // This is expected behavior for a non-existent file.
        assert!(result.is_err() || {
            let result_map = result.unwrap();
            result_map.contains_key("nonexistent.txt") && {
                let value = &result_map["nonexistent.txt"];
                value.is_instance_of::<pyo3::types::PyString>(py).unwrap()
            }
        }, "Result for nonexistent file should be an error string");
    });
}

#[tokio::test]
async fn test_blame_invalid_repo() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create a manager with a nonexistent repo
    let invalid_repo_url = "https://github.com/nonexistent/repo.git";
    let manager = RepoManager::new(
        vec![invalid_repo_url.to_string()],
        "test_user".to_string(),
        "fake_token".to_string(),
    );
    
    // Try to run bulk_blame on the nonexistent repo
    Python::with_gil(|py| {
        let future = manager.bulk_blame(
            py,
            invalid_repo_url.to_string(),
            vec!["README.md".to_string()]
        ).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let result = future.into_future().await;
            assert!(result.is_err(), "Blame on invalid repo should fail");
            
            // Just to satisfy the type system, return a dummy value
            Ok::<_, pyo3::PyErr>(())
        });
        
        assert!(result.is_ok(), "Test should complete without panic");
    });
}
