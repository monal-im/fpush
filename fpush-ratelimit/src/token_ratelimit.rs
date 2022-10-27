use dashmap::DashMap;
use log::debug;
use std::time::Duration;

pub struct FpushTokenRateLimit {
    ratelimit_map: DashMap<String, TokenRateLimitValue>,
    time_between_pushes: Duration,
    time_till_cleanup: Duration,
    hard_ratelimit_time: Duration,
    enabled: bool,
}

struct TokenRateLimitValue {
    last_push: std::time::Instant,
}

impl TokenRateLimitValue {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            last_push: std::time::Instant::now(),
        }
    }

    #[inline(always)]
    pub(crate) fn new_with_duration(ratelimit_duration: Duration) -> Self {
        Self {
            last_push: std::time::Instant::now() + ratelimit_duration,
        }
    }

    #[inline(always)]
    pub(crate) fn time_since_last_push(&self) -> Duration {
        if self.last_push > std::time::Instant::now() {
            Duration::ZERO
        } else {
            self.last_push.elapsed()
        }
    }

    #[inline(always)]
    pub(crate) fn reset_to_now(&mut self) {
        self.last_push = std::time::Instant::now();
    }

    #[inline(always)]
    pub(crate) fn toggle_timer(&mut self, timeout: Duration) {
        self.last_push += timeout;
    }

    #[inline(always)]
    pub(crate) fn hard_ratelimit(&mut self, timeout: Duration) {
        self.last_push = std::time::Instant::now() + timeout;
    }
}

impl FpushTokenRateLimit {
    pub fn new(config: &crate::RatelimitSettings) -> Self {
        Self {
            ratelimit_map: DashMap::new(),
            time_between_pushes: config.ratelimit_time(),
            time_till_cleanup: config.ratelimit_cleanup_interval(),
            hard_ratelimit_time: config.hard_ratelimit_time(),
            enabled: config.is_enabled(),
        }
    }

    #[inline(always)]
    pub async fn lookup_ratelimit(&self, token: String) -> bool {
        let (sendpush, wait_duration_opt) = self.internal_ratelimit_check(&token);
        // wait
        if let Some(wait_duration) = wait_duration_opt {
            debug!(
                "Ratelimit: sleeping {}s for token {}",
                wait_duration.as_secs(),
                token
            );
            tokio::time::sleep(wait_duration).await;
        }
        sendpush
    }

    // bool -> send push
    #[inline(always)]
    pub fn internal_ratelimit_check(&self, token: &str) -> (bool, Option<Duration>) {
        if !self.enabled {
            return (true, None);
        }
        if token.len() < 64 || token.len() > 512 {
            return (false, None);
        }
        if let Some(mut ratelimit_entry) = self.ratelimit_map.get_mut(token) {
            // check if timer exists
            if ratelimit_entry.time_since_last_push().is_zero() {
                // ignore push
                (false, None)
            } else {
                // no active timer
                let duration_since_last_push = ratelimit_entry.time_since_last_push();
                if duration_since_last_push >= self.time_between_pushes {
                    debug!(
                        "Ignoring existing rate limit for token {}, as it is to old: {}s",
                        token,
                        duration_since_last_push.as_secs()
                    );
                    // reset ratelimit
                    ratelimit_entry.reset_to_now();
                    (true, None)
                } else {
                    let timeout = self.time_between_pushes - duration_since_last_push;
                    ratelimit_entry.toggle_timer(timeout);
                    (true, Some(timeout))
                }
            }
        } else {
            // no entry exists -> create new entry
            // no rate limit
            self.ratelimit_map
                .insert(token.to_string(), TokenRateLimitValue::new());
            debug!("Inserting rate limit entry for token {}", token);
            (true, None)
        }
    }

    #[inline(always)]
    pub fn hard_ratelimit(&self, token: String) {
        debug!("Adding hard rate limit for token {}", token);
        if let Some(mut ratelimit_entry) = self.ratelimit_map.get_mut(&token) {
            ratelimit_entry.hard_ratelimit(self.hard_ratelimit_time)
        } else {
            // no entry exists -> create new entry
            self.ratelimit_map.insert(
                token.to_string(),
                TokenRateLimitValue::new_with_duration(self.hard_ratelimit_time),
            );
        };
    }

    pub fn cleanup(&self) {
        self.ratelimit_map
            .retain(|_, v| v.time_since_last_push() >= self.time_till_cleanup);
    }
}

#[cfg(test)]
mod tests {
    use crate::FpushTokenRateLimit;
    use crate::RatelimitSettings;

    use tokio::time::sleep;
    use tokio::time::Duration;
    use tokio::time::Instant;

    async fn check_lookup(
        tr: &FpushTokenRateLimit,
        key: String,
        send_push_expected: bool,
        min_execution_time: Duration,
        max_execution_time: Duration,
    ) {
        let now = Instant::now();
        let send_push = tr.lookup_ratelimit(key.clone()).await;
        let time_needed = now.elapsed();
        assert_eq!(
            send_push, send_push_expected,
            "Expected {} but received {}, {}, {:?}, {:?}",
            send_push_expected, send_push, key, min_execution_time, max_execution_time
        );
        assert!(
            time_needed + Duration::from_millis(100) > min_execution_time,
            "expected at least {}ms delay, but only {}ms were used",
            min_execution_time.as_millis(),
            time_needed.as_millis()
        );
        assert!(
            time_needed <= max_execution_time,
            "ratelimit was allowed {}ms, but used: {}ms",
            max_execution_time.as_millis(),
            time_needed.as_millis()
        );
    }

