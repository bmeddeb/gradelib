# Gradelib Testing Guide

This document outlines the testing strategy for the Gradelib library and provides instructions for running the tests.

## Test Structure

The test suite is organized into the following components:

1. **Utility Modules**
   - `tests/utils/mod.rs`: Contains test utilities for creating and managing Git repositories
   - `tests/mocks/github_api.rs`: Contains mock implementations of GitHub API responses

2. **Integration Tests**
   - `tests/test_rust_integration.rs`: Basic integration tests for core functionality
   - `tests/test_repo_manager.rs`: Tests for repository management operations
   - `tests/test_blame.rs`: Tests for Git blame functionality
   - `tests/test_commits.rs`: Tests for commit analysis
   - `tests/test_branches.rs`: Tests for branch analysis
   - `tests/test_github_api.rs`: Tests for GitHub API integrations

## Test Coverage

The test suite covers the following areas:

### Core Functionality
- RepoManager creation and initialization
- Repository cloning operations
- Error handling for invalid repositories and URLs

### Git Operations
- Git blame analysis for files
- Commit history extraction and analysis
- Branch analysis and detection

### GitHub API Integration
- Fetching collaborator information
- Fetching issue information
- Fetching pull request information
- Fetching code review data
- Fetching comments (issues, PRs, code reviews, commits)
- Error handling for API failures (404, 401, rate limiting)

## Running the Tests

To run the complete test suite:

```bash
cargo test
```

To run a specific test file:

```bash
cargo test --test test_repo_manager
cargo test --test test_blame
cargo test --test test_commits
cargo test --test test_branches
cargo test --test test_github_api
```

To run a specific test:

```bash
cargo test test_clone_and_fetch_tasks
```

### Test Isolation

Some tests may interfere with each other if run concurrently. We've used the `serial_test` crate for tests that need isolation. When adding new tests that perform file I/O or network operations, consider using the `#[serial]` attribute to prevent race conditions.

## Adding New Tests

When adding new tests:

1. Create a new test file in the `tests/` directory if needed
2. Import the necessary test utilities from `tests/utils/mod.rs` and `tests/mocks/`
3. Follow the existing patterns for setup and teardown
4. Use the `tokio::test` attribute for async tests
5. Use Python's `with_gil` pattern for tests that interact with Python

### Example Test Structure

```rust
#[tokio::test]
async fn test_new_feature() {
    // Setup
    setup_async(Python::with_gil(|py| py)).unwrap();
    
    // Create test data
    let (temp_dir, repo, repo_path) = utils::create_simple_repo();
    
    // Create a RepoManager
    let manager = RepoManager::new(/* ... */);
    
    // Test the functionality
    Python::with_gil(|py| {
        let future = manager.some_method(py, /* ... */).unwrap();
        
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            future.into_future().await.unwrap().extract(py).unwrap()
        });
        
        // Assert expectations
        assert!(/* ... */);
    });
}
```

## Future Test Improvements

Areas for future improvement in the test suite:

1. Add coverage for Taiga API integrations
2. Implement more extensive error case handling
3. Add property-based testing for complex algorithms
4. Add stress testing for concurrent operations
5. Implement integration tests that use both Rust and Python interfaces
