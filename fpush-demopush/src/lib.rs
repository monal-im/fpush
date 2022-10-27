use std::time::Duration;

use fpush_traits::push::{PushError, PushResult, PushTrait};

use async_trait::async_trait;
use rand::Rng;
pub struct FpushDemoPush {}

impl FpushDemoPush {
    pub fn init() -> PushResult<Self> {
        let s = Self {};
        Ok(s)
    }
}

#[async_trait]
impl PushTrait for FpushDemoPush {
    async fn send(&self, _token: String) -> PushResult<()> {
        let wait_time;
        let return_code;
        {
            let mut rng = rand::thread_rng();
            wait_time = Duration::from_millis(rng.gen_range(10..500));
            return_code = rng.gen_range(0..300);
        }
        // wait random time
        tokio::time::sleep(wait_time).await;

        match return_code {
            0 => Err(PushError::PushEndpointPersistent),
            1 => Err(PushError::TokenBlocked),
            2 => Err(PushError::TokenRateLimited),
            3..=300 => Ok(()),
            _ => Err(PushError::PushEndpointPersistent),
        }
    }
}
