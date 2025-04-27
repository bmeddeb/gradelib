use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT},
    Client, Request, Response, StatusCode,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, Semaphore};

// Rate limit information from GitHub API response headers
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Maximum number of requests per hour
    pub limit: u32,
    /// Remaining requests for the current window
    pub remaining: u32,
    /// When the rate limit resets (Unix timestamp)
    pub reset_time: u64,
    /// Last time this info was updated
    pub last_updated: Instant,
    /// Resource type (core, search, etc.)
    pub resource: String,
}

impl Default for RateLimitInfo {
    fn default() -> Self {
        Self {
            limit: 5000, // Default authenticated limit
            remaining: 5000,
            reset_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 3600, // Default to 1 hour from now
            last_updated: Instant::now(),
            resource: "core".to_string(),
        }
    }
}

// ETag cache for conditional requests
#[derive(Debug)]
pub struct ETagCache {
    etags: HashMap<String, String>,
}

impl ETagCache {
    fn new() -> Self {
        Self {
            etags: HashMap::new(),
        }
    }

    fn get_etag(&self, url: &str) -> Option<&str> {
        self.etags.get(url).map(|s| s.as_str())
    }

    fn update_etag(&mut self, url: &str, etag: &str) {
        self.etags.insert(url.to_string(), etag.to_string());
    }
}

// A rate-limit aware GitHub client
#[derive(Clone, Debug)]
pub struct RateLimitedClient {
    /// Inner reqwest client
    client: Client,
    /// Current rate limit information
    rate_info: Arc<Mutex<RateLimitInfo>>,
    /// Semaphore for controlling concurrent requests
    semaphore: Arc<Semaphore>,
    /// Maximum concurrent requests
    max_concurrent: usize,
    /// Cache for ETags to support conditional requests
    etag_cache: Arc<Mutex<ETagCache>>,
}

impl RateLimitedClient {
    /// Creates a new rate-limited GitHub client
    pub fn new(token: &str, max_concurrent: usize) -> Result<Self, reqwest::Error> {
        let client = create_github_client(token)?;

        // Start with a conservative number of permits
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        Ok(Self {
            client,
            rate_info: Arc::new(Mutex::new(RateLimitInfo::default())),
            semaphore,
            max_concurrent,
            etag_cache: Arc::new(Mutex::new(ETagCache::new())),
        })
    }

    /// Helper method to build requests with the inner client
    pub fn build_request(
        &self,
        method: reqwest::Method,
        url: &str,
    ) -> Result<reqwest::Request, reqwest::Error> {
        self.client.request(method, url).build()
    }

    /// Makes a rate-limit aware request to the GitHub API
    pub async fn request(&self, mut request: Request) -> Result<Response, reqwest::Error> {
        // Add ETag header for conditional request if we have a cached ETag
        let url = request.url().to_string();

        {
            let etag_cache = self.etag_cache.lock().await;
            if let Some(etag) = etag_cache.get_etag(&url) {
                request.headers_mut().insert(
                    "If-None-Match",
                    HeaderValue::from_str(etag).unwrap_or_else(|_| HeaderValue::from_static("")),
                );
            }
        }

        // Acquire semaphore permit to limit concurrent requests
        let permit = self.semaphore.acquire().await.unwrap();

        // Check if we need to wait for rate limit reset
        self.wait_if_rate_limited().await;

        // Make the actual request
        let response = self.client.execute(request).await?;

        // Update ETag cache if response has an ETag header
        if let Some(etag) = response.headers().get("etag") {
            if let Ok(etag_str) = etag.to_str() {
                let mut etag_cache = self.etag_cache.lock().await;
                etag_cache.update_etag(&url, etag_str);
            }
        }

        // Update rate limit info from response headers
        self.update_rate_info_from_response(&response).await;

        // Adapt semaphore if needed
        self.adapt_concurrency().await;

        // Drop permit automatically when it goes out of scope
        drop(permit);

        Ok(response)
    }

