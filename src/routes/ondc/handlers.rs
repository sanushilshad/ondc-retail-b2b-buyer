use actix_web::web;
use anyhow::Context;
use sqlx::PgPool;

use super::errors::ONDCBuyerError;
use super::schemas::{
    ONDCOnConfirmRequest, ONDCOnInitRequest, ONDCOnSearchRequest, ONDCOnSelectRequest,
    ONDCSelectRequest, WSConfirm, WSConfirmData, WSInit, WSInitData, WSSelect,
};
use super::utils::{
    fetch_ondc_order_request, fetch_ondc_seller_info, get_ondc_order_param_from_commerce,
    get_ondc_order_param_from_req, get_ondc_seller_location_info_mapping,
    get_ondc_seller_product_info_mapping, get_product_from_on_search_request,
    get_product_search_params, get_search_ws_body, get_websocket_params_from_search_req,
    save_ondc_seller_info, save_ondc_seller_location_info, save_ondc_seller_product_info,
};
use super::{
    ONDCOnCancelRequest, ONDCOnStatusRequest, ONDCOnUpdateRequest, ONDCRequestType, WSCancel,
    WSStatus, WSUpdate,
};
use crate::chat_client::ChatClient;
use crate::constants::ONDC_TTL;
use crate::routes::ondc::{ONDCActionType, ONDCBuyerErrorCode, ONDCResponse};
use crate::routes::order::utils::{
    fetch_order_by_id, initialize_order_on_cancel, initialize_order_on_confirm,
    initialize_order_on_init, initialize_order_on_select, initialize_order_on_status,
    initialize_order_on_update, send_rfq_accept_chat, send_rfq_cancel_chat,
    send_rfq_confirmed_chat, send_rfq_init_chat, send_rfq_reject_chat, send_rfq_status_chat,
    send_rfq_update_chat,
};
use crate::routes::product::schemas::WSSearchData;
use crate::user_client::CustomerType;
use crate::user_client::UserClient;
use crate::websocket_client::{ProcessType, WebSocketActionType, WebSocketClient};

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
                    .send_msg(
                        ws_params,
                        WebSocketActionType::Search,
                        ws_json,
                        Some(ProcessType::Immediate),
                    )
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
    chat_client: web::Data<ChatClient>,
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
    let business_account = user_client
        .get_business_account(
            ondc_select_model.user_id,
            ondc_select_model.business_id,
            vec![CustomerType::RetailB2bBuyer],
        )
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
        .ok_or(ONDCBuyerError::BuyerInternalServerError { path: None })?;

    let is_rfq = ondc_select_req.context.ttl != ONDC_TTL;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    if body.error.is_none() {
        let task_1 = user_client.get_user_account(None, Some(ondc_select_model.user_id));
        // .await
        // .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
        let bpp_id = body.context.bpp_id.as_deref().unwrap_or("");
        let location_id_list: Vec<String> = body
            .message
            .order
            .provider
            .locations
            .iter()
            .map(|location| location.id.to_owned())
            .collect();
        let item_code_list: Vec<&str> = body
            .message
            .order
            .items
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        let task_2 = get_ondc_seller_product_info_mapping(
            &pool,
            body.context.bpp_id.as_ref().unwrap(),
            &body.message.order.provider.id,
            &item_code_list,
        );

        let task_3 = get_ondc_seller_location_info_mapping(
            &pool,
            bpp_id,
            &body.message.order.provider.id,
            &location_id_list,
        );
        let task_4 = fetch_ondc_seller_info(&pool, bpp_id, &body.message.order.provider.id);
        let (user_res, product_map_res, seller_info_map_res, seller_location_map_res) =
            futures::future::join4(task_1, task_2, task_3, task_4).await;
        let user = match user_res {
            Ok(user) => user,
            Err(_) => {
                return Err(ONDCBuyerError::BuyerInternalServerError { path: None });
            }
        };

        let product_map = match product_map_res {
            Ok(product_map) => product_map,
            Err(_) => {
                return Err(ONDCBuyerError::BuyerInternalServerError { path: None });
            }
        };
        let seller_info_map = match seller_info_map_res {
            Ok(seller_info_map) => seller_info_map,
            Err(_) => {
                return Err(ONDCBuyerError::BuyerInternalServerError { path: None });
            }
        };
        let seller_location_map = match seller_location_map_res {
            Ok(seller_location_map) => seller_location_map,
            Err(_) => {
                return Err(ONDCBuyerError::BuyerInternalServerError { path: None });
            }
        };

        initialize_order_on_select(
            &mut transaction,
            &body,
            &user,
            &business_account,
            &ondc_select_req,
            &chat_client,
            &product_map,
            &seller_info_map,
            &seller_location_map,
        )
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
        if is_rfq {
            send_rfq_accept_chat(&chat_client, &body, &product_map)
                .await
                .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
        }
    } else if let Some(error) = body.error {
        send_rfq_reject_chat(
            &chat_client,
            &error.message,
            body.context.transaction_id,
            &body.message.order.provider.id,
        )
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
    }

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let _ = websocket_srv
        .send_msg(ws_params_obj, WebSocketActionType::Select, ws_json, None)
        .await;
    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On Init Payload", skip(pool), fields())]
