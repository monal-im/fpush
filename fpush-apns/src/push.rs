use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use a2::{
    request::notification::CollapseId, request::payload::PayloadLike, Client, ClientConfig,
    DefaultNotificationBuilder, NotificationBuilder, NotificationOptions, Priority, PushType,
};
use fpush_traits::push::{PushError, PushResult, PushTrait};

use async_trait::async_trait;
use log::{debug, error};
use serde_json::Value;

use crate::config::{ApnsAuth, AppleApnsConfig};

pub struct FpushApns {
    apns: a2::client::Client,
    topic: String,
    additional_data: Option<HashMap<String, Value>>,
}

impl FpushApns {
    fn open_file(filename: &str) -> PushResult<std::fs::File> {
        std::fs::File::open(filename).map_err(|e| {
            error!("Could not open file {}: {}", filename, e);
            PushError::CertLoading
        })
    }

    pub fn init(apns_config: &AppleApnsConfig) -> PushResult<Self> {
        let mut client_config = ClientConfig::new(apns_config.endpoint());
        client_config.pool_idle_timeout_secs = Some(apns_config.pool_idle_timeout());
        client_config.request_timeout_secs = Some(apns_config.request_timeout());

        let apns_conn = match apns_config.auth() {
            Some(ApnsAuth::Certificate { path, password }) => {
                let mut cert_file = Self::open_file(path)?;
                Client::certificate(&mut cert_file, password, client_config).map_err(|e| {
                    error!("Problem initializing apple certificate config: {}", e);
                    PushError::PushEndpointTmp
                })?
            }
            Some(ApnsAuth::Token { key_path, key_id, team_id }) => {
                let key_file = Self::open_file(key_path)?;
                Client::token(key_file, key_id, team_id, client_config).map_err(|e| {
                    error!("Problem initializing apple token config: {}", e);
                    PushError::PushEndpointTmp
                })?
            }
            None => {
                error!("APNs config requires either (certFilePath + certPassword) or (keyPath + keyId + teamId)");
                return Err(PushError::CertLoading);
            }
        };

        Ok(Self {
            apns: apns_conn,
            topic: apns_config.topic().to_string(),
            additional_data: apns_config.additional_data().clone(),
        })
    }
}

#[async_trait]
impl PushTrait for FpushApns {
    #[inline(always)]
    async fn send(&self, token: String) -> PushResult<()> {
        // APNs payload structure matches Signal-Server APNSender's APN_NSE_NOTIFICATION_PAYLOAD:
        //   - mutable-content: 1 (wakes the NSE)
        //   - alert title/body present so iOS shows something if NSE fails to replace it
        //   - NO sound: the NSE-posted notification carries the user's chosen sound;
        //     when NSE fails (and the fallback alert is what shows), match Signal's
        //     silent-failure UX rather than inconsistent audio cues.
        let notification_builder = DefaultNotificationBuilder::new()
            .set_title("New Message")
            .set_body("New Message?")
            .set_mutable_content();
        // Signal-Server uses apns-collapse-id "incoming-message" on every push.
        // Effect: if NSE fails to post a user-visible notification (rare after
        // recent NSE minimization work) AND multiple pushes arrive while the
        // device is locked, the fallback alerts collapse to one entry on the
        // lock screen instead of stacking N copies of "New Message". The
        // per-conversation notifications NSE posts via UNUserNotificationCenter
        // use their own identifiers and are unaffected.
        let collapse_id = CollapseId::new("incoming-message")
            .expect("collapse id literal fits the 64-byte limit");
        let mut payload = notification_builder.build(
            &token,
            NotificationOptions {
                apns_priority: Some(Priority::High),
                apns_topic: Some(&self.topic),
                apns_expiration: Some(
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 4 * 7 * 24 * 3600,
                ),
                apns_push_type: Some(PushType::Alert),
                apns_collapse_id: Some(collapse_id),
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
