use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleApnsConfig {
    cert_file_path: String,
    cert_password: String,
    topic: String,
    #[serde(default = "ApnsEndpoint::production")]
    environment: ApnsEndpoint,
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
