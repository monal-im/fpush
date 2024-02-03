use derive_more::{Display, From};
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    Io(std::io::Error),
    #[allow(clippy::enum_variant_names)]
    ConfigError(serde_json::Error),
    Config(String),
    Xmpp(Box<tokio_xmpp::Error>),
    PubSubNonPublish,
    PubSubInvalidFormat,
    PubSubToManyPublishOptions,
    PubSubInvalidPushModuleConfiguration,
}

impl std::convert::From<tokio_xmpp::Error> for Error {
    fn from(e: tokio_xmpp::Error) -> Self {
        Error::Xmpp(Box::new(e))
    }
}
