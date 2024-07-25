use crate::errors::GenericError;
use crate::routes::product::schemas::FulfillmentType;
use crate::routes::product::schemas::{CategoryDomain, PaymentType};
use crate::schemas::CountryCode;
use crate::utils::deserialize_non_empty_vector;
use actix_http::Payload;
use actix_web::{web, FromRequest, HttpRequest};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuyerTerms {
    pub item_req: String,
    pub packaging_req: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderSelectItem {
    pub item_code: String,
    pub location_ids: Vec<String>,
    pub quantity: i32,
    pub buyer_term: Option<BuyerTerms>,
    pub fulfillment_ids: Vec<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct Country {
    pub code: CountryCode,
    name: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct City {
    pub code: String,
    name: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FulfillmentLocation {
    pub gps: String,
    pub area_code: String,
    pub address: String,
    pub city: City,
    pub country: Country,
    pub state: String,
    pub contact_mobile: Option<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum IncorTermType {
    Exw,
    Cif,
    Fob,
    Dap,
    Ddp,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderDeliveyTerm {
    pub inco_terms: IncorTermType,
    pub place_of_delivery: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderSelectFulfillment {
    pub id: String,
    pub r#type: FulfillmentType,
    #[serde(deserialize_with = "deserialize_non_empty_vector")]
    pub locations: Vec<FulfillmentLocation>,
    pub delivery_terms: Option<OrderDeliveyTerm>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Rfq,
    Sale,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderSelectRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub domain_category_code: CategoryDomain,
    pub payment_types: Vec<PaymentType>,
    pub provider_id: String,
    pub items: Vec<OrderSelectItem>,
    pub ttl: String,
    pub fulfillments: Vec<OrderSelectFulfillment>,
    pub order_type: OrderType,
    pub bpp_id: String,
    pub is_import: bool,
}

impl FromRequest for OrderSelectRequest {
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
