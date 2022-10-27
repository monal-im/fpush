use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleFcmConfig {
    pub fcm_secret_path: String,
}

impl GoogleFcmConfig {
    pub fn fcm_secret_path(&self) -> &str {
        &self.fcm_secret_path
    }
}
