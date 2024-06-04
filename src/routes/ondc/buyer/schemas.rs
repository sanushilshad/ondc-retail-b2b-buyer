use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::routes::ondc::schemas::ONDCContext;
use crate::routes::product::schemas::{FulfillmentType, PaymentType};
use crate::routes::schemas::VectorType;
use crate::schemas::{FeeType, ONDCNetworkType};

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
#[serde(rename_all = "snake_case")]
pub enum ONDCFulfillmentStopType {
    Start,
    End,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentTagDescriptor {
    pub code: IntentTagValueCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentTagValue {
    pub descriptor: IntentTagDescriptor,
    pub value: String,
}

impl IntentTagValue {
    pub fn get_buyer_fee_type(fee_type: ONDCFeeType) -> IntentTagValue {
        IntentTagValue {
            descriptor: IntentTagDescriptor {
                code: IntentTagValueCode::FinderFeeType,
            },
            value: fee_type.to_string(),
        }
    }
    pub fn get_buyer_fee_amount(fee_amount: &str) -> IntentTagValue {
        IntentTagValue {
            descriptor: IntentTagDescriptor {
                code: IntentTagValueCode::FinderFeeType,
            },
            value: fee_amount.to_owned(),
        }
    }

    pub fn get_buyer_id_code(id_code: &VectorType) -> IntentTagValue {
        IntentTagValue {
            descriptor: IntentTagDescriptor {
                code: IntentTagValueCode::BuyerIdCode,
            },
            value: id_code.to_string(),
        }
    }
    pub fn get_buyer_id_no(id_no: &str) -> IntentTagValue {
        IntentTagValue {
            descriptor: IntentTagDescriptor {
                code: IntentTagValueCode::BuyerIdNo,
            },
            value: id_no.to_string(),
        }
    }
}

// #[derive(Debug, Serialize)]
// #[serde(rename_all = "lowercase")]
// pub enum ONDCTagType{
//     BuyerId,
//     BapTerms
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ONDCFeeType {
    Percent,
    Amount,
}

impl std::fmt::Display for ONDCFeeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ONDCFeeType::Percent => "percent",
            ONDCFeeType::Amount => "amount",
        };

        write!(f, "{}", s)
    }
}

impl ONDCFeeType {
    pub fn get_fee_type(fee_type: &FeeType) -> Self {
        match fee_type {
            FeeType::Percent => ONDCFeeType::Percent,
            FeeType::Amount => ONDCFeeType::Amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCIntentTag {
    pub descriptor: MainIntentTagDescriptor,
    pub list: Vec<IntentTagValue>,
}

impl ONDCIntentTag {
    pub fn get_buyer_fee_tag(
        finder_fee_type: ONDCFeeType,
        finder_fee_amount: &str,
    ) -> ONDCIntentTag {
        ONDCIntentTag {
            descriptor: MainIntentTagDescriptor {
                code: IntentTagType::BapTerms,
            },
            list: vec![
                IntentTagValue::get_buyer_fee_type(finder_fee_type),
                IntentTagValue::get_buyer_fee_amount(finder_fee_amount),
            ],
        }
    }

    pub fn get_buyer_id_tag(id_code: &VectorType, id_no: &str) -> ONDCIntentTag {
        ONDCIntentTag {
            descriptor: MainIntentTagDescriptor {
                code: IntentTagType::BuyerId,
            },
            list: vec![
                IntentTagValue::get_buyer_id_code(id_code),
                IntentTagValue::get_buyer_id_no(id_no),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchLocation {
    pub gps: String,
    pub area_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchStop {
    pub r#type: ONDCFulfillmentStopType,
    pub location: ONDCSearchLocation,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCFulfillmentType {
    #[serde(rename = "Delivery")]
    Delivery,
    #[serde(rename = "Self-Pickup")]
    SelfPickup,
}

impl ONDCFulfillmentType {
    pub fn get_ondc_fulfillment(payment_type: &FulfillmentType) -> ONDCFulfillmentType {
        match payment_type {
            FulfillmentType::Delivery => ONDCFulfillmentType::Delivery,
            FulfillmentType::SelfPickup => ONDCFulfillmentType::SelfPickup,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchFulfillment {
    pub r#type: ONDCFulfillmentType,
    pub stops: Option<Vec<ONDCSearchStop>>,
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
pub struct ONDCSearchItem {
    pub descriptor: Option<ONDCSearchDescriptor>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]

pub enum ONDCPaymentType {
    #[serde(rename = "PRE-FULFILLMENT")]
    PreFulfillment,
    #[serde(rename = "ON-FULFILLMENT")]
    OnFulfillment,
    #[serde(rename = "POST-FULFILLMENT")]
    PostFulfillment,
}

impl ONDCPaymentType {
    pub fn get_ondc_payment(payment_type: &PaymentType) -> ONDCPaymentType {
        match payment_type {
            PaymentType::CashOnDelivery => ONDCPaymentType::OnFulfillment,
            PaymentType::PrePaid => ONDCPaymentType::PreFulfillment,
            PaymentType::Credit => ONDCPaymentType::PostFulfillment,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchPayment {
    pub r#type: ONDCPaymentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchProvider {
    id: String,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchIntent {
    pub item: Option<ONDCSearchItem>,
    pub fulfillment: Option<ONDCSearchFulfillment>,
    pub tags: Vec<ONDCIntentTag>,
    pub payment: Option<ONDCSearchPayment>,
    pub provider: Option<ONDCSearchProvider>,
    pub category: Option<ONDCSearchCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchMessage {
    pub intent: ONDCSearchIntent,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchRequest {
    pub context: ONDCContext,
    pub message: ONDCSearchMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCError {
    r#type: String,
    code: String,
    path: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OnSearchContentType {
    #[serde(rename = "text/html")]
    Html,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchAdditionalDescriptor {
    pub url: String,
    pub content_type: OnSearchContentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchImage {
    url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchDescriptor {
    name: String,
    code: Option<String>,
    short_desc: String,
    long_desc: String,
    additional_desc: Option<ONDCOnSearchAdditionalDescriptor>,
    images: Vec<ONDCOnSearchImage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchPayment {
    pub id: String,
    pub payment_type: ONDCPaymentType,
    pub collected_by: ONDCNetworkType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCFullFillment {
    pub id: String,
    pub fulfillment_type: ONDCFulfillmentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCredential {
    pub id: String,
    pub r#type: String,
    pub desc: String,
    pub icon: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchProvider {
    id: String,
    payments: Option<Vec<ONDCOnSearchPayment>>,
    rating: String,
    ttl: String,
    creds: Option<Vec<ONDCCredential>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchCatalog {
    descriptor: ONDCOnSearchDescriptor,
    payments: Vec<ONDCOnSearchPayment>,
    fulfillments: Vec<ONDCFullFillment>,
    providers: Vec<ONDCOnSearchProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchMessage {
    catalog: Option<ONDCOnSearchCatalog>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchRequest {
    pub context: ONDCContext,
    pub message: ONDCOnSearchMessage,
    pub error: Option<ONDCError>,
}
