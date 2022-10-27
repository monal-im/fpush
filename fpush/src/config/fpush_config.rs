use std::time::Duration;

use crate::error::Result;
use fpush_push::FpushPushConfig;

use derive_getters::Getters;
use serde::Deserialize;

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FpushConfig {
    component: FpushComponentSettings,
    push_modules: FpushPushConfig,
    #[serde(default)]
    timeout: TimeoutConfig,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FpushComponentSettings {
    component_hostname: String,
    component_key: String,
    server_hostname: String,
    server_port: u16,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TimeoutConfig {
    #[serde(deserialize_with = "serde_humantime")]
    xmppconnection_error: std::time::Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            xmppconnection_error: Duration::from_secs(10),
        }
    }
}

pub fn serde_humantime<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde_humantime::De::<Duration>::deserialize(deserializer)
        .map(|wrapped_de: serde_humantime::De<Duration>| wrapped_de.into_inner())
}

// use serde to parse from file to struct
pub(crate) fn load_config(config_path: &str) -> Result<FpushConfig> {
    let settings_file = std::fs::File::open(config_path)?;
    let settings_reader = std::io::BufReader::new(settings_file);

    let config: FpushConfig = serde_json::from_reader(settings_reader)?;

    Ok(config)
}
