use actix::Addr;
use actix_web::web;
use sqlx::PgPool;

use super::errors::ONDCBuyerError;
use super::schemas::{ONDCOnSearchRequest, ONDCOnSelectRequest, WSError, WSSelect};
use super::utils::{
    get_ondc_order_params, get_product_from_on_search_request, get_product_search_params,
    get_search_ws_body, get_websocket_params_from_search_req,
};
use crate::routes::ondc::{ONDCActionType, ONDCBuyerErrorCode, ONDCResponse};
use crate::schemas::WSKeyTrait;
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
    let product_objs = get_product_from_on_search_request(&body).map_err(|e| {
        ONDCBuyerError::InvalidResponseError {
            path: None,
            message: e.to_string(),
        }
    })?;

    if let Some(product_objs) = product_objs {
        if !product_objs.providers.is_empty() && !extracted_search_obj.update_cache {
            let ws_params = get_websocket_params_from_search_req(extracted_search_obj);
            let ws_body = get_search_ws_body(
                body.context.message_id,
                body.context.transaction_id,
                product_objs,
            );
            let ws_json = serde_json::to_value(ws_body).unwrap();
            let msg = MessageToClient::new(
                WebSocketActionType::Search,
                ws_json,
                Some(ws_params.get_key()),
            );
            websocket_srv.do_send(msg);
        }
    }

    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On select Payload", skip(pool), fields())]
pub async fn on_select(
    pool: web::Data<PgPool>,
    body: ONDCOnSelectRequest,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let ws_obj = WSSelect {
        transaction_id: body.context.transaction_id,
        message_id: body.context.message_id,
        action_type: WebSocketActionType::Select,
    };
    let mut ws_json = serde_json::to_value(ws_obj).unwrap();
    let ws_params_obj = get_ondc_order_params(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
        ONDCActionType::Select,
    )
    .await
    .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let ws_params =
        ws_params_obj.ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    if let Some(error) = body.error {
        let ws_error_obj = WSError {
            transaction_id: body.context.transaction_id,
            message_id: body.context.message_id,
            action_type: WebSocketActionType::Select,
            error_message: error.message,
        };
        ws_json = serde_json::to_value(ws_error_obj).unwrap();
    };

    let msg = MessageToClient::new(
        WebSocketActionType::Search,
        ws_json,
        Some(ws_params.get_key()),
    );
    websocket_srv.do_send(msg);
    //     }
    // }
    Ok(web::Json(ONDCResponse::successful_response(None)))
}
