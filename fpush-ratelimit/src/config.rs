use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatelimitSettings {
    #[serde(deserialize_with = "serde_humantime")]
    pub hard_ratelimit_time: Duration,
    #[serde(deserialize_with = "serde_humantime")]
    pub ratelimit_time: Duration,
    #[serde(deserialize_with = "serde_humantime")]
    pub ratelimit_cleanup_interval: Duration,
    pub enabled: bool,
}

impl Default for RatelimitSettings {
    fn default() -> Self {
        Self {
            hard_ratelimit_time: Duration::from_secs(600),
            ratelimit_time: Duration::from_secs(20),
            ratelimit_cleanup_interval: Duration::from_secs(300),
            enabled: true,
        }
    }
}

impl RatelimitSettings {
    pub fn hard_ratelimit_time(&self) -> Duration {
        self.hard_ratelimit_time
    }

    pub fn ratelimit_time(&self) -> Duration {
        self.ratelimit_time
    }

    pub fn ratelimit_cleanup_interval(&self) -> Duration {
        self.ratelimit_cleanup_interval
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

pub fn serde_humantime<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde_humantime::De::<Duration>::deserialize(deserializer)
        .map(|wrapped_de: serde_humantime::De<Duration>| wrapped_de.into_inner())
}
