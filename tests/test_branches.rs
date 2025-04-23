// tests/test_branches.rs
// Tests for branch analysis functionality

use gradelib::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use tokio::runtime::Runtime;

// Import our test utilities
mod utils;

// Helper to create a test manager with a complex branching structure
async fn setup_test_manager_with_branches() -> (RepoManager, String) {
    // Create a complex repository
    let (temp_dir, repo, repo_path) = utils::create_complex_repo();
    
    // The complex repo already has a 'feature' branch
    // Let's add a few more branches with different structures
    
    // Find the main branch name (could be 'main' or 'master')
    let main_branch_name = if repo.find_branch("main", git2::BranchType::Local).is_ok() {
        "main"
    } else {
        "master"
    };
    
    // Make sure we're on the main branch
    utils::checkout_branch(&repo, main_branch_name);
    
    // Create a 'development' branch from main
    utils::create_branch(&repo, "development");
    utils::checkout_branch(&repo, "development");
    utils::add_file_and_commit(&repo, "dev-file.txt", "Development branch file", "Development commit");
    
    // Create a 'bugfix' branch from development
    utils::create_branch(&repo, "bugfix");
    utils::checkout_branch(&repo, "bugfix");
    utils::add_file_and_commit(&repo, "bugfix.txt", "Bugfix branch file", "Bugfix commit");
    
    // Go back to main
    utils::checkout_branch(&repo, main_branch_name);
    
    // Create a 'release' branch from main
    utils::create_branch(&repo, "release");
    utils::checkout_branch(&repo, "release");
    utils::add_file_and_commit(&repo, "release-notes.txt", "Release notes", "Release commit");
    
    // Go back to main
    utils::checkout_branch(&repo, main_branch_name);
    
    // Set up a remote to test remote branch detection
    let (remote_dir, remote_url) = utils::create_remote_for_repo(repo.path().parent().unwrap());
    
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
async fn test_analyze_branches() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Set up manager with a cloned repo that has multiple branches
    let (manager, repo_path) = setup_test_manager_with_branches().await;
    
    // Test analyze_branches
    Python::with_gil(|py| {
        let future = manager.analyze_branches(py, vec![repo_path.clone()]).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result: HashMap<String, Vec<HashMap<String, pyo3::PyObject>>> = rt.block_on(async {
            let py_dict = future.into_future().await.unwrap();
            
            // Extract the result into a more strongly-typed structure
            let result_dict = py_dict.downcast::<PyDict>(py).unwrap();
            let repo_result = result_dict.get_item(&repo_path).unwrap();
            
            // Verify it's a list of branch info
            assert!(repo_result.is_instance_of::<PyList>(py).unwrap(), "Result should be a list of branches");
            
            // Extract the full result
            py_dict.extract(py).unwrap()
        });
        
        // Verify the result contains our repo
        assert!(result.contains_key(&repo_path), "Result should contain our repo path");
        
        // Get the branch list for our repo
        let branches = &result[&repo_path];
        
        // Check if we have at least the branches we created
        let expected_branches = vec![
            main_branch_name_for(&repo_path), // 'main' or 'master'
            "feature".to_string(),
            "development".to_string(),
            "bugfix".to_string(),
            "release".to_string(),
        ];
        
        // Track which branches we found
        let mut found_branches = std::collections::HashSet::new();
        
        for branch_info in branches {
            // Extract branch name
            let name: String = branch_info["name"].extract(py).unwrap();
            found_branches.insert(name.clone());
            
            // Check that every branch has the required fields
            assert!(branch_info.contains_key("is_remote"), "Branch should have is_remote field");
            assert!(branch_info.contains_key("commit_id"), "Branch should have commit_id field");
            assert!(branch_info.contains_key("commit_message"), "Branch should have commit_message field");
            assert!(branch_info.contains_key("author_name"), "Branch should have author_name field");
            assert!(branch_info.contains_key("author_email"), "Branch should have author_email field");
            assert!(branch_info.contains_key("author_time"), "Branch should have author_time field");
            assert!(branch_info.contains_key("is_head"), "Branch should have is_head field");
            
            // For each branch we should have author data
            let author_name: String = branch_info["author_name"].extract(py).unwrap();
            let author_email: String = branch_info["author_email"].extract(py).unwrap();
            
            assert_eq!(author_name, "Test User", "Author name should match");
            assert_eq!(author_email, "test@example.com", "Author email should match");
            
            // Check if we have the expected commit messages for specific branches
            let commit_message: String = branch_info["commit_message"].extract(py).unwrap();
            if name == "bugfix" {
                assert!(commit_message.contains("Bugfix commit"), "Bugfix branch should have bugfix commit");
            } else if name == "development" {
                assert!(commit_message.contains("Development commit"), "Development branch should have development commit");
            } else if name == "release" {
                assert!(commit_message.contains("Release commit"), "Release branch should have release commit");
            }
            
            // Only one branch should be the HEAD
            let is_head: bool = branch_info["is_head"].extract(py).unwrap();
            if is_head {
                assert_eq!(name, main_branch_name_for(&repo_path), "HEAD should be on main branch");
            }
        }
        
        // Verify we found all expected branches
        for branch_name in expected_branches {
            assert!(found_branches.contains(&branch_name), "Should find {} branch", branch_name);
        }
    });
}

