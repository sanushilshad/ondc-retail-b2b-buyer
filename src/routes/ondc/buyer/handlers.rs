use actix::Addr;
use actix_web::web;
use sqlx::PgPool;

use super::errors::ONDCBuyerError;
use super::schemas::ONDCOnSearchRequest;
use super::utils::{
    get_product_from_on_search_request, get_product_search_params, get_search_ws_body,
    get_websocket_params_from_search_req,
};
use crate::routes::ondc::{ONDCBuyerErrorCode, ONDCResponse};
use crate::websocket::{MessageToClient, Server, WebSocketActionType};

#[tracing::instrument(name = "ONDC On Search Payload", skip(pool), fields())]
pub async fn on_search(
    pool: web::Data<PgPool>,
    body: ONDCOnSearchRequest,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let _products = get_product_from_on_search_request(&body)
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let search_obj = get_product_search_params(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
    )
    .await
    .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let extracted_search_obj =
        search_obj.ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    let products = get_product_from_on_search_request(&body).map_err(|e| {
        ONDCBuyerError::InvalidResponseError {
            path: None,
            message: e.to_string(),
        }
    })?;
    if products.is_empty() {
        return Ok(web::Json(ONDCResponse::successful_response(None)));
    }
    if extracted_search_obj.update_cache {
        let ws_params = get_websocket_params_from_search_req(extracted_search_obj);
        let ws_body = get_search_ws_body(
            body.context.message_id,
            body.context.transaction_id,
            products,
        );
        let ws_json = serde_json::to_value(ws_body).unwrap();
        let msg = MessageToClient::new(
            WebSocketActionType::Search,
            ws_json,
            Some(ws_params.get_key()),
        );
        websocket_srv.do_send(msg);
    }
    // else {
    // }

    Ok(web::Json(ONDCResponse::successful_response(None)))
}
