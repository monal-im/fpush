use std::collections::HashMap;
use std::time::SystemTime;

use a2::{
    Client, DefaultNotificationBuilder, NotificationBuilder, NotificationOptions, Priority,
    PushType,
};
use fpush_traits::push::{PushError, PushResult, PushTrait};

use async_trait::async_trait;
use log::{debug, error};
use serde_json::Value;

use crate::AppleApnsConfig;
pub struct FpushApns {
    apns: a2::client::Client,
    topic: String,
    additional_data: Option<HashMap<String, Value>>,
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
            apns_config.endpoint(),
        ) {
            Ok(apns_conn) => {
                let wrapped_conn = Self {
                    apns: apns_conn,
                    topic: apns_config.topic().to_string(),
                    additional_data: apns_config.additional_data().clone(),
                };
                Ok(wrapped_conn)
            }
            Err(a2::error::Error::ReadError(e)) => {
                error!("Could not read apns: {}", e);
                Err(PushError::PushEndpointPersistent)
            }
            Err(e) => {
                error!("Problem initializing apple config: {}", e);
                Err(PushError::PushEndpointTmp)
            }
        }
    }
}

#[async_trait]
impl PushTrait for FpushApns {
    #[inline(always)]
    async fn send(&self, token: String) -> PushResult<()> {
        let notification_builder = DefaultNotificationBuilder::new()
            .set_title("New Message")
            .set_body("New Message?")
            .set_mutable_content()
            .set_sound("default");
        let mut payload = notification_builder.build(
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
        match &self.additional_data {
            None => {}
            Some(additional_data) => {
                for (key, value) in additional_data {
                    payload.add_custom_data(key, value).unwrap();
                }
            }
        }
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
                response_code_to_push_error(response.code)
            }
            Err(e) => {
                error!("Could not send apns message to apple: {}", e);
                if let a2::Error::ResponseError(response) = e {
                    return response_code_to_push_error(response.code);
                }
                Err(PushError::PushEndpointTmp)
            }
        }
    }
}

fn response_code_to_push_error(response_code: u16) -> PushResult<()> {
    match response_code {
        200 => Ok(()),
        400 => Err(PushError::PushEndpointPersistent),
        403 => Err(PushError::PushEndpointPersistent),
        405 => Err(PushError::PushEndpointPersistent),
        410 => Err(PushError::TokenBlocked),
        429 => Err(PushError::TokenRateLimited),
        500 => Err(PushError::PushEndpointTmp),
        503 => Err(PushError::PushEndpointTmp),
        ecode => {
            error!("Received unhandled error code from apple apns: {}", ecode);
            Err(PushError::Unknown(ecode))
        }
    }
}
