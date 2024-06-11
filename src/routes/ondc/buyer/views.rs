use actix::Addr;
use actix_web::web;
use sqlx::PgPool;

use super::schemas::ONDCOnSearchRequest;
use super::utils::get_websocket_params_from_ondc_search_req;
use crate::routes::ondc::errors::ONDCError;
use crate::routes::ondc::ONDCResponse;
use crate::websocket::{MessageToClient, Server};

#[tracing::instrument(name = "ONDC On Search Response", skip(pool), fields())]
pub async fn on_search(
    pool: web::Data<PgPool>,
    body: web::Json<ONDCOnSearchRequest>,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<ONDCResponse>, ONDCError> {
    let json_obj = serde_json::to_value(&body.0).unwrap();

    match get_websocket_params_from_ondc_search_req(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
    )
    .await
    {
        Ok(Some(ws_params)) => {
            let msg = MessageToClient::new("search", json_obj.clone(), Some(ws_params.get_key()));
            websocket_srv.do_send(msg);
        }
        Ok(None) => return Err(ONDCError::InternalServerError { path: None }),
        Err(_) => return Err(ONDCError::InternalServerError { path: None }),
    };
    // Ok(web::Json(ONDCResponse::successful_response(None)))
    return Err(ONDCError::StaleError { path: None });
}
