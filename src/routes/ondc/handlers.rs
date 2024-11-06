use actix_web::web;
use sqlx::PgPool;

use super::errors::ONDCBuyerError;
use super::schemas::{
    ONDCOnConfirmRequest, ONDCOnInitRequest, ONDCOnSearchRequest, ONDCOnSelectRequest,
    ONDCSelectRequest, WSConfirm, WSConfirmData, WSInit, WSInitData, WSSelect,
};
use super::utils::{
    fetch_ondc_order_request, get_ondc_order_param_from_req, get_product_from_on_search_request,
    get_product_search_params, get_search_ws_body, get_websocket_params_from_search_req,
    save_ondc_seller_info, save_ondc_seller_location_info, save_ondc_seller_product_info,
};
use super::{ONDCOnStatusRequest, WSStatus};
use crate::constants::ONDC_TTL;
use crate::routes::ondc::{ONDCActionType, ONDCBuyerErrorCode, ONDCResponse};
use crate::routes::order::utils::{
    fetch_order_by_id, initialize_order_on_confirm, initialize_order_on_init,
    initialize_order_on_select, initialize_order_on_status,
};
use crate::routes::product::schemas::WSSearchData;
use crate::user_client::CustomerType;
use crate::user_client::UserClient;
use crate::websocket_client::{WebSocketActionType, WebSocketClient};

#[tracing::instrument(name = "ONDC On Search Payload", skip(pool, body), fields())]
pub async fn on_search(
    pool: web::Data<PgPool>,
    body: ONDCOnSearchRequest,
    websocket_srv: web::Data<WebSocketClient>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let search_obj =
        get_product_search_params(&pool, body.context.transaction_id, body.context.message_id)
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
                let _ = websocket_srv
                    .send_msg(ws_params, WebSocketActionType::Search, ws_json)
                    .await;
            }
            let task1 = save_ondc_seller_product_info(&pool, &product_objs);
            let task2 = save_ondc_seller_info(&pool, &product_objs);
            let task3 = save_ondc_seller_location_info(&pool, &product_objs);
            let (_, _, _) = futures::future::join3(task1, task2, task3).await;
        }
    }

    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On select Payload", skip(pool), fields())]
pub async fn on_select(
    pool: web::Data<PgPool>,
    body: ONDCOnSelectRequest,
    websocket_srv: web::Data<WebSocketClient>,
    user_client: web::Data<UserClient>,
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
        body.context.transaction_id,
        body.context.message_id,
        &ONDCActionType::Select,
    )
    .await
    .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
    .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    let ws_json = serde_json::to_value(ws_obj).unwrap();
    let ws_params_obj = get_ondc_order_param_from_req(&ondc_select_model);

    let ondc_select_req =
        serde_json::from_value::<ONDCSelectRequest>(ondc_select_model.request_payload).unwrap();
    let is_rfq = ondc_select_req.context.ttl != ONDC_TTL;
    if (is_rfq) || body.error.is_none() {
        let user = user_client
            .get_user_account(None, Some(ondc_select_model.user_id))
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
        let business_account = user_client
            .get_business_account(
                ondc_select_model.user_id,
                ondc_select_model.business_id,
                vec![CustomerType::RetailB2bBuyer],
            )
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
            .ok_or(ONDCBuyerError::BuyerInternalServerError { path: None })?;

        initialize_order_on_select(&pool, &body, &user, &business_account, &ondc_select_req)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    };
    let _ = websocket_srv
        .send_msg(ws_params_obj, WebSocketActionType::Select, ws_json)
        .await;
    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On Init Payload", skip(pool), fields())]
pub async fn on_init(
    pool: web::Data<PgPool>,
    body: ONDCOnInitRequest,
    websocket_srv: web::Data<WebSocketClient>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let order_request_model = fetch_ondc_order_request(
        &pool,
        body.context.transaction_id,
        body.context.message_id,
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

    initialize_order_on_init(&pool, &body)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    let _ = websocket_srv
        .send_msg(ws_params_obj, WebSocketActionType::Init, ws_json)
        .await;

    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On confirm Payload", skip(pool), fields())]
pub async fn on_confirm(
    pool: web::Data<PgPool>,
    body: ONDCOnConfirmRequest,
    websocket_srv: web::Data<WebSocketClient>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let task1 = fetch_ondc_order_request(
        &pool,
        body.context.transaction_id,
        body.context.message_id,
        &ONDCActionType::Confirm,
    );

    let task2 = fetch_order_by_id(&pool, body.context.transaction_id);
    let (res1, res2) = futures::future::join(task1, task2).await;
    let order_request_model = res1
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
        .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    let order = res2
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
        .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;

    let payment_links: Vec<&str> = body
        .message
        .order
        .payments
        .iter()
        .filter_map(|payment| payment.uri.as_deref())
        .collect();
    let ws_confirm_data = (!payment_links.is_empty()).then_some(WSConfirmData { payment_links });
    let ws_obj = WSConfirm {
        transaction_id: body.context.transaction_id,
        message_id: body.context.message_id,
        error: body
            .error
            .as_ref()
            .map_or_else(|| None, |s| Some(s.message.as_str())),
        data: ws_confirm_data,
    };
    let ws_json = serde_json::to_value(ws_obj).unwrap();
    let ws_params_obj = get_ondc_order_param_from_req(&order_request_model);

    initialize_order_on_confirm(&pool, &body, &order)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    let _ = websocket_srv
        .send_msg(ws_params_obj, WebSocketActionType::Confirm, ws_json)
        .await;

    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On status Payload", skip(pool), fields())]
pub async fn on_status(
    pool: web::Data<PgPool>,
    body: ONDCOnStatusRequest,
    websocket_srv: web::Data<WebSocketClient>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let task1 = fetch_ondc_order_request(
        &pool,
        body.context.transaction_id,
        body.context.message_id,
        &ONDCActionType::Status,
    );

    let task2 = fetch_order_by_id(&pool, body.context.transaction_id);
    let (res1, res2) = futures::future::join(task1, task2).await;
    let order_request_model =
        res1.map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let order = res2
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
        .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;

    initialize_order_on_status(&pool, &body, &order)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    if let Some(order_request_model) = order_request_model {
        let ws_obj = WSStatus {
            transaction_id: body.context.transaction_id,
            message_id: body.context.message_id,
            error: body
                .error
                .as_ref()
                .map_or_else(|| None, |s| Some(s.message.to_owned())),
        };
        let ws_json = serde_json::to_value(ws_obj).unwrap();
        let ws_params_obj = get_ondc_order_param_from_req(&order_request_model);
        let _ = websocket_srv
            .send_msg(ws_params_obj, WebSocketActionType::Status, ws_json)
            .await;
    }

    Ok(web::Json(ONDCResponse::successful_response(None)))
}
