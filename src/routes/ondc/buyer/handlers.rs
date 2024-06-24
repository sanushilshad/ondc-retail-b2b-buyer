use actix::Addr;
use actix_web::web;
use sqlx::PgPool;

use super::errors::ONDCBuyerError;
use super::schemas::ONDCOnSearchRequest;
use super::utils::get_websocket_params_from_ondc_search_req;
use crate::routes::ondc::{ONDCBuyerErrorCode, ONDCResponse};
use crate::websocket::{MessageToClient, Server, WebSocketActionType};

#[tracing::instrument(name = "ONDC On Search Payload", skip(pool), fields())]
pub async fn on_search(
    pool: web::Data<PgPool>,
    body: ONDCOnSearchRequest,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let json_obj = serde_json::to_value(&body).unwrap();

    match get_websocket_params_from_ondc_search_req(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
    )
    .await
    {
        Ok(Some(ws_params)) => {
            let msg = MessageToClient::new(
                WebSocketActionType::Search,
                json_obj,
                Some(ws_params.get_key()),
            );
            websocket_srv.do_send(msg);
        }
        Ok(None) => return Err(ONDCBuyerError::BuyerResponseSequenceError { path: None }),
        Err(_) => return Err(ONDCBuyerError::BuyerInternalServerError { path: None }),
    };
    Ok(web::Json(ONDCResponse::successful_response(None)))
}
