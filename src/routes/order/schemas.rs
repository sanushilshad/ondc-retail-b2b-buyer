use crate::routes::product::schemas::PaymentType;
use crate::{errors::GenericError, routes::product::schemas::FulfillmentType};
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
    pub location_ids: Vec<String>,
    pub quantity: i32,
    pub buyer_term: Option<BuyerTerms>,
    pub fulfillment_ids: Vec<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SelectFulfillmentLocation {
    pub gps: String,
    pub area_code: String,
    pub city: String,
    pub country: String,
    pub state: String,
    pub contact_mobile: Option<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderSelectFulfillment {
    pub id: String,
    pub r#type: FulfillmentType,
    pub locations: Vec<SelectFulfillmentLocation>,
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
    pub payments: Vec<PaymentType>,
    pub provider_id: String,
    pub items: Vec<OrderSelectItem>,
    pub ttl: String,
    pub fulfillment_types: Vec<OrderSelectFulfillment>,
    pub order_type: OrderType,
    pub bpp_id: String,
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
