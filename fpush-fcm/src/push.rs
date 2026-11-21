use std::{collections::HashMap, path::Path};

use fpush_traits::push::{PushError, PushResult, PushTrait};

use async_trait::async_trait;
use google_fcm1::{
    api::{AndroidConfig, Message, SendMessageRequest},
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
    #[serde(default)]
    details: Vec<FcmErrorDetail>,
    #[allow(dead_code)]
    #[serde(default)]
    message: String,
    #[allow(dead_code)]
    #[serde(default)]
    status: String,
}

// Only the google.firebase.fcm.v1.FcmError entry has an errorCode, other
// detail types (e.g. google.rpc.BadRequest) can come along with it.
#[derive(Debug, Deserialize)]
struct FcmErrorDetail {
    #[allow(dead_code)]
    #[serde(rename = "@type", default)]
    type_url: String,
    #[serde(rename = "errorCode", default)]
    error_code: Option<FcmErrorCode>,
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
    #[serde(other)]
    Other,
}

// Only errors caused by the token itself may block it. Outages, quota and our
// own credentials are temporary and must not get the token blocked.
fn map_fcm_error(error: &FcmErrorDetails) -> PushError {
    match error.details.iter().find_map(|d| d.error_code.as_ref()) {
        Some(FcmErrorCode::Unregistered) | Some(FcmErrorCode::SenderIdMismatch) => {
            PushError::TokenBlocked
        }
        Some(FcmErrorCode::QuotaExceeded) => PushError::TokenRateLimited,
        Some(FcmErrorCode::Unavailable)
        | Some(FcmErrorCode::Internal)
        | Some(FcmErrorCode::ThirdPartyAuthError) => PushError::PushEndpointTmp,
        _ => match error.code {
            404 => PushError::TokenBlocked,
            429 => PushError::TokenRateLimited,
            401 | 403 | 500..=599 => PushError::PushEndpointTmp,
            code => PushError::Unknown(code),
        },
    }
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
            Err(google_fcm1::Error::BadRequest(error_body)) => {
                warn!("FCM returned {}", error_body);
                match serde_json::from_value::<FcmErrorResponse>(error_body) {
                    Ok(body) => Err(map_fcm_error(&body.error)),
                    Err(e) => {
                        error!("Could not parse fcm response: {}", e);
                        Err(PushError::PushEndpointTmp)
                    }
                }
            }
            Err(e) => {
                warn!("FCM returned {}", e);
                Err(PushError::PushEndpointTmp)
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
        // Data-only messages default to normal priority, which Android defers while the
        // device is in Doze. A wake-up is only useful if it arrives now.
        android: Some(AndroidConfig {
            priority: Some("HIGH".to_string()),
            // Pushes carry no payload, so a device coming back online only needs one.
            collapse_key: Some("fpush".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_message_is_high_priority() {
        let msg = serde_json::to_value(create_push_message("token".to_string())).unwrap();
        assert_eq!(msg["token"], "token");
        assert_eq!(msg["android"]["priority"], "HIGH");
        assert_eq!(msg["android"]["collapseKey"], "fpush");
        assert!(msg["notification"].is_null());
    }

    fn fcm_error(body: serde_json::Value) -> PushError {
        map_fcm_error(
            &serde_json::from_value::<FcmErrorResponse>(body)
                .unwrap()
                .error,
        )
    }

    #[test]
    fn unregistered_token_is_blocked() {
        let err = fcm_error(serde_json::json!({"error": {
            "code": 404, "message": "Requested entity was not found.", "status": "NOT_FOUND",
            "details": [{"@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError",
                         "errorCode": "UNREGISTERED"}]}}));
        assert!(matches!(err, PushError::TokenBlocked));
    }

    #[test]
    fn unavailable_is_temporary() {
        let err = fcm_error(serde_json::json!({"error": {
            "code": 503, "message": "The service is currently unavailable.", "status": "UNAVAILABLE",
            "details": [{"@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError",
                         "errorCode": "UNAVAILABLE"}]}}));
        assert!(matches!(err, PushError::PushEndpointTmp));
    }

    #[test]
    fn other_detail_types_and_unknown_codes() {
        let err = fcm_error(serde_json::json!({"error": {
            "code": 500, "message": "Internal error.", "status": "INTERNAL",
            "details": [{"@type": "type.googleapis.com/google.rpc.BadRequest", "fieldViolations": []},
                        {"@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError",
                         "errorCode": "NOT_A_REAL_CODE"}]}}));
        assert!(matches!(err, PushError::PushEndpointTmp));
    }

    #[test]
    fn falls_back_to_http_status() {
        let err = fcm_error(serde_json::json!({"error": {
            "code": 503, "message": "", "status": "UNAVAILABLE"}}));
        assert!(matches!(err, PushError::PushEndpointTmp));
        let err = fcm_error(serde_json::json!({"error": {
            "code": 400, "message": "", "status": "INVALID_ARGUMENT"}}));
        assert!(matches!(err, PushError::Unknown(400)));
    }
}
