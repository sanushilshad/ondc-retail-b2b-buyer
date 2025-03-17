use crate::{
    routes::order::schemas::{PaymentSettlementPhase, PaymentSettlementType},
    schemas::{FeeType, RegisteredNetworkParticipant},
};
use secrecy::SecretString;
use sqlx::{types::BigDecimal, FromRow};

#[derive(Debug, FromRow)]
pub struct RegisteredNetworkParticipantModel {
    pub id: i32,
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
    pub observability_token: Option<String>,
}

impl RegisteredNetworkParticipantModel {
    pub fn into_schema(self) -> RegisteredNetworkParticipant {
        RegisteredNetworkParticipant {
            code: self.code,
            name: self.name,
            logo: self.logo,
            signing_key: self.signing_key.to_owned(),
            id: self.id,
            subscriber_id: self.subscriber_id,
            subscriber_uri: self.subscriber_uri,
            long_description: self.long_description,
            short_description: self.short_description,
            fee_type: self.fee_type,
            fee_value: self.fee_value,
            unique_key_id: self.unique_key_id,
            settlement_phase: self.settlement_phase,
            settlement_type: self.settlement_type,
            bank_account_no: self.bank_account_no,
            bank_ifsc_code: self.bank_ifsc_code,
            bank_beneficiary_name: self.bank_beneficiary_name,
            bank_name: self.bank_name,
            observability_token: self.observability_token.map(SecretString::from),
        }
    }
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
