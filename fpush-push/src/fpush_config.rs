use std::collections::HashMap;

use fpush_ratelimit::RatelimitSettings;
use fpush_tokenblocker::BlacklistSettings;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct FpushPushConfig(std::collections::HashMap<String, PushConfig>);

impl FpushPushConfig {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn config(&self) -> &std::collections::HashMap<String, PushConfig> {
        &self.0
    }

    pub fn insert(&mut self, identifier: String, push_config: PushConfig) {
        self.0.insert(identifier, push_config);
    }
}

impl Default for FpushPushConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushConfig {
    #[cfg(feature = "enable_apns_support")]
    Apple {
        apns: fpush_apns::AppleApnsConfig,
        #[serde(default)]
        blacklist: BlacklistSettings,
        #[serde(default)]
        ratelimit: RatelimitSettings,
        #[serde(default)]
        is_default_module: bool,
    },
    #[cfg(feature = "enable_fcm_support")]
    Google {
        fcm: fpush_fcm::GoogleFcmConfig,
        #[serde(default)]
        blacklist: BlacklistSettings,
        #[serde(default)]
        ratelimit: RatelimitSettings,
        #[serde(default)]
        is_default_module: bool,
    },
    #[cfg(feature = "enable_demo_support")]
    Demo {
        #[serde(default)]
        blacklist: BlacklistSettings,
        #[serde(default)]
        ratelimit: RatelimitSettings,
        #[serde(default)]
        is_default_module: bool,
    },
}
