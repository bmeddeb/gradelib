// tests/utils/mod.rs
// Test utilities for creating and managing test Git repositories

use git2::{Commit, Repository, Signature, Time};
use std::fs::{self, create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Use a lazy static Mutex to manage the test counter for unique IDs
static TEST_COUNTER: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));

/// Creates a temporary directory with a unique name for a test
pub fn create_temp_dir() -> TempDir {
    let mut counter = TEST_COUNTER.lock().unwrap();
    *counter += 1;
    let dir_name = format!("gradelib_test_{}", *counter);
    tempfile::Builder::new().prefix(&dir_name).tempdir().unwrap()
}

/// Creates a simple Git repository with a README.md file and an initial commit
pub fn create_simple_repo() -> (TempDir, Repository, String) {
    let temp_dir = create_temp_dir();
    let repo_path = temp_dir.path();
    
    // Initialize the repository
    let repo = Repository::init(repo_path).unwrap();
    
    // Create a README.md file
    let readme_path = repo_path.join("README.md");
    let mut file = File::create(&readme_path).unwrap();
    writeln!(file, "# Test Repository\n\nThis is a test repository for gradelib.").unwrap();
    
    // Add and commit the file
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    
    let signature = Signature::now("Test User", "test@example.com").unwrap();
    Commit::create(
        &repo,
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    ).unwrap();
    
    (temp_dir, repo, repo_path.to_string_lossy().to_string())
}

/// Creates a more complex Git repository with multiple files, branches, and commits
pub fn create_complex_repo() -> (TempDir, Repository, String) {
    let temp_dir = create_temp_dir();
    let repo_path = temp_dir.path();
    
    // Initialize the repository
    let repo = Repository::init(repo_path).unwrap();
    
    // Create a README.md file
    let readme_path = repo_path.join("README.md");
    let mut file = File::create(&readme_path).unwrap();
    writeln!(file, "# Complex Test Repository\n\nThis is a more complex test repository for gradelib.").unwrap();
    
    // Create a src directory with a file
    let src_dir = repo_path.join("src");
    create_dir_all(&src_dir).unwrap();
    let main_path = src_dir.join("main.rs");
    let mut file = File::create(&main_path).unwrap();
    writeln!(file, "fn main() {{\n    println!(\"Hello, world!\");\n}}").unwrap();
    
    // Add and commit the initial files
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.add_path(Path::new("src/main.rs")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    
    let signature = Signature::now("Test User", "test@example.com").unwrap();
    let initial_commit = Commit::create(
        &repo,
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    ).unwrap();
    
    // Create a new branch
    let obj = repo.find_object(initial_commit, None).unwrap();
    repo.branch("feature", &obj, false).unwrap();
    
    // Create another file on main
    let docs_dir = repo_path.join("docs");
    create_dir_all(&docs_dir).unwrap();
    let doc_path = docs_dir.join("documentation.md");
    let mut file = File::create(&doc_path).unwrap();
    writeln!(file, "# Documentation\n\nThis is documentation for the test repository.").unwrap();
    
    // Add and commit the new file
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("docs/documentation.md")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    
    let head = repo.head().unwrap();
    let parent_commit = head.peel_to_commit().unwrap();
    
    Commit::create(
        &repo,
        &signature,
        &signature,
        "Add documentation",
        &tree,
        &[&parent_commit],
    ).unwrap();
    
    // Switch to feature branch
    let feature_branch = repo.find_branch("feature", git2::BranchType::Local).unwrap();
    let feature_ref = feature_branch.get();
    let obj = repo.find_object(feature_ref.target().unwrap(), None).unwrap();
    repo.checkout_tree(&obj, None).unwrap();
    repo.set_head("refs/heads/feature").unwrap();
    
    // Create a file in the feature branch
    let feature_file_path = repo_path.join("feature.txt");
    let mut file = File::create(&feature_file_path).unwrap();
    writeln!(file, "This is a feature-specific file.").unwrap();
    
    // Update the main.rs file
    let main_path = src_dir.join("main.rs");
    let mut file = File::create(&main_path).unwrap();
    writeln!(file, "fn main() {{\n    println!(\"Hello from the feature branch!\");\n}}").unwrap();
    
    // Add and commit the changes
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("feature.txt")).unwrap();
    index.add_path(Path::new("src/main.rs")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    
    let head = repo.head().unwrap();
    let parent_commit = head.peel_to_commit().unwrap();
    
    Commit::create(
        &repo,
        &signature,
        &signature,
        "Feature implementation",
        &tree,
        &[&parent_commit],
    ).unwrap();
    
    // Switch back to main
    let main_branch = repo.find_branch("main", git2::BranchType::Local)
        .or_else(|_| repo.find_branch("master", git2::BranchType::Local)).unwrap();
    let main_ref = main_branch.get();
    let obj = repo.find_object(main_ref.target().unwrap(), None).unwrap();
    repo.checkout_tree(&obj, None).unwrap();
    
    let main_branch_name = if main_branch.name().unwrap().unwrap() == "main" {
        "refs/heads/main"
    } else {
        "refs/heads/master"
    };
    repo.set_head(main_branch_name).unwrap();
    
    (temp_dir, repo, repo_path.to_string_lossy().to_string())
}

/// Makes a simple repository remote-friendly by setting up a bare clone that can be used as a remote
pub fn create_remote_for_repo(repo_path: &Path) -> (TempDir, String) {
    // Create a bare clone to simulate a remote
    let remote_dir = create_temp_dir();
    let remote_path = remote_dir.path();
    
    // Initialize a bare repository
    let remote_repo = Repository::init_bare(remote_path).unwrap();
    
    // Set the original repo to use the bare repo as a remote
    let repo = Repository::open(repo_path).unwrap();
    repo.remote("origin", &remote_path.to_string_lossy()).unwrap();
    
    // Push to the remote
    let mut remote = repo.find_remote("origin").unwrap();
    let refspecs = ["refs/heads/*:refs/heads/*"];
    remote.push(&refspecs, None).unwrap();
    
    (remote_dir, format!("{}", remote_path.to_string_lossy()))
}

/// Helper function to add a file to a repository and commit it
pub fn add_file_and_commit(
    repo: &Repository, 
    file_path: &str, 
    content: &str, 
    commit_message: &str
) -> git2::Oid {
    let repo_path = repo.path().parent().unwrap();
    let full_path = repo_path.join(file_path);
    
    // Create parent directories if needed
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    
    // Create and write to the file
    let mut file = File::create(&full_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    
    // Add and commit the file
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(file_path)).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    
    let signature = Signature::now("Test User", "test@example.com").unwrap();
    let head = repo.head().unwrap();
    let parent_commit = head.peel_to_commit().unwrap();
    
    Commit::create(
        &repo,
        &signature,
        &signature,
        commit_message,
        &tree,
        &[&parent_commit],
    ).unwrap()
}

/// Helper to create a branch in a repository
pub fn create_branch(repo: &Repository, branch_name: &str) {
    let head = repo.head().unwrap();
    let head_commit = head.peel_to_commit().unwrap();
    repo.branch(branch_name, &head_commit, false).unwrap();
}

/// Helper to checkout a branch
pub fn checkout_branch(repo: &Repository, branch_name: &str) {
    let branch = repo.find_branch(branch_name, git2::BranchType::Local).unwrap();
    let branch_ref = branch.get();
    let obj = repo.find_object(branch_ref.target().unwrap(), None).unwrap();
    repo.checkout_tree(&obj, None).unwrap();
    repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();
}