    /// Makes a GET request to the GitHub API with rate limiting
    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        let request = self.client.get(url).build()?;
        // Use Box::pin to avoid infinite recursion in async functions
        Box::pin(self.request(request)).await
    }

    /// Executes a request with automatic retries for rate limit errors
    pub async fn execute_with_retry(
        &self,
        request: Request,
        max_retries: u32,
    ) -> Result<Response, reqwest::Error> {
        let mut attempts = 0;
        let mut last_err = None;
        let request_url = request.url().clone();

        while attempts < max_retries {
            match Box::pin(self.request(request.try_clone().unwrap())).await {
                Ok(response) => {
                    // If we get a 304 Not Modified, that's a success!
                    if response.status() == StatusCode::NOT_MODIFIED {
                        println!("Resource not modified: {}", request_url);
                        return Ok(response);
                    }

                    // Check if we hit a rate limit
                    if response.status() == StatusCode::FORBIDDEN
                        || response.status() == StatusCode::TOO_MANY_REQUESTS
                    {
                        if let Some(retry_after) = response.headers().get("retry-after") {
                            if let Ok(seconds) = retry_after.to_str().unwrap_or("60").parse::<u64>()
                            {
                                println!("Rate limited with retry-after: {} seconds", seconds);
                                tokio::time::sleep(Duration::from_secs(seconds)).await;
                                attempts += 1;
                                continue;
                            }
                        }

                        // No retry-after header, check if it's a rate limit response
                        let retry_needed = response.status() == StatusCode::FORBIDDEN
                            && response
                                .headers()
                                .get("x-ratelimit-remaining")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|s| s.parse::<u32>().ok())
                                .map(|r| r == 0)
                                .unwrap_or(false);

                        if retry_needed {
                            // Use exponential backoff
                            let backoff = Duration::from_secs(2u64.pow(attempts));
                            println!(
                                "Rate limited. Backing off for {} seconds",
                                backoff.as_secs()
                            );
                            tokio::time::sleep(backoff).await;
                            attempts += 1;
                            continue;
                        }

                        // If it's not a rate limit issue, return the response
                        return Ok(response);
                    }

                    // Other status code, return the response
                    return Ok(response);
                }
                Err(e) => {
                    last_err = Some(e);
                    attempts += 1;

                    // Use exponential backoff
                    let backoff = Duration::from_secs(2u64.pow(attempts));
                    println!(
                        "Request error. Backing off for {} seconds",
                        backoff.as_secs()
                    );
                    tokio::time::sleep(backoff).await;
                }
            }
        }

        Err(last_err.unwrap())
    }

    /// Gets the current rate limit information
    pub async fn get_rate_info(&self) -> RateLimitInfo {
        self.rate_info.lock().await.clone()
    }

    /// Fetch current rate limit status directly from GitHub API
    pub async fn fetch_rate_limit_status(&self) -> Result<(), reqwest::Error> {
        let response = self.get("https://api.github.com/rate_limit").await?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct ResourceRateLimit {
                limit: u32,
                remaining: u32,
                reset: u64,
            }

            #[derive(Deserialize)]
            struct Resources {
                core: ResourceRateLimit,
                search: ResourceRateLimit,
            }

            #[derive(Deserialize)]
            struct RateLimitResponse {
                resources: Resources,
            }

            let rate_limit: RateLimitResponse = response.json().await?;

            // Update our internal rate limit info with the fetched data
            let mut info = self.rate_info.lock().await;
            info.limit = rate_limit.resources.core.limit;
            info.remaining = rate_limit.resources.core.remaining;
            info.reset_time = rate_limit.resources.core.reset;
            info.last_updated = Instant::now();

            println!(
                "Updated rate limit info: {}/{} requests remaining, resets at {}",
                info.remaining, info.limit, info.reset_time
            );
        }

        Ok(())
    }

    /// Waits if we're approaching rate limits
    async fn wait_if_rate_limited(&self) {
        let wait_needed = {
            let info = self.rate_info.lock().await;

            if info.remaining <= 10 {
                // Get current time as unix timestamp
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if info.reset_time > now {
                    // Need to wait until reset
                    Some(Duration::from_secs(info.reset_time - now + 1))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(wait_duration) = wait_needed {
            // We're near the limit, wait until reset
            println!(
                "Rate limit almost reached. Waiting for {} seconds until reset.",
                wait_duration.as_secs()
            );
            tokio::time::sleep(wait_duration).await;

            // After waiting, refresh the rate limit info
            let _ = self.fetch_rate_limit_status().await;
        }
    }

    /// Updates rate limit information from response headers
    async fn update_rate_info_from_response(&self, response: &Response) {
        if let (Some(limit), Some(remaining), Some(reset)) = (
            parse_header_to_u32(response, "x-ratelimit-limit"),
            parse_header_to_u32(response, "x-ratelimit-remaining"),
            parse_header_to_u64(response, "x-ratelimit-reset"),
        ) {
            let resource = response
                .headers()
                .get("x-ratelimit-resource")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("core")
                .to_string();

            let mut info = self.rate_info.lock().await;
            info.limit = limit;
            info.remaining = remaining;
            info.reset_time = reset;
            info.last_updated = Instant::now();
            info.resource = resource;

            // Log if rate limit is getting low
            if remaining < 100 {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let time_until_reset = if reset > now { reset - now } else { 0 };

                println!(
                    "Rate limit getting low! {}/{} requests remaining, resets in {} seconds",
                    remaining, limit, time_until_reset
                );
            }
        }
    }

    /// Dynamically adjusts the semaphore based on remaining rate limit
    async fn adapt_concurrency(&self) {
        let info = self.rate_info.lock().await;

        // Calculate the ideal number of concurrent requests based on remaining rate
        let ideal_concurrent = match info.remaining {
            0..=10 => 1,                           // Critical: one at a time
            11..=100 => self.max_concurrent / 4,   // Very low: quarter capacity
            101..=1000 => self.max_concurrent / 2, // Low: half capacity
            _ => self.max_concurrent,              // Normal: full capacity
        };

        // Get current number of permits
        let current_permits = self.semaphore.available_permits();

        // Adjust permits if needed
        if ideal_concurrent > current_permits {
            // Add more permits
            self.semaphore
                .add_permits(ideal_concurrent - current_permits);
        }
        // If we need fewer permits, they'll be consumed naturally
    }
}

/// Helper to parse a header to u32
fn parse_header_to_u32(response: &Response, header_name: &str) -> Option<u32> {
    response
        .headers()
        .get(header_name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok())
}

/// Helper to parse a header to u64
fn parse_header_to_u64(response: &Response, header_name: &str) -> Option<u64> {
    response
        .headers()
        .get(header_name)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

/// Creates a GitHub API client with proper authentication and standard headers
pub fn create_github_client(token: &str) -> Result<Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token)).unwrap(),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("gradelib-github-client/0.1.0"),
    );
    Client::builder().default_headers(headers).build()
}
