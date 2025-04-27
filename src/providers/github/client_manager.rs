use crate::providers::github::client::RateLimitedClient;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Global rate-limited GitHub client manager
#[derive(Debug)]
pub struct GitHubClientManager {
    client: RateLimitedClient,
}

impl GitHubClientManager {
    /// Creates a new GitHub client manager (async)
    async fn new(token: &str, max_concurrent: usize) -> Result<Self, reqwest::Error> {
        let client = RateLimitedClient::new(token, max_concurrent).await?;
        Ok(Self { client })
    }

    /// Gets a reference to the rate-limited client
    pub fn get_client(&self) -> RateLimitedClient {
        self.client.clone()
    }
}

// Global async singleton instance
static INSTANCE: OnceCell<Arc<GitHubClientManager>> = OnceCell::const_new();

/// Initializes the global GitHub client manager (async)
///
/// This should be called early in the application lifecycle, typically when
/// the RepoManager is first created. It sets up a shared client that all
/// modules can use.
pub async fn init(token: &str, max_concurrent: usize) -> Result<(), reqwest::Error> {
    if INSTANCE.get().is_none() {
        let manager = GitHubClientManager::new(token, max_concurrent).await?;
        let _ = INSTANCE.set(Arc::new(manager));
    }
    Ok(())
}

/// Gets the global GitHub client instance
///
/// Returns None if the client hasn't been initialized yet with `init()`.
pub fn get_client() -> Option<RateLimitedClient> {
    INSTANCE.get().map(|manager| manager.get_client())
}

/// Gets or initializes the global GitHub client (async)
///
/// If the client hasn't been initialized yet, this will create a new instance.
/// This is useful for modules that don't want to worry about initialization
/// but need to ensure they have a valid client.
pub async fn get_or_init_client(
    token: &str,
    max_concurrent: usize,
) -> Result<RateLimitedClient, reqwest::Error> {
    if INSTANCE.get().is_none() {
        init(token, max_concurrent).await?;
    }
    Ok(INSTANCE.get().unwrap().get_client())
}