    #[test]
    fn create() {
        let _tr = FpushTokenRateLimit::new(&RatelimitSettings::default());
    }

    #[tokio::test]
    async fn token_length_check() {
        const T_BETWEEN_PUSHES: Duration = Duration::from_secs(10);
        let tr = FpushTokenRateLimit::new(&RatelimitSettings {
            hard_ratelimit_time: Duration::from_secs(10),
            ratelimit_time: T_BETWEEN_PUSHES,
            ratelimit_cleanup_interval: Duration::from_secs(180),
            ..Default::default()
        });
        check_lookup(
            &tr,
            "shortToken".to_owned(),
            false,
            Duration::from_secs(0),
            Duration::from_millis(100),
        )
        .await;
    }

    #[tokio::test]
    async fn insert() {
        let token = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz".to_string();
        const T_BETWEEN_PUSHES: Duration = Duration::from_secs(10);
        let tr = FpushTokenRateLimit::new(&RatelimitSettings {
            hard_ratelimit_time: Duration::from_secs(10),
            ratelimit_time: T_BETWEEN_PUSHES,
            ratelimit_cleanup_interval: Duration::from_secs(180),
            ..Default::default()
        });
        check_lookup(
            &tr,
            token.clone(),
            true,
            Duration::from_secs(0),
            Duration::from_millis(100),
        )
        .await;
        check_lookup(
            &tr,
            token.clone(),
            true,
            T_BETWEEN_PUSHES,
            Duration::from_secs(12),
        )
        .await;
    }

    #[test]
    fn cleanup() {
        const T_BETWEEN_PUSHES: Duration = Duration::from_secs(10);
        let tr = FpushTokenRateLimit::new(&RatelimitSettings {
            hard_ratelimit_time: Duration::from_secs(40),
            ratelimit_time: T_BETWEEN_PUSHES,
            ratelimit_cleanup_interval: Duration::from_secs(180),
            ..Default::default()
        });
        tr.cleanup();
    }

    #[tokio::test]
    async fn ratelimit_sequential() {
        let token = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz".to_string();
        const T_BETWEEN_PUSHES: Duration = Duration::from_secs(5);
        let tr = FpushTokenRateLimit::new(&RatelimitSettings {
            hard_ratelimit_time: Duration::from_secs(40),
            ratelimit_time: T_BETWEEN_PUSHES,
            ratelimit_cleanup_interval: Duration::from_secs(180),
            ..Default::default()
        });
        // sequential calls to ratelimit should allways return true == send a push
        check_lookup(
            &tr,
            token.clone(),
            true,
            Duration::from_secs(0),
            Duration::from_millis(100),
        )
        .await;
        for _i in 0..=4 {
            check_lookup(
                &tr,
                token.clone(),
                true,
                T_BETWEEN_PUSHES,
                T_BETWEEN_PUSHES + Duration::from_millis(100),
            )
            .await;
        }
    }

    #[tokio::test]
    async fn ratelimit_multi_thread() {
        const T_BETWEEN_PUSHES: Duration = Duration::from_secs(10);
        let tr = FpushTokenRateLimit::new(&RatelimitSettings {
            hard_ratelimit_time: Duration::from_secs(40),
            ratelimit_time: Duration::from_secs(20),
            ratelimit_cleanup_interval: Duration::from_secs(180),
            ..Default::default()
        });
        let tr_arc = std::sync::Arc::new(tr);
        // inital request should allow sending a push and be fast
        let tr_arc1 = tr_arc.clone();
        tokio::spawn(async move {
            check_lookup(
                &tr_arc1,
                "hello".to_owned(),
                true,
                Duration::from_secs(0),
                Duration::from_millis(100),
            )
            .await;
        });
        // wait for first thread to process
        sleep(Duration::from_millis(200)).await;
        // second request should still allow to send a push, but be queued and therefore slow
        let tr_arc2 = tr_arc.clone();
        tokio::spawn(async move {
            check_lookup(
                &tr_arc2,
                "hello".to_owned(),
                true,
                T_BETWEEN_PUSHES,
                T_BETWEEN_PUSHES + Duration::from_millis(100),
            )
            .await;
        });
        // wait for second thread to process
        sleep(Duration::from_millis(200)).await;
        // first and second slot full - no push should be allowed and all ratelimit requests fast
        let tr_arc3 = tr_arc.clone();
        tokio::spawn(async move {
            for _i in 0..=19 {
                check_lookup(
                    &tr_arc3,
                    "hello".to_owned(),
                    false,
                    Duration::from_secs(0),
                    Duration::from_millis(100),
                )
                .await;
                sleep(Duration::from_millis(500)).await;
            }
        });
    }
}
