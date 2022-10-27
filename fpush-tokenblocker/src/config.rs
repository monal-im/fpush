use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistSettings {
    #[serde(deserialize_with = "serde_humantime")]
    normal_blacklisting: Duration,
    #[serde(deserialize_with = "serde_humantime")]
    extended_blacklisting: Duration,
}

impl Default for BlacklistSettings {
    fn default() -> Self {
        Self {
            normal_blacklisting: Duration::from_secs(600),
            extended_blacklisting: Duration::from_secs(1200),
        }
    }
}

impl BlacklistSettings {
    #[cfg(test)]
    pub fn new(normal_blacklisting: Duration, extended_blacklisting: Duration) -> Self {
        Self {
            normal_blacklisting,
            extended_blacklisting,
        }
    }

    pub fn normal_blacklisting(&self) -> Duration {
        self.normal_blacklisting
    }

    pub fn extended_blacklisting(&self) -> Duration {
        self.extended_blacklisting
    }
}

pub fn serde_humantime<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde_humantime::De::<Duration>::deserialize(deserializer)
        .map(|wrapped_de: serde_humantime::De<Duration>| wrapped_de.into_inner())
}
