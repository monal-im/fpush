use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleApnsConfig {
    cert_file_path: String,
    cert_password: String,
    topic: String,
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
}
