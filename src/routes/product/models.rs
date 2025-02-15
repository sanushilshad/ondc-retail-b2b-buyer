use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    routes::{ondc::ONDCItemUOM, order::schemas::FulfillmentStatusType},
    schemas::ONDCNetworkType,
};

use super::schemas::{CredentialType, PaymentType};
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchProviderContactModel {
    pub mobile_no: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchProviderTermsModel {
    pub gst_credit_invoice: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchProviderCredentialModel {
    pub id: String,
    pub r#type: CredentialType,
    pub desc: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductVariantAttributeModel {
    pub attribute_code: String,
    pub sequence: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderPaymentOptionModel {
    pub id: String,
    pub r#type: PaymentType,
    pub collected_by: ONDCNetworkType,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReplacementTermModel {
    pub replace_within: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnTimeModel {
    pub duration: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnLocationModel {
    pub address: String,
    pub gps: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnTermModel {
    pub fulfillment_state: FulfillmentStatusType,
    pub return_eligible: bool,
    pub return_time: WSItemReturnTimeModel,
    pub return_location: WSItemReturnLocationModel,
    pub fulfillment_managed_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum WSItemCancellationFeeModel {
    Percent { percentage: f64 },
    Amount { amount: f64 },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemCancellationTermModel {
    pub fulfillment_state: FulfillmentStatusType,
    pub reason_required: bool,
    pub fee: WSItemCancellationFeeModel,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemCancellationModel {
    pub is_cancellable: bool,
    pub terms: Vec<WSItemCancellationTermModel>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemAttributeModel {
    pub label: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
#[skip_serializing_none]
#[serde(rename_all = "snake_case")]
pub struct WSPriceSlabModel {
    pub min: BigDecimal,
    pub max: Option<BigDecimal>,
    pub price_with_tax: BigDecimal,
    pub price_without_tax: BigDecimal,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSCreatorContactDataModel {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSProductCreatorModel {
    pub name: String,
    pub contact: WSCreatorContactDataModel,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSProductCategoryModel {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemValidityModel {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQtyModel {
    pub measure: WSSearchItemQtyMeasureModel,
    pub count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQtyMeasureModel {
    pub unit: ONDCItemUOM,
    pub value: BigDecimal,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQuantityModel {
    pub unitized: WSSearchItemQtyMeasureModel,
    pub available: WSSearchItemQtyModel,
    pub maximum: WSSearchItemQtyModel,
    pub minimum: Option<WSSearchItemQtyModel>,
}
