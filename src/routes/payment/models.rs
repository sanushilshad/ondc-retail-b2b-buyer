use super::schemas::CommercePaymentMetaData;
use crate::routes::{
    order::schemas::{PaymentCollectedBy, PaymentStatus},
    product::schemas::PaymentType,
};
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Debug, FromRow)]
pub struct CommercePaymentMetaModel {
    pub payment_type: PaymentType,
    pub payment_status: Option<PaymentStatus>,
    pub payment_order_id: Option<String>,
    pub collected_by: Option<PaymentCollectedBy>,
    pub id: Uuid,
}

impl CommercePaymentMetaModel {
    pub fn into_schema(self) -> CommercePaymentMetaData {
        CommercePaymentMetaData {
            payment_type: self.payment_type,
            payment_status: self.payment_status,
            payment_order_id: self.payment_order_id,
            collected_by: self.collected_by,
            id: self.id,
        }
    }
}
