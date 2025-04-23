// tests/test_commits.rs
// Tests for commit analysis functionality

use gradelib::*;
use pyo3::prelude::*;
use pyo3::types::PyList;
use tokio::runtime::Runtime;

// Import our test utilities
mod utils;

// Helper to create a test manager with a cloned repo that has multiple commits
async fn setup_test_manager_with_complex_repo() -> (RepoManager, String) {
    // Create a complex repository with multiple commits
    let (temp_dir, repo, repo_path) = utils::create_complex_repo();
    
    // Add a few more commits to have a richer history
    utils::add_file_and_commit(&repo, "docs/api.md", "# API Documentation\n\nThis is API documentation.", "Add API docs");
    utils::add_file_and_commit(&repo, "src/lib.rs", "pub fn hello() -> &'static str {\n    \"Hello, world!\"\n}", "Add library code");
    utils::add_file_and_commit(&repo, "tests/test.rs", "#[test]\nfn test_hello() {\n    assert_eq!(hello(), \"Hello, world!\");\n}", "Add tests");
    
    // Create a branch with separate commits
    utils::create_branch(&repo, "development");
    utils::checkout_branch(&repo, "development");
    utils::add_file_and_commit(&repo, "src/module.rs", "pub fn new_feature() {}", "Add new feature");
    
    // Go back to main branch
    let main_branch_name = if repo.find_branch("main", git2::BranchType::Local).is_ok() {
        "main"
    } else {
        "master"
    };
    utils::checkout_branch(&repo, main_branch_name);
    
    // Create a merge commit
    let merge_file = "merge.txt";
    utils::add_file_and_commit(&repo, merge_file, "This is a file added before merge", "Pre-merge commit");
    
    // Create a RepoManager and clone the repo
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
async fn test_analyze_commits() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up manager with a cloned repo that has multiple commits
    let (manager, repo_path) = setup_test_manager_with_complex_repo().await;
    
    // Test analyze_commits
    Python::with_gil(|py| {
        let future = manager.analyze_commits(py, repo_path.clone()).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            future.into_future().await.unwrap()
        });
        
        // Verify the result is a non-empty list
        assert!(result.is_instance_of::<PyList>(py).unwrap(), "Result should be a list");
        let commit_list = result.downcast::<PyList>(py).unwrap();
        assert!(commit_list.len() > 0, "Commit list should not be empty");
        
        // Check the structure of a commit info object
        let first_commit = commit_list.get_item(0).unwrap();
        let sha: String = first_commit.getattr("sha").unwrap().extract().unwrap();
        let message: String = first_commit.getattr("message").unwrap().extract().unwrap();
        let author_name: String = first_commit.getattr("author_name").unwrap().extract().unwrap();
        let author_email: String = first_commit.getattr("author_email").unwrap().extract().unwrap();
        
        // Verify the basic commit info fields
        assert!(!sha.is_empty(), "Commit SHA should not be empty");
        assert!(!message.is_empty(), "Commit message should not be empty");
        assert_eq!(author_name, "Test User", "Author name should match");
        assert_eq!(author_email, "test@example.com", "Author email should match");
        
        // Check if we have expected commits
        let mut found_initial = false;
        let mut found_api_docs = false;
        let mut found_library = false;
        
        for i in 0..commit_list.len() {
            let commit = commit_list.get_item(i).unwrap();
            let msg: String = commit.getattr("message").unwrap().extract().unwrap();
            
            if msg.contains("Initial commit") {
                found_initial = true;
            } else if msg.contains("Add API docs") {
                found_api_docs = true;
            } else if msg.contains("Add library code") {
                found_library = true;
            }
        }
        
        assert!(found_initial, "Should find 'Initial commit'");
        assert!(found_api_docs, "Should find 'Add API docs'");
        assert!(found_library, "Should find 'Add library code'");
    });
}

#[tokio::test]
async fn test_analyze_commits_empty_repo() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create an empty repository (initialize but don't add any commits)
    let temp_dir = utils::create_temp_dir();
    let repo_path = temp_dir.path();
    let repo = git2::Repository::init(repo_path).unwrap();
    let repo_path_str = repo_path.to_string_lossy().to_string();
    
    // Create a RepoManager and clone the repo
    let manager = RepoManager::new(
        vec![repo_path_str.clone()],
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
    
    // Test analyze_commits on the empty repo
    Python::with_gil(|py| {
        let future = manager.analyze_commits(py, repo_path_str.clone()).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let result = future.into_future().await;
            
            // For an empty repo with no commits, the function might either return an empty list
            // or raise an exception, depending on the implementation
            if result.is_err() {
                // If it raises an exception, we consider the test passed
                return Ok::<_, pyo3::PyErr>(());
            }
            
            // If it returns a value, it should be an empty list
            let commit_list = result.unwrap().downcast::<PyList>(py).unwrap();
            assert_eq!(commit_list.len(), 0, "Commit list for empty repo should be empty");
            
            Ok(())
        });
        
        assert!(result.is_ok(), "Test should complete without panic");
    });
}

#[tokio::test]
async fn test_analyze_commits_invalid_repo() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create a manager with a nonexistent repo
    let invalid_repo_url = "https://github.com/nonexistent/repo.git";
    let manager = RepoManager::new(
        vec![invalid_repo_url.to_string()],
        "test_user".to_string(),
        "fake_token".to_string(),
    );
    
    // Try to analyze commits on the nonexistent repo
    Python::with_gil(|py| {
        let future = manager.analyze_commits(py, invalid_repo_url.to_string()).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let result = future.into_future().await;
            assert!(result.is_err(), "Commit analysis on invalid repo should fail");
            
            // Just to satisfy the type system, return a dummy value
            Ok::<_, pyo3::PyErr>(())
        });
        
        assert!(result.is_ok(), "Test should complete without panic");
    });
}
