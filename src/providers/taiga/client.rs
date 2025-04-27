use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

/// Configuration for connecting to the Taiga API
#[derive(Debug, Clone)]
pub struct TaigaClientConfig {
    pub base_url: String,
    pub auth_token: String, // Use empty string for public access
    pub username: String,
}

/// Main Taiga API client for making HTTP requests
#[derive(Debug, Clone)]
pub struct TaigaClient {
    config: TaigaClientConfig,
    client: reqwest::Client,
}

impl TaigaClient {
    /// Create a new Taiga API client
    pub fn new(config: TaigaClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(25))
            .build()
            .expect("Failed to build reqwest client with timeout");
        Self { config, client }
    }

    /// Constructs the headers for a request, including Authorization if a token is present
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("x-disable-pagination", HeaderValue::from_static("True"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("gradelib-taiga-provider"),
        );

        if !self.config.auth_token.is_empty() {
            if let Ok(header_val) =
                HeaderValue::from_str(&format!("Bearer {}", self.config.auth_token))
            {
                headers.insert(AUTHORIZATION, header_val);
            }
        }

        headers
    }

    /// Perform a GET request to the Taiga API
    pub async fn get(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let headers = self.create_headers();

        self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    "Taiga not responsive, timed out. Please try again later.".to_string()
                } else {
                    format!("Taiga API request failed: {}", e)
                }
            })?
            .text()
            .await
            .map_err(|e| format!("Failed to read Taiga API response: {}", e))
    }

    /// Perform a POST request to the Taiga API
    pub async fn post(&self, endpoint: &str, body: &str) -> Result<String, String> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let headers = self.create_headers();

        self.client
            .post(&url)
            .headers(headers)
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    "Taiga not responsive, timed out. Please try again later.".to_string()
                } else {
                    format!("Taiga API request failed: {}", e)
                }
            })?
            .text()
            .await
            .map_err(|e| format!("Failed to read Taiga API response: {}", e))
    }

    /// Returns true if the client is authenticated (has a non-empty token)
    pub fn is_authenticated(&self) -> bool {
        !self.config.auth_token.trim().is_empty()
    }
}
