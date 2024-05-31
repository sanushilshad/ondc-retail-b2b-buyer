use secrecy::Secret;
use sqlx::{types::BigDecimal, FromRow};
use uuid::Uuid;

use crate::schemas::FeeType;

#[derive(Debug, FromRow)]
pub struct RegisteredNetworkParticipantModel {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub logo: String,
    pub signing_key: Secret<String>,
    pub subscriber_id: String,
    pub subscriber_uri: String,
    pub long_description: String,
    pub short_description: String,
    pub fee_type: FeeType,
    pub fee_value: BigDecimal,
    pub unique_key_id: String,
}
