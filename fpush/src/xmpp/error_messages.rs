use log::error;
use tokio::sync::mpsc;
use xmpp::jid::Jid;
use xmpp_parsers::{iq::Iq, stanza_error::StanzaError};

#[inline(always)]
pub async fn send_ack_iq(conn: &mpsc::Sender<Iq>, id: &str, jid: Jid, from: Jid) {
    if let Err(e) = conn
        .send(Iq::empty_result(jid, (*id).to_string()).with_from(from))
        .await
    {
        error!("Could not forward outgoing iq to main handler: {}", e);
    }
}

#[inline(always)]
pub async fn send_error_policy_iq(conn: &mpsc::Sender<Iq>, id: &str, jid: Jid, from: Jid) {
    let error_stanza = StanzaError::new(
        xmpp_parsers::stanza_error::ErrorType::Cancel,
        xmpp_parsers::stanza_error::DefinedCondition::PolicyViolation,
        "en",
        "A error occured",
    );
    if let Err(e) = conn
        .send(
            Iq::from_error((*id).to_string(), error_stanza)
                .with_to(jid)
                .with_from(from),
        )
        .await
    {
        error!("Could not forward outgoing iq to main handler: {}", e);
    }
}

#[inline(always)]
pub async fn send_error_iq(conn: &mpsc::Sender<Iq>, id: &str, jid: Jid, from: Jid) {
    let error_stanza = StanzaError::new(
        xmpp_parsers::stanza_error::ErrorType::Cancel,
        xmpp_parsers::stanza_error::DefinedCondition::BadRequest,
        "en",
        "A error occured",
    );
    if let Err(e) = conn
        .send(
            Iq::from_error((*id).to_string(), error_stanza)
                .with_to(jid)
                .with_from(from),
        )
        .await
    {
        error!("Could not forward outgoing iq to main handler: {}", e);
    }
}

#[inline(always)]
pub async fn send_wait_iq_reason_old_prosody(
    conn: &mpsc::Sender<Iq>,
    id: &str,
    jid: Jid,
    from: Jid,
) {
    let error_stanza = StanzaError::new(
        xmpp_parsers::stanza_error::ErrorType::Wait,
        xmpp_parsers::stanza_error::DefinedCondition::BadRequest,
        "en",
        "Invalid push format, update your prosody community modules (debian stable has a bug, use backports instead)",
    );
    if let Err(e) = conn
        .send(
            Iq::from_error((*id).to_string(), error_stanza)
                .with_to(jid)
                .with_from(from),
        )
        .await
    {
        error!("Could not forward outgoing iq to main handler: {}", e);
    }
}
