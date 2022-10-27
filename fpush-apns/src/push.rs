use std::time::SystemTime;

use a2::{Client, NotificationOptions, Priority, PushType};
use a2::{LocalizedNotificationBuilder, NotificationBuilder};
use fpush_traits::push::{PushError, PushResult, PushTrait};

use async_trait::async_trait;
use log::{debug, error};

use crate::AppleApnsConfig;
pub struct FpushApns {
    apns: a2::client::Client,
    topic: String,
}

impl FpushApns {
    fn open_cert(filename: &str) -> PushResult<std::fs::File> {
        if let Ok(file) = std::fs::File::open(filename) {
            Ok(file)
        } else {
            Err(PushError::CertLoading)
        }
    }

    pub fn init(apns_config: &AppleApnsConfig) -> PushResult<Self> {
        let mut certificate = FpushApns::open_cert(apns_config.cert_file_path())?;
        match Client::certificate(
            &mut certificate,
            apns_config.cert_password(),
            a2::Endpoint::Production,
        ) {
            Ok(apns_conn) => {
                let wrapped_conn = Self {
                    apns: apns_conn,
                    topic: apns_config.topic().to_string(),
                };
                Ok(wrapped_conn)
            }
            Err(a2::error::Error::ReadError(_)) => Err(PushError::PushEndpointPersistent),
            Err(_) => Err(PushError::PushEndpointTmp),
        }
    }
}

#[async_trait]
impl PushTrait for FpushApns {
    #[inline(always)]
    async fn send(&self, token: String) -> PushResult<()> {
        let mut notification_builder =
            LocalizedNotificationBuilder::new("New Message", "New Message?");
        notification_builder.set_mutable_content();
        notification_builder.set_sound("default");
        let payload = notification_builder.build(
            &token,
            NotificationOptions {
                apns_priority: Some(Priority::High),
                apns_topic: Some(&self.topic),
                apns_expiration: Some(
                    SystemTime::now().elapsed().unwrap().as_secs() + 4 * 7 * 24 * 3600,
                ),
                apns_push_type: PushType::Alert,
                ..Default::default()
            },
        );
        log::debug!(
            "Payload send to apple: {}",
            payload.clone().to_json_string().unwrap()
        );
        match self.apns.send(payload).await {
            Ok(response) => {
                debug!(
                    "Got response {} from apple for token {}",
                    response.code, token
                );
                match response.code {
                    200 => Ok(()),
                    400 => Err(PushError::PushEndpointPersistent),
                    403 => Err(PushError::PushEndpointPersistent),
                    // TODO: check reason for 403 return code
                    405 => Err(PushError::PushEndpointPersistent),
                    410 => Err(PushError::TokenBlocked),
                    429 => Err(PushError::TokenRateLimited),
                    500 => Err(PushError::PushEndpointTmp),
                    503 => Err(PushError::PushEndpointTmp),
                    ecode => {
                        error!("Received unhandled error code from apple apns: {}", ecode);
                        Err(PushError::Unkown(ecode))
                    }
                }
            }
            Err(e) => {
                error!("Could not send apns message to apple: {}", e);
                Err(PushError::PushEndpointTmp)
            }
        }
    }
}
