use crate::errors::GenericError;
use actix_http::Payload;
use actix_web::web::Json;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOrderCreationRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
}
impl FromRequest for PaymentOrderCreationRequest {
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
