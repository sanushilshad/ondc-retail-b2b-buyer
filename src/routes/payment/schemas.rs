use crate::errors::GenericError;

use crate::payment_client::PaymentServiceStatusType;
use crate::routes::order::schemas::PaymentCollectedBy;
use crate::routes::order::schemas::PaymentStatus;
use crate::routes::product::schemas::PaymentType;
use actix_http::Payload;
use actix_web::web::Json;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentOrderRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
}
impl FromRequest for CreatePaymentOrderRequest {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct CommercePaymentMetaData {
    pub id: Uuid,
    pub payment_type: PaymentType,
    pub payment_status: Option<PaymentStatus>,
    pub payment_order_id: Option<String>,
    pub collected_by: Option<PaymentCollectedBy>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOrderData {
    pub order_id: String,
    pub status: PaymentStatus,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentNotificationRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    pub payment_order_id: String,
    pub payment_id: String,
    pub status: PaymentServiceStatusType,
}
impl FromRequest for PaymentNotificationRequest {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSPayment {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    pub message: String,
}
