use std::{collections::HashMap, path::Path};

use fpush_traits::push::{PushError, PushResult, PushTrait};

use async_trait::async_trait;
use google_fcm1::{
    api::{Message, SendMessageRequest},
    hyper_rustls, hyper_util, yup_oauth2, FirebaseCloudMessaging,
};
use log::{error, warn};

use serde::Deserialize;

use crate::config::GoogleFcmConfig;
pub struct FpushFcm {
    fcm_conn: FirebaseCloudMessaging<
        hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    >,
    fcm_parent: String,
}

impl FpushFcm {
    async fn load_oauth2_app_secret(fcm_config: &GoogleFcmConfig) -> yup_oauth2::ServiceAccountKey {
        match yup_oauth2::read_service_account_key(Path::new(fcm_config.fcm_secret_path())).await {
            Ok(s) => s,
            Err(e) => panic!(
                "Could not read fcm config file at {} reason: {}",
                fcm_config.fcm_secret_path(),
                e
            ),
        }
    }

    pub async fn init(fcm_config: &GoogleFcmConfig) -> PushResult<Self> {
        let fcm_secret = Self::load_oauth2_app_secret(fcm_config).await;

        // create login auth object
        let auth = match yup_oauth2::ServiceAccountAuthenticator::builder(fcm_secret.clone())
            .build()
            .await
        {
            Ok(auth) => auth,
            Err(e) => {
                error!("Could not load fcm DeviceFlowAuthenticator: {}", e);
                return Err(PushError::CertLoading);
            }
        };

        let hyper_client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .unwrap()
                        .https_only()
                        .enable_http2()
                        .build(),
                );
        let fcm_conn = FirebaseCloudMessaging::new(hyper_client, auth);
        Ok(Self {
            fcm_conn,
            fcm_parent: format!("projects/{}", fcm_secret.project_id.unwrap()),
        })
    }
}

#[derive(Debug, Deserialize)]
struct FcmErrorResponse {
    error: FcmErrorDetails,
}

#[derive(Debug, Deserialize)]
struct FcmErrorDetails {
    code: u16,
    details: Vec<FcmErrorDetail>,
    #[allow(dead_code)]
    message: String,
    #[allow(dead_code)]
    status: String,
}

#[derive(Debug, Deserialize)]
struct FcmErrorDetail {
    #[allow(dead_code)]
    #[serde(rename = "@type")]
    type_url: String,
    #[serde(rename = "errorCode")]
    error_code: FcmErrorCode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FcmErrorCode {
    UnspecifiedError,
    InvalidArgument,
    Unregistered,
    SenderIdMismatch,
    QuotaExceeded,
    Unavailable,
    Internal,
    ThirdPartyAuthError,
}

#[async_trait]
impl PushTrait for FpushFcm {
    #[inline(always)]
    async fn send(&self, token: String) -> PushResult<()> {
        let req = SendMessageRequest {
            message: Some(create_push_message(token)),
            validate_only: None,
        };

        let fcm_result = self
            .fcm_conn
            .projects()
            .messages_send(req, &self.fcm_parent)
            .doit()
            .await;
        match fcm_result {
            Err(e) => {
                warn!("FCM returned {}", e);
                if let google_fcm1::Error::BadRequest(error_body) = e {
                    let parsed_error_body: FcmErrorResponse =
                        match serde_json::from_value(error_body) {
                            Ok(body) => body,
                            Err(e) => {
                                error!("Could not parse fcm response: {}", e);
                                return Err(PushError::PushEndpointTmp);
                            }
                        };
                    // Get the first error code if available
                    if let Some(detail) = parsed_error_body.error.details.first() {
                        match detail.error_code {
                            FcmErrorCode::Unregistered => Err(PushError::TokenBlocked),
                            FcmErrorCode::QuotaExceeded => Err(PushError::TokenRateLimited),
                            FcmErrorCode::Unavailable => Err(PushError::PushEndpointTmp),
                            FcmErrorCode::Internal => Err(PushError::PushEndpointTmp),
                            FcmErrorCode::SenderIdMismatch => Err(PushError::TokenBlocked),
                            _ => Err(PushError::Unknown(u16::MAX)),
                        }
                    } else {
                        Err(PushError::Unknown(parsed_error_body.error.code))
                    }
                } else {
                    Err(PushError::PushEndpointTmp)
                }
            }
            Ok(_) => Ok(()),
        }
    }
}

#[inline(always)]
fn create_push_message(token: String) -> Message {
    Message {
        data: Some(HashMap::new()),
        token: Some(token),
        ..Default::default()
    }
}
