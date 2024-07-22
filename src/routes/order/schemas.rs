use crate::errors::GenericError;
use actix_http::Payload;
use actix_web::{web, FromRequest, HttpRequest};
use futures_util::future::LocalBoxFuture;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Deserialize, Debug, ToSchema)]
pub struct OrderSelectRequest {
    pub transaction_id: Uuid,
    pub message_id: Uuid,
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
