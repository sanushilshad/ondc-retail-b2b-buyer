use secrecy::SecretString;
use sqlx::{types::BigDecimal, FromRow};
use uuid::Uuid;

use crate::{
    routes::order::schemas::{PaymentSettlementPhase, PaymentSettlementType},
    schemas::FeeType,
};

#[derive(Debug, FromRow)]
pub struct RegisteredNetworkParticipantModel {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub logo: String,
    pub signing_key: SecretString,
    pub subscriber_id: String,
    pub subscriber_uri: String,
    pub long_description: String,
    pub short_description: String,
    pub fee_type: FeeType,
    pub fee_value: BigDecimal,
    pub unique_key_id: String,
    pub settlement_phase: PaymentSettlementPhase,
    pub settlement_type: PaymentSettlementType,
    pub bank_account_no: String,
    pub bank_ifsc_code: String,
    pub bank_beneficiary_name: String,
    pub bank_name: String,
}

#[derive(Debug)]
pub struct SeriesNoModel {
    pub prefix: String,
    pub series_no: i64,
}

impl SeriesNoModel {
    pub fn get_final_no(&self) -> String {
        format!("{}{}", self.prefix, self.series_no)
    }
}
