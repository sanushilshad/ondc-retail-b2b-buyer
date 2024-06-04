// impl_serialize_format!(InventoryRequest, Display);
// #[derive(sqlx::FromRow, Serialize, Deserialize)]
// pub struct ProductInventory {
//     #[sqlx(rename = "code")]
//     product_code: String,
//     #[sqlx(rename = "no_of_items")]
//     qty: BigDecimal,
// }

// impl_serialize_format!(MyResponse, Display);
// #[derive(Serialize, Deserialize)]
// pub struct MyResponse {
//     pub status: bool,
//     pub customer_message: String,
//     pub success_code: String,
//     pub data: Vec<ProductInventory>,
// }

// impl_serialize_format!(InventoryRequest, Debug);
// #[derive(Deserialize, Serialize)]
// pub struct InventoryRequest {
//     username: String,
//     session_id: String,
//     product_codes: Vec<String>,
// }

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schemas::CountryCode;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    PrePaid,
    CashOnDelivery,
    Credit,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]

pub enum FulfillmentType {
    Delivery,
    SelfPickup,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProductSearchType {
    Item,
    Fulfillment,
    Category,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductFulFillmentLocations {
    pub latitude: f64,
    pub longitude: f64,
    pub area_code: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchRequest {
    pub query: String,
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub domain_category_code: String,
    pub country_code: CountryCode,

    pub payment_type: Option<PaymentType>,
    pub fulfillment_type: FulfillmentType,
    pub search_type: ProductSearchType,
    pub fulfillment_locations: Option<Vec<ProductFulFillmentLocations>>,
}
