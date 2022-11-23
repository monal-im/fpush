use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistSettings {
    invalid_token: BlacklistBlockingTimes,
    push_error: BlacklistBlockingTimes,
    #[serde(deserialize_with = "serde_humantime")]
    block_extension: Duration,
}

impl Default for BlacklistSettings {
    fn default() -> Self {
        Self {
            invalid_token: BlacklistBlockingTimes::new(
                Duration::from_secs(60 * 60 * 24),
                Duration::from_secs(60 * 60 * 24 * 5),
            ),
            push_error: BlacklistBlockingTimes::new(
                Duration::from_secs(60 * 10),
                Duration::from_secs(60 * 20),
            ),
            block_extension: Duration::from_secs(600),
        }
    }
}

impl BlacklistSettings {
    #[cfg(test)]
    pub fn new_debug_config(
        invalid_token: BlacklistBlockingTimes,
        push_error: BlacklistBlockingTimes,
        block_extension: Duration,
    ) -> Self {
        Self {
            invalid_token,
            push_error,
            block_extension,
        }
    }

    pub fn invalid_token(&self) -> &BlacklistBlockingTimes {
        &self.invalid_token
    }

    pub fn push_error(&self) -> &BlacklistBlockingTimes {
        &self.push_error
    }

    pub fn block_extension(&self) -> &Duration {
        &self.block_extension
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistBlockingTimes {
    #[serde(deserialize_with = "serde_humantime")]
    inital_blocking: Duration,
    #[serde(deserialize_with = "serde_humantime")]
    extended_blocking: Duration,
}

impl Default for BlacklistBlockingTimes {
    fn default() -> Self {
        Self {
            inital_blocking: Duration::from_secs(600),
            extended_blocking: Duration::from_secs(1200),
        }
    }
}

impl BlacklistBlockingTimes {
    pub fn new(inital_blocking: Duration, extended_blocking: Duration) -> Self {
        Self {
            inital_blocking,
            extended_blocking,
        }
    }

    pub fn inital_blocking(&self) -> Duration {
        self.inital_blocking
    }

    pub fn extended_blocking(&self) -> Duration {
        self.extended_blocking
    }
}

pub fn serde_humantime<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    serde_humantime::De::<Duration>::deserialize(deserializer)
        .map(|wrapped_de: serde_humantime::De<Duration>| wrapped_de.into_inner())
}
