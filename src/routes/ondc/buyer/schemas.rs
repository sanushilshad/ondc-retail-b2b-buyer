use serde::{Deserialize, Serialize};

use crate::routes::ondc::schemas::ONDCContext;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentTagType {
    BuyerId,
    BapTerms,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MainIntentTagDescriptor {
    code: IntentTagType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentTagValueCode {
    BuyerIdCode,
    BuyerIdNo,
    FinderFeeType,
    FinderFeeAmount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentTagDescriptor {
    code: IntentTagValueCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentTagValue {
    descriptor: IntentTagDescriptor,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCIntentTag {
    descriptor: MainIntentTagDescriptor,
    list: Vec<IntentTagValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchLocation {
    gps: String,
    area_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchStop {
    r#type: String,
    location: ONDCSearchLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchFulfillment {
    r#type: String,
    stops: ONDCSearchStop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchDescriptor {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchCategory {
    pub id: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCItem {
    pub descriptor: Option<ONDCSearchDescriptor>,
    pub category: Option<ONDCSearchCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCPaymentType {
    #[serde(rename = "PRE-FULFILLMENT")]
    PreFulfillment,
    #[serde(rename = "ON-FULFILLMENT")]
    OnFulfillment,
    #[serde(rename = "POST-FULFILLMENT")]
    PostFulfillment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchPayment {
    r#type: ONDCPaymentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchProvider {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchIntent {
    item: Option<ONDCItem>,
    fulfillment: ONDCSearchFulfillment,
    tags: Vec<ONDCIntentTag>,
    payment: ONDCSearchPayment,
    provider: Option<ONDCSearchProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchMessage {
    intent: ONDCSearchIntent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchRequest {
    pub context: ONDCContext,
    pub message: ONDCSearchMessage,
}