pub async fn on_init(
    pool: web::Data<PgPool>,
    body: ONDCOnInitRequest,
    websocket_srv: web::Data<WebSocketClient>,
    chat_client: web::Data<ChatClient>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let task_1 = fetch_ondc_order_request(
        &pool,
        body.context.transaction_id,
        body.context.message_id,
        &ONDCActionType::Init,
    );
    let task_2 = fetch_order_by_id(&pool, body.context.transaction_id);

    let (order_request_model_opt, commerce_data_opt) = match tokio::try_join!(task_1, task_2) {
        Ok((order_request_model, commerce_data_opt)) => (order_request_model, commerce_data_opt),
        Err(_) => {
            return Err(ONDCBuyerError::BuyerInternalServerError { path: None });
        }
    };
    let order_request_model =
        order_request_model_opt.ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    let commerce_data =
        commerce_data_opt.ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    // .await
    // .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
    // .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
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
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    initialize_order_on_init(&mut transaction, &body)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    if commerce_data.record_type.is_purchase_order() {
        send_rfq_init_chat(&chat_client, body.context.transaction_id, &commerce_data)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    }

    let _ = websocket_srv
        .send_msg(ws_params_obj, WebSocketActionType::Init, ws_json, None)
        .await;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On confirm Payload", skip(pool), fields())]
pub async fn on_confirm(
    pool: web::Data<PgPool>,
    body: ONDCOnConfirmRequest,
    websocket_srv: web::Data<WebSocketClient>,
    chat_client: web::Data<ChatClient>,
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
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    initialize_order_on_confirm(&mut transaction, &body, &order)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    if order.record_type.is_purchase_order() {
        send_rfq_confirmed_chat(&chat_client, body.context.transaction_id, &order)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    }

    let _ = websocket_srv
        .send_msg(ws_params_obj, WebSocketActionType::Confirm, ws_json, None)
        .await;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On status Payload", skip(pool), fields())]
pub async fn on_status(
    pool: web::Data<PgPool>,
    body: ONDCOnStatusRequest,
    websocket_srv: web::Data<WebSocketClient>,
    chat_client: web::Data<ChatClient>,
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
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    if order.record_type.is_purchase_order() {
        let proforma_present = body.message.order.documents.is_some();
        send_rfq_status_chat(
            &chat_client,
            body.context.transaction_id,
            body.message
                .order
                .state
                .get_commerce_status(&order.record_type, Some(proforma_present)),
            &body.message.order.fulfillments,
            body.error.as_ref().map(|e| e.message.clone()),
            &order,
        )
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    }
    initialize_order_on_status(&mut transaction, &body, &order)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let request_type = if order_request_model.is_some() {
        ONDCRequestType::Solicted
    } else {
        ONDCRequestType::UnSolicted
    };
    let ws_obj = WSStatus {
        transaction_id: body.context.transaction_id,
        message_id: body.context.message_id,
        error: body
            .error
            .as_ref()
            .map_or_else(|| None, |s| Some(s.message.to_owned())),
        request_type,
    };

    let ws_json = serde_json::to_value(ws_obj).unwrap();
    let ws_params_obj = get_ondc_order_param_from_commerce(&order);
    let _ = websocket_srv
        .send_msg(ws_params_obj, WebSocketActionType::Status, ws_json, None)
        .await;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On cancel Payload", skip(pool), fields())]
pub async fn on_cancel(
    pool: web::Data<PgPool>,
    body: ONDCOnCancelRequest,
    websocket_srv: web::Data<WebSocketClient>,
    chat_client: web::Data<ChatClient>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let task1 = fetch_ondc_order_request(
        &pool,
        body.context.transaction_id,
        body.context.message_id,
        &ONDCActionType::Cancel,
    );

    let task2 = fetch_order_by_id(&pool, body.context.transaction_id);
    let (res1, res2) = futures::future::join(task1, task2).await;
    let order_request_model =
        res1.map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let order = res2
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
        .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    if order.record_type.is_purchase_order() {
        send_rfq_cancel_chat(&chat_client, body.context.transaction_id, &order)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    }
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    initialize_order_on_cancel(&mut transaction, &body, &order)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    if let Some(order_request_model) = order_request_model {
        let ws_obj = WSCancel {
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
            .send_msg(ws_params_obj, WebSocketActionType::Cancel, ws_json, None)
            .await;
    }
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    Ok(web::Json(ONDCResponse::successful_response(None)))
}

#[tracing::instrument(name = "ONDC On update Payload", skip(pool), fields())]
pub async fn on_update(
    pool: web::Data<PgPool>,
    body: ONDCOnUpdateRequest,
    websocket_srv: web::Data<WebSocketClient>,
    chat_client: web::Data<ChatClient>,
) -> Result<web::Json<ONDCResponse<ONDCBuyerErrorCode>>, ONDCBuyerError> {
    let task1 = fetch_ondc_order_request(
        &pool,
        body.context.transaction_id,
        body.context.message_id,
        &ONDCActionType::Update,
    );

    let task2 = fetch_order_by_id(&pool, body.context.transaction_id);
    let (res1, res2) = futures::future::join(task1, task2).await;
    let order_request_model =
        res1.map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    let order = res2
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?
        .ok_or(ONDCBuyerError::BuyerResponseSequenceError { path: None })?;
    if order.record_type.is_purchase_order() {
        send_rfq_update_chat(&chat_client, body.context.transaction_id, &order)
            .await
            .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    }
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    initialize_order_on_update(&mut transaction, &body, &order)
        .await
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;
    if let Some(order_request_model) = order_request_model {
        let ws_obj = WSUpdate {
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
            .send_msg(ws_params_obj, WebSocketActionType::Update, ws_json, None)
            .await;
    }
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|_| ONDCBuyerError::BuyerInternalServerError { path: None })?;

    Ok(web::Json(ONDCResponse::successful_response(None)))
}
