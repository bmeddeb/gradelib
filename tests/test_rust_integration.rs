// tests/test_rust_integration.rs

// Integration tests for the gradelib Rust crate.

use gradelib::setup_async;
use pyo3::prelude::Python;

#[test]
fn test_setup_async() {
    Python::with_gil(|py| {
        assert!(setup_async(py).is_ok(), "setup_async() should succeed");
    });
}

// The rest of our tests are in separate files:
// - test_repo_manager.rs: Tests for RepoManager creation and clone operations
// - test_blame.rs: Tests for git blame functionality
// - test_commits.rs: Tests for commit analysis
// - test_branches.rs: Tests for branch analysis
