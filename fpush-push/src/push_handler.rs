use crate::error::{PushRequestError, PushRequestResult};

use crate::push_module::PushModuleEnum;
use fpush_traits::push::PushError;

use log::debug;

#[inline(always)]
pub async fn handle_push_request(
    push_module: &PushModuleEnum,
    token: String,
) -> PushRequestResult<()> {
    if push_module.blocklist().is_blocked(&token) {
        return Err(PushRequestError::TokenBlocked);
    }
    if push_module
        .ratelimit()
        .lookup_ratelimit(token.to_string())
        .await
    {
        match push_module.send(token.to_string()).await {
            Ok(()) => {
                debug!(
                    "{}: Send push message to token {}",
                    push_module.identifier(),
                    token
                );
                Ok(())
            }
            Err(PushError::TokenBlocked) => {
                debug!(
                    "{}: Received push request from blocked token {}",
                    push_module.identifier(),
                    token,
                );
                push_module.blocklist().block(token);
                Err(PushRequestError::TokenBlocked)
            }
            Err(PushError::TokenRateLimited) => {
                // TODO ratelimit hard
                push_module.ratelimit().hard_ratelimit(token.to_string());
                Err(PushRequestError::TokenRatelimited)
            }
            Err(e) => {
                debug!(
                    "{}: Blocking token {} due to error: {}",
                    push_module.identifier(),
                    token,
                    e
                );
                push_module.blocklist().block(token);
                Err(PushRequestError::Internal)
            }
        }
    } else {
        debug!(
            "{}: Ignoring push request for token {} due to ratelimit",
            push_module.identifier(),
            token,
        );
        Err(PushRequestError::TokenRatelimited)
    }
}
