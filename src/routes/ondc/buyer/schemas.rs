use argon2::password_hash::Decimal;
use serde::{Deserialize, Serialize};

use crate::routes::{
    ondc::schemas::ONDCContext,
    product::schemas::{FulfillmentType, PaymentType},
};

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
    pub fn get_buyer_fee_type(fee_type: BuyerFeeType) -> IntentTagValue {
        IntentTagValue {
            descriptor: IntentTagDescriptor {
                code: IntentTagValueCode::FinderFeeType,
            },
            value: serde_json::to_string(&fee_type).unwrap(),
        }
    }
    pub fn get_buyer_fee_amount(fee_amount: String) -> IntentTagValue {
        IntentTagValue {
            descriptor: IntentTagDescriptor {
                code: IntentTagValueCode::FinderFeeType,
            },
            value: fee_amount,
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
pub enum BuyerFeeType {
    Percentage,
    Amount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCIntentTag {
    pub descriptor: MainIntentTagDescriptor,
    pub list: Vec<IntentTagValue>,
}

impl ONDCIntentTag {
    pub fn get_buyer_tag(
        finder_fee_type: BuyerFeeType,
        finder_fee_amount: String,
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
        return match payment_type {
            FulfillmentType::Delivery => ONDCFulfillmentType::Delivery,
            FulfillmentType::SelfPickup => ONDCFulfillmentType::SelfPickup,
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchFulfillment {
    pub r#type: ONDCFulfillmentType,
    pub stops: Vec<ONDCSearchStop>,
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

impl ONDCPaymentType {
    pub fn get_ondc_payment(payment_type: PaymentType) -> ONDCPaymentType {
        return match payment_type {
            PaymentType::COD => ONDCPaymentType::OnFulfillment,
            PaymentType::Pre_paid => ONDCPaymentType::PreFulfillment,
            PaymentType::Credit => ONDCPaymentType::PostFulfillment,
        };
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchIntent {
    pub item: Option<ONDCItem>,
    pub fulfillment: ONDCSearchFulfillment,
    pub tags: Vec<ONDCIntentTag>,
    pub payment: ONDCSearchPayment,
    pub provider: Option<ONDCSearchProvider>,
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
