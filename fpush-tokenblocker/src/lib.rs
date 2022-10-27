mod config;
pub use config::BlacklistSettings;

use dashmap::DashMap;
use log::{error, info};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
pub struct FpushBlocklistValue {
    blocking_start: u64,
    blocking_end: u64,
}

impl FpushBlocklistValue {
    pub fn new(current_timestamp: &Duration, block_time: &Duration) -> Self {
        Self {
            blocking_start: current_timestamp.as_secs(),
            blocking_end: current_timestamp.as_secs() + block_time.as_secs(),
        }
    }

    #[inline(always)]
    pub fn block_and_reset(&mut self, current_timestamp: &Duration, block_time: &Duration) {
        self.blocking_start = current_timestamp.as_secs();
        self.block(current_timestamp, block_time);
    }

    #[inline(always)]
    pub fn block(&mut self, current_timestamp: &Duration, block_time: &Duration) {
        self.blocking_end = current_timestamp.as_secs() + block_time.as_secs();
    }

    #[inline(always)]
    pub fn extend_block(&mut self, current_timestamp: &Duration, extend_block_time: &Duration) {
        if current_timestamp.as_secs() - self.blocking_start
            >= (self.blocking_end - self.blocking_start) / 2
        {
            self.block(current_timestamp, extend_block_time);
        }
    }

    #[inline(always)]
    pub fn is_blocked(&self, current_timestamp: &Duration) -> bool {
        self.blocking_start != 0 && current_timestamp.as_secs() <= self.blocking_end
    }
}

pub struct FpushBlocklist {
    token_blocklist: DashMap<String, FpushBlocklistValue>,
    block_time: Duration,
    extended_block_time: Duration,
}

impl FpushBlocklist {
    pub fn new(config: &BlacklistSettings) -> Self {
        Self {
            token_blocklist: DashMap::new(),
            block_time: config.normal_blacklisting(),
            extended_block_time: config.extended_blacklisting(),
        }
    }

    #[inline(always)]
    fn is_blocked_token(&self, token: &str) -> bool {
        if let Some(mut blocklist_entry) = self.token_blocklist.get_mut(token) {
            if let Ok(timestamp) = SystemTime::now().duration_since(UNIX_EPOCH) {
                if blocklist_entry.is_blocked(&timestamp) {
                    blocklist_entry.extend_block(&timestamp, &self.extended_block_time);
                    true
                } else {
                    false
                }
            } else {
                error!("Could not get current SystemTime");
                false
            }
        } else {
            false
        }
    }

    pub fn is_blocked(&self, token: &str) -> bool {
        self.is_blocked_token(token)
    }

    pub fn block(&self, token: String) {
        if let Ok(timestamp) = SystemTime::now().duration_since(UNIX_EPOCH) {
            if let Some(mut blocklist_entry) = self.token_blocklist.get_mut(&token) {
                if blocklist_entry.is_blocked(&timestamp) {
                    info!("Extending block time of token {}", token);
                    blocklist_entry.extend_block(&timestamp, &self.extended_block_time);
                } else {
                    info!("Reblocking token {}", token);
                    blocklist_entry.block_and_reset(&timestamp, &self.block_time);
                }
            } else {
                info!("Blocking token {}", token);
                self.token_blocklist.insert(
                    token,
                    FpushBlocklistValue::new(&timestamp, &self.block_time),
                );
            }
        } else {
            error!("Could not get current SystemTime");
        }
    }

    pub fn cleanup(&self) {
        if let Ok(timestamp) = SystemTime::now().duration_since(UNIX_EPOCH) {
            self.token_blocklist
                .retain(|_, v| !v.is_blocked(&timestamp));
        } else {
            error!("Could not get current SystemTime");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::{BlacklistSettings, FpushBlocklist};

    #[test]
    fn extended_blocking() {
        let settings = BlacklistSettings::new(Duration::from_secs(10), Duration::from_secs(20));
        let blocklist = FpushBlocklist::new(&settings);

        // check random token
        assert!(!blocklist.is_blocked_token("some-token"));

        // block token
        blocklist.block("some-token".to_string());
        assert!(blocklist.is_blocked_token("some-token"));

        sleep(Duration::from_secs(9));
        assert!(blocklist.is_blocked_token("some-token"));
        sleep(Duration::from_secs(22));
        assert!(!blocklist.is_blocked_token("some-token"));
        assert!(!blocklist.is_blocked_token("some-token"));
    }

    #[test]
    fn is_blocked() {
        let settings = BlacklistSettings::new(Duration::from_secs(10), Duration::from_secs(20));
        let blocklist = FpushBlocklist::new(&settings);

        assert!(!blocklist.is_blocked("some-token"));

        // block token
        blocklist.block("some-token".to_string());
        assert!(blocklist.is_blocked("some-token"));

        sleep(Duration::from_secs(9));
        assert!(blocklist.is_blocked("some-token"));
        sleep(Duration::from_secs(22));
        assert!(!blocklist.is_blocked("some-token"));
        assert!(!blocklist.is_blocked("some-token"));
    }
}
