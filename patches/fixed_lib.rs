// This is a fixed version of lib.rs with the authentication improvements
// Copy the relevant parts to your actual lib.rs file if the patch doesn't apply cleanly

// [... existing imports and code ...]

// --- Exposed Python Class: RepoManager ---
#[pyclass(name = "RepoManager", module = "gradelib")] // Add module for clarity
#[derive(Clone)]
pub struct RepoManager {
    // Holds the internal logic handler using Arc for shared ownership
    inner: Arc<InternalRepoManagerLogic>,
}

#[pymethods]
impl RepoManager {
    #[new]
    #[pyo3(signature = (urls, github_token, github_username=None))]
    fn new(urls: Vec<String>, github_token: String, github_username: Option<String>) -> Self {
        let string_urls: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
        
        // Use empty string if username is None
        let username = github_username.unwrap_or_default();
        
        // Create the internal logic handler instance, wrapped in Arc
        Self {
            inner: Arc::new(InternalRepoManagerLogic::new(
                &string_urls,
                &username,
                &github_token,
            )),
        }
    }
    
    // [... rest of the implementation ...]
}

// [... rest of the file ...]
