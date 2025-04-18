use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

/// Taiga API client configuration
#[derive(Debug, Clone)]
pub struct TaigaClientConfig {
    pub base_url: String,
    pub auth_token: String,
    pub username: String,
}

/// Main Taiga API client
#[derive(Debug, Clone)]
pub struct TaigaClient {
    config: TaigaClientConfig,
    client: reqwest::Client,
}

impl TaigaClient {
    /// Creates a new TaigaClient instance
    pub fn new(config: TaigaClientConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Creates the default headers for Taiga API requests
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.auth_token)).unwrap(),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("gradelib-taiga-provider"),
        );
        headers
    }

    /// Makes a GET request to the Taiga API
    pub async fn get(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let headers = self.create_headers();

        self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Taiga API request failed: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to read Taiga API response: {}", e))
    }

    /// Makes a POST request to the Taiga API
    pub async fn post(&self, endpoint: &str, body: &str) -> Result<String, String> {
        let url = format!("{}{}", self.config.base_url, endpoint);
        let headers = self.create_headers();

        self.client
            .post(&url)
            .headers(headers)
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("Taiga API request failed: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to read Taiga API response: {}", e))
    }
}
