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

use std::fmt::{Display, Formatter};

use crate::{errors::GenericError, schemas::CountryCode};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    PrePaid,
    CashOnDelivery,
    Credit,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]

pub enum FulfillmentType {
    Delivery,
    SelfPickup,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProductSearchType {
    Item,
    Fulfillment,
    Category,
    City,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductFulFillmentLocations {
    pub latitude: f64,
    pub longitude: f64,
    pub area_code: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchRequest {
    pub query: String,
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub domain_category_code: CategoryDomain,
    pub country_code: CountryCode,
    pub payment_type: Option<PaymentType>,
    pub fulfillment_type: FulfillmentType,
    pub search_type: ProductSearchType,
    pub fulfillment_locations: Option<Vec<ProductFulFillmentLocations>>,
    pub city_code: String,
}

impl FromRequest for ProductSearchRequest {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
pub enum CategoryDomain {
    #[serde(rename = "RET10")]
    Grocery,
}

impl Display for CategoryDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CategoryDomain::Grocery => "RET10",
            }
        )
    }
}
