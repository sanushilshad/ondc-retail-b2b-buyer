use actix::Addr;
use actix_web::web;
use sqlx::PgPool;

use super::errors::ONDCBuyerError;
use super::schemas::{
    ONDCOnInitRequest, ONDCOnSearchRequest, ONDCOnSelectRequest, ONDCSelectRequest, WSInit,
    WSInitData, WSSelect,
};
use super::utils::{
    fetch_ondc_order_request, get_ondc_order_param_from_req, get_product_from_on_search_request,
    get_product_search_params, get_search_ws_body, get_websocket_params_from_search_req,
    save_ondc_seller_product_info,
};
use crate::constants::ONDC_TTL;
use crate::routes::ondc::{ONDCActionType, ONDCBuyerErrorCode, ONDCResponse};
use crate::routes::order::utils::{initialize_order_on_init, initialize_order_on_select};
use crate::routes::product::schemas::WSSearchData;

use crate::routes::user::utils::{get_business_account, get_user};
use crate::schemas::WSKeyTrait;
use crate::websocket::{MessageToClient, Server, WebSocketActionType};
#[tracing::instrument(name = "ONDC On Search Payload", skip(pool, body), fields())]
pub async fn on_search(
    pool: web::Data<PgPool>,
    body: ONDCOnSearchRequest,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let search_obj = get_product_search_params(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
    )
    .await
    .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let extracted_search_obj =
        search_obj.ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    let product_objs: Option<WSSearchData<'_>> = get_product_from_on_search_request(&body)
        .map_err(|e| ONDCBuyerError::InvalidResponseError {
            path: None,
            message: e.to_string(),
        })?;

    if let Some(product_objs) = product_objs {
        if !product_objs.providers.is_empty() {
            if !extracted_search_obj.update_cache {
                let ws_params = get_websocket_params_from_search_req(extracted_search_obj);
                let ws_body = get_search_ws_body(
                    body.context.message_id,
                    body.context.transaction_id,
                    &product_objs,
                );
                let ws_json = serde_json::to_value(ws_body).unwrap();
                let msg = MessageToClient::new(
                    WebSocketActionType::Search,
                    ws_json,
                    Some(ws_params.get_key()),
                );
                websocket_srv.do_send(msg);
            }
            let _ = save_ondc_seller_product_info(&pool, &product_objs).await;
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
        error: body
            .error
            .as_ref()
            .map_or_else(|| None, |s| Some(s.message.as_str())),
    };
    let ondc_select_model = fetch_ondc_order_request(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
        &ONDCActionType::Select,
    )
    .await
    .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
    .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    // let ws_param_obj = ONDCOrderParams{};
    let ws_json = serde_json::to_value(ws_obj).unwrap();
    let ws_params_obj = get_ondc_order_param_from_req(&ondc_select_model);
    let msg = MessageToClient::new(
        WebSocketActionType::Select,
        ws_json,
        Some(ws_params_obj.get_key()),
    );
    let ondc_select_req =
        serde_json::from_value::<ONDCSelectRequest>(ondc_select_model.request_payload).unwrap();
    let is_rfq = ondc_select_req.context.ttl != ONDC_TTL;
    if (is_rfq) || body.error.is_none() {
        let user_id = &ondc_select_model.user_id.unwrap();
        let business_id = &ondc_select_model.business_id.unwrap();
        let user = get_user(vec![&user_id.to_string()], &pool)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
        let business_account = get_business_account(&pool, user_id, business_id)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
            .ok_or(ONDCBuyerError::BuyerInternalServerError { path: None })?;

        initialize_order_on_select(&pool, &body, &user, &business_account, &ondc_select_req)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    };
    websocket_srv.do_send(msg);
    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On Init Payload", skip(pool), fields())]
pub async fn on_init(
    pool: web::Data<PgPool>,
    body: ONDCOnInitRequest,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let order_request_model = fetch_ondc_order_request(
        &pool,
        &body.context.transaction_id,
        &body.context.message_id,
        &ONDCActionType::Init,
    )
    .await
    .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
    .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    let payment_links: Vec<&str> = body
        .message
        .order
        .payments
        .iter()
        .filter_map(|payment| payment.uri.as_deref())
        .collect();
    let ws_init_data = (!payment_links.is_empty()).then_some(WSInitData { payment_links });
    let ws_obj = WSInit {
        transaction_id: body.context.transaction_id,
        message_id: body.context.message_id,
        action_type: WebSocketActionType::Init,
        error: body
            .error
            .as_ref()
            .map_or_else(|| None, |s| Some(s.message.as_str())),
        data: ws_init_data,
    };
    let ws_json = serde_json::to_value(ws_obj).unwrap();
    let ws_params_obj = get_ondc_order_param_from_req(&order_request_model);
    let msg = MessageToClient::new(
        WebSocketActionType::Init,
        ws_json,
        Some(ws_params_obj.get_key()),
    );

    initialize_order_on_init(&pool, &body)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    websocket_srv.do_send(msg);

    Ok(web::Json(ONDCResponse::successful_response(None)))
}
