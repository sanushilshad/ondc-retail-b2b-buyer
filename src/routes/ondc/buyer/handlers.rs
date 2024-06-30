use actix::Addr;
use actix_web::web;
use sqlx::PgPool;

use super::errors::ONDCBuyerError;
use super::schemas::ONDCOnSearchRequest;
use super::utils::{get_product_search_params, get_websocket_params_from_search_req};
use crate::routes::ondc::{ONDCBuyerErrorCode, ONDCResponse};
use crate::websocket::{MessageToClient, Server, WebSocketActionType};

#[tracing::instrument(name = "ONDC On Search Payload", skip(pool), fields())]
pub async fn on_search(
    pool: web::Data<PgPool>,
    body: ONDCOnSearchRequest,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let json_obj = serde_json::to_value(&body).unwrap();
    let search_obj = get_product_search_params(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
    )
    .await
    .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let extracted_search_obj =
        search_obj.ok_or_else(|| ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    match extracted_search_obj.is_real_time {
        true => {
            let ws_params = get_websocket_params_from_search_req(extracted_search_obj);
            let msg = MessageToClient::new(
                WebSocketActionType::Search,
                json_obj,
                Some(ws_params.get_key()),
            );
            websocket_srv.do_send(msg);
        }
        false => todo!(),
    }

    Ok(web::Json(ONDCResponse::successful_response(None)))
}
