// Utility functions for the library
pub fn extract_repo_name(url: &str) -> Result<String, String> {
    if url.starts_with("https://github.com/") {
        let parts: Vec<&str> = url
            .strip_prefix("https://github.com/")
            .unwrap()
            .split('/')
            .collect();
        if parts.len() >= 2 {
            Ok(format!(
                "{}/{}",
                parts[0],
                parts[1].trim_end_matches(".git")
            ))
        } else {
            Err("Invalid GitHub URL format".to_string())
        }
    } else if url.starts_with("git@github.com:") {
        let parts: Vec<&str> = url
            .strip_prefix("git@github.com:")
            .unwrap()
            .split('/')
            .collect();
        if parts.len() >= 2 {
            Ok(format!(
                "{}/{}",
                parts[0],
                parts[1].trim_end_matches(".git")
            ))
        } else {
            Err("Invalid GitHub URL format".to_string())
        }
    } else {
        Err(format!("Unsupported URL format: {}", url))
    }
}
