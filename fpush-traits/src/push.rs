use async_trait::async_trait;
use derive_more::{Display, From};

pub type PushResult<T> = std::result::Result<T, PushError>;

#[derive(Debug, From, Display)]
pub enum PushError {
    CertLoading,
    PushEndpointTmp,
    PushEndpointPersistent,
    TokenRateLimited,
    TokenBlocked,
    Unkown(u16),
}

#[async_trait]
pub trait PushTrait {
    /// returns false if the token should be blocked
    async fn send(&self, token: String) -> PushResult<()>;
}