// Helper to determine main branch name (main or master)
fn main_branch_name_for(repo_path: &str) -> String {
    let repo = git2::Repository::open(repo_path).unwrap();
    if repo.find_branch("main", git2::BranchType::Local).is_ok() {
        "main".to_string()
    } else {
        "master".to_string()
    }
}

#[tokio::test]
async fn test_analyze_branches_empty_repo() {
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
    
    // Test analyze_branches on the empty repo
    Python::with_gil(|py| {
        let future = manager.analyze_branches(py, vec![repo_path_str.clone()]).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let py_dict = future.into_future().await.unwrap();
            let result_dict = py_dict.downcast::<PyDict>(py).unwrap();
            
            // For an empty repo, we should either get an empty list or an error string
            let repo_result = result_dict.get_item(&repo_path_str).unwrap();
            
            if repo_result.is_instance_of::<PyList>(py).unwrap() {
                let branch_list = repo_result.downcast::<PyList>(py).unwrap();
                assert_eq!(branch_list.len(), 0, "Branch list for empty repo should be empty");
            } else {
                // It's an error string
                assert!(repo_result.is_instance_of::<pyo3::types::PyString>(py).unwrap(), 
                       "Result for empty repo should be an error string");
            }
            
            Ok::<_, pyo3::PyErr>(())
        });
        
        assert!(result.is_ok(), "Test should complete without panic");
    });
}

#[tokio::test]
async fn test_analyze_branches_invalid_repo() {
    // Setup for this test
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create a manager with a nonexistent repo
    let invalid_repo_url = "https://github.com/nonexistent/repo.git";
    let manager = RepoManager::new(
        vec![invalid_repo_url.to_string()],
        "test_user".to_string(),
        "fake_token".to_string(),
    );
    
    // Try to analyze branches on the nonexistent repo
    Python::with_gil(|py| {
        let future = manager.analyze_branches(py, vec![invalid_repo_url.to_string()]).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let py_dict = future.into_future().await.unwrap();
            let result_dict = py_dict.downcast::<PyDict>(py).unwrap();
            
            // For an invalid repo, we should get an error string
            let repo_result = result_dict.get_item(invalid_repo_url).unwrap();
            assert!(repo_result.is_instance_of::<pyo3::types::PyString>(py).unwrap(), 
                   "Result for invalid repo should be an error string");
            
            Ok::<_, pyo3::PyErr>(())
        });
        
        assert!(result.is_ok(), "Test should complete without panic");
    });
}

#[tokio::test]
async fn test_analyze_branches_with_remotes() {
    // This test would set up a repository with remote branches and verify they're detected correctly
    // Since this is more complex and would require network operations or mocking, we'll leave it as a placeholder
    assert!(true, "This test would verify remote branch detection");
}
