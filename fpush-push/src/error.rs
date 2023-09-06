use derive_more::{Display, From};
pub(crate) type Result<T> = std::result::Result<T, Error>;
pub type PushRequestResult<T> = std::result::Result<T, PushRequestError>;

#[derive(Debug, From, Display)]
pub enum PushRequestError {
    TokenRatelimited,
    TokenBlocked,
    Internal,
    UnknownPushModule,
}

#[derive(Debug, From, Display)]
pub(crate) enum Error {
    PushErrors(fpush_traits::push::PushError),
}
