// This is a fixed version of the credentials callback in repo.rs
// Copy the relevant parts to your actual repo.rs file if the patch doesn't apply cleanly

// [... existing imports and code ...]

// In the clone() method of InternalRepoManagerLogic:
pub async fn clone(&self, url: String) -> (Result<PathBuf, String>, String) {
    self.update_status(&url, InternalCloneStatus::Cloning(0))
        .await;
    let manager_logic = Clone::clone(self);
    let username = self.github_username.clone();
    let token = self.github_token.clone();
    let url_clone = url.clone();
    let result: Result<Result<PathBuf, String>, tokio::task::JoinError> =
        tokio::task::spawn_blocking(move || {
            let temp_dir = TempDir::new().map_err(|e| e.to_string())?;
            let temp_path = temp_dir.path().to_path_buf();
            let mut callbacks = RemoteCallbacks::new();
            let username_cb = username.clone();
            let token_cb = token.clone();
            
            // Improved credentials callback to handle empty usernames and provide better error handling
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                // Log auth attempt for debugging
                eprintln!("Git authentication attempt for URL: {}", url);
                if let Some(user) = username_from_url {
                    eprintln!("Username from URL: {}", user);
                }
                
                // Determine which username to use
                let effective_username = if username_cb.is_empty() {
                    // Use "git" as fallback username for GitHub URLs
                    if url.contains("github.com") {
                        "git"
                    } else {
                        // For non-GitHub URLs, try with the URL-provided username if available
                        username_from_url.unwrap_or("")
                    }
                } else {
                    // Use the provided username
                    &username_cb
                };
                
                // Log what we're using for authentication (careful not to log the token)
                eprintln!("Using username '{}' for authentication", effective_username);
                
                // Create the credential
                Cred::userpass_plaintext(effective_username, &token_cb)
            });
            
            // [... rest of the code ...]
        });
        
    // [... rest of the method ...]
}

// [... rest of the file ...]
