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

// TODO: Add #[tokio::test] async fn tests for RepoManager, clone_all, bulk_blame, etc.
