use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleApnsConfig {
    cert_file_path: String,
    cert_password: String,
    topic: String,
    additional_data: Option<HashMap<String, Value>>,
    #[serde(default = "ApnsEndpoint::production")]
    environment: ApnsEndpoint,
    #[serde(default = "AppleApnsConfig::default_pool_timeout")]
    pool_idle_timeout: u64,
    #[serde(default = "AppleApnsConfig::default_request_timeout")]
    request_timeout: u64,
}

impl AppleApnsConfig {
    pub fn cert_file_path(&self) -> &str {
        &self.cert_file_path
    }

    pub fn cert_password(&self) -> &str {
        &self.cert_password
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn endpoint(&self) -> a2::Endpoint {
        match self.environment {
            ApnsEndpoint::Production => a2::Endpoint::Production,
            ApnsEndpoint::Sandbox => a2::Endpoint::Sandbox,
        }
    }

    pub fn additional_data(&self) -> &Option<HashMap<String, Value>> {
        &self.additional_data
    }

    pub fn pool_idle_timeout(&self) -> u64 {
        self.pool_idle_timeout
    }

    pub fn request_timeout(&self) -> u64 {
        self.request_timeout
    }

    pub fn default_pool_timeout() -> u64 {
        600
    }

    pub fn default_request_timeout() -> u64 {
        5
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApnsEndpoint {
    Production,
    Sandbox,
}

impl ApnsEndpoint {
    fn production() -> Self {
        Self::Production
    }
}
