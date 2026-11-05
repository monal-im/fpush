use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleApnsConfig {
    topic: String,
    additional_data: Option<HashMap<String, Value>>,
    #[serde(default = "ApnsEndpoint::production")]
    environment: ApnsEndpoint,
    #[serde(default = "AppleApnsConfig::default_pool_timeout")]
    pool_idle_timeout: u64,
    #[serde(default = "AppleApnsConfig::default_request_timeout")]
    request_timeout: u64,

    // Certificate-based auth (p12)
    cert_file_path: Option<String>,
    cert_password: Option<String>,

    // Token-based auth (p8)
    key_path: Option<String>,
    key_id: Option<String>,
    team_id: Option<String>,
}

pub enum ApnsAuth<'a> {
    Certificate { path: &'a str, password: &'a str },
    Token { key_path: &'a str, key_id: &'a str, team_id: &'a str },
}

impl AppleApnsConfig {
    pub fn auth(&self) -> Option<ApnsAuth<'_>> {
        if let (Some(path), Some(password)) = (
            self.cert_file_path.as_deref(),
            self.cert_password.as_deref(),
        ) {
            return Some(ApnsAuth::Certificate { path, password });
        }
        if let (Some(key_path), Some(key_id), Some(team_id)) = (
            self.key_path.as_deref(),
            self.key_id.as_deref(),
            self.team_id.as_deref(),
        ) {
            return Some(ApnsAuth::Token { key_path, key_id, team_id });
        }
        None
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
