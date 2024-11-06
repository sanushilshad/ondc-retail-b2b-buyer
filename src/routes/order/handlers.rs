use std::collections::HashSet;

use actix_web::web;
use utoipa::TupleUnit;
// use anyhow::Context;
use crate::configuration::ONDCSetting;
use crate::errors::GenericError;
use crate::routes::ondc::utils::{
    get_lookup_data_from_db, get_ondc_cancel_payload, get_ondc_seller_location_info_mapping,
    get_ondc_status_payload,
};
use crate::routes::ondc::utils::{
    get_ondc_confirm_payload, get_ondc_init_payload, get_ondc_select_payload, send_ondc_payload,
};
use crate::routes::ondc::{ONDCActionType, ONDCDomain};
use crate::user_client::{BusinessAccount, UserAccount};
use crate::utils::{create_authorization_header, get_np_detail};

use crate::schemas::{GenericResponse, ONDCNPType, ONDCNetworkType, RequestMetaData};
use sqlx::PgPool;

use super::schemas::{
    OrderCancelRequest, OrderConfirmRequest, OrderInitRequest, OrderSelectRequest,
    OrderStatusRequest, OrderType,
};
use super::utils::{fetch_order_by_id, initialize_order_select, save_ondc_order_request};

#[utoipa::path(
    post,
    path = "/order/select",
    tag = "Order",
    description="This API generates the ONDC select request based on user input.",
    summary= "Order Select Request",
    request_body(content = OrderSelectRequest, description = "Request Body"),
    responses(
        (status=200, description= "Order Select Response", body= GenericResponse<TupleUnit>),
    )
)]
#[tracing::instrument(name = "order select", skip(pool), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn order_select(
    body: OrderSelectRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCSetting>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let task1 = get_np_detail(&pool, &meta_data.domain_uri, &ONDCNPType::Buyer);
    let ondc_domain = ONDCDomain::get_ondc_domain(&body.domain_category_code);
    let task2 = get_lookup_data_from_db(&pool, &body.bpp_id, &ONDCNetworkType::Bpp, &ondc_domain);
    let location_id_list: Vec<String> = body
        .items
        .iter()
        .flat_map(|item| item.location_ids.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    println!("a{}", &meta_data.domain_uri);
    let task3 = get_ondc_seller_location_info_mapping(
        &pool,
        &body.bpp_id,
        &body.provider_id,
        &location_id_list,
    );
    let (bap_detail_res, bpp_detail_res, seller_location_info_mapping_res) =
        futures::future::join3(task1, task2, task3).await;
    let bap_detail = match bap_detail_res {
        Ok(Some(bap_detail)) => bap_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                meta_data.domain_uri
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching NP credentials".to_string(),
                e,
            ));
        }
    };
    let bpp_detail = match bpp_detail_res {
        Ok(Some(np_detail)) => np_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a Valid BPP Id",
                &body.bpp_id
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching BPP credentials".to_string(),
                e,
            ));
        }
    };
    let seller_location_info_mapping = match seller_location_info_mapping_res {
        Ok(location_info_mapping) if !location_info_mapping.is_empty() => location_info_mapping,
        Ok(_) => {
            return Err(GenericError::ValidationError(
                "Location mapping is Invalid".to_string(),
            ));
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching locations".to_string(),
                e,
            ));
        }
    };

    let ondc_select_payload = get_ondc_select_payload(
        &user_account,
        &business_account,
        &body,
        &bap_detail,
        &bpp_detail,
        &seller_location_info_mapping,
    )?;

    let ondc_select_payload_str = serde_json::to_string(&ondc_select_payload).map_err(|e| {
        GenericError::SerializationError(format!("Failed to serialize ONDC select payload: {}", e))
    })?;
    let header = create_authorization_header(&ondc_select_payload_str, &bap_detail, None, None)?;
    let select_json_obj = serde_json::to_value(&ondc_select_payload)?;
    let task_4 = save_ondc_order_request(
        &pool,
        &user_account,
        &business_account,
        &meta_data,
        &select_json_obj,
        body.transaction_id,
        body.message_id,
        ONDCActionType::Select,
    );
    let task_5 = send_ondc_payload(
        &bpp_detail.subscriber_url,
        &ondc_select_payload_str,
        &header,
        ONDCActionType::Select,
    );
    futures::future::join(task_4, task_5).await.1?;
    if body.order_type == OrderType::PurchaseOrder {
        if let Err(e) = initialize_order_select(
            &pool,
            &user_account,
            &business_account,
            &body,
            &bap_detail,
            &bpp_detail,
            &seller_location_info_mapping,
        )
        .await
        {
            return Err(GenericError::DatabaseError(
                "Something went wrong while commiting order to database".to_string(),
                e,
            ));
        };
    }

    Ok(web::Json(GenericResponse::success(
        "Successfully send select request",
        Some(()),
    )))
}

#[utoipa::path(
    post,
    path = "/order/init",
    tag = "Order",
    description="This API generates the ONDC init request based on user input.",
    summary= "Order Init Request",
    request_body(content = OrderInitRequest, description = "Request Body"),
    responses(
        (status=200, description= "Order init Response", body= GenericResponse<TupleUnit>),
    )
)]
#[tracing::instrument(name = "order init", skip(pool), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn order_init(
    body: OrderInitRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCSetting>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let order = match fetch_order_by_id(&pool, body.transaction_id).await {
        Ok(Some(order_detail)) => order_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not found in datbase",
                &body.transaction_id
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching Order detail".to_string(),
                e,
            ));
        }
    };

    let bap_detail = match get_np_detail(&pool, &meta_data.domain_uri, &ONDCNPType::Buyer).await {
        Ok(Some(bap_detail)) => bap_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                meta_data.domain_uri
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching NP credentials".to_string(),
                e,
            ));
        }
    };

    let ondc_init_payload = get_ondc_init_payload(&user_account, &business_account, &order, &body)?;

    let ondc_init_payload_str = serde_json::to_string(&ondc_init_payload).map_err(|e| {
        GenericError::SerializationError(format!("Failed to serialize ONDC init payload: {}", e))
    })?;
    let header = create_authorization_header(&ondc_init_payload_str, &bap_detail, None, None)?;
    let init_json_obj = serde_json::to_value(&ondc_init_payload)?;
    let task_3 = save_ondc_order_request(
        &pool,
        &user_account,
        &business_account,
        &meta_data,
        &init_json_obj,
        body.transaction_id,
        body.message_id,
        ONDCActionType::Init,
    );
    let task_4 = send_ondc_payload(
        &order.bpp.uri,
        &ondc_init_payload_str,
        &header,
        ONDCActionType::Init,
    );
    futures::future::join(task_3, task_4).await.1?;

    Ok(web::Json(GenericResponse::success(
        "Successfully send init request",
        Some(()),
    )))
}

#[utoipa::path(
    post,
    path = "/order/confirm",
    tag = "Order",
    description="This API generates the ONDC confirm request based on user input.",
    summary= "Order confirm Request",
    request_body(content = OrderConfirmRequest, description = "Request Body"),
    responses(
        (status=200, description= "Order confirm Response", body= GenericResponse<TupleUnit>),
    )
)]
#[tracing::instrument(name = "order confirm", skip(pool), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn order_confirm(
    body: OrderConfirmRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCSetting>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let order = match fetch_order_by_id(&pool, body.transaction_id).await {
        Ok(Some(order_detail)) => order_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not found in datbase",
                &body.transaction_id
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching Order detail".to_string(),
                e,
            ));
        }
    };

    let bap_detail = match get_np_detail(&pool, &meta_data.domain_uri, &ONDCNPType::Buyer).await {
        Ok(Some(bap_detail)) => bap_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                meta_data.domain_uri
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching NP credentials".to_string(),
                e,
            ));
        }
    };

    let ondc_confirm_payload =
        get_ondc_confirm_payload(&user_account, &business_account, &order, &body, &bap_detail)?;

    let ondc_confirm_payload_str = serde_json::to_string(&ondc_confirm_payload).map_err(|e| {
        GenericError::SerializationError(format!("Failed to serialize ONDC init payload: {}", e))
    })?;
    let header = create_authorization_header(&ondc_confirm_payload_str, &bap_detail, None, None)?;
    let confirm_json_obj = serde_json::to_value(&ondc_confirm_payload)?;
    let task_3 = save_ondc_order_request(
        &pool,
        &user_account,
        &business_account,
        &meta_data,
        &confirm_json_obj,
        body.transaction_id,
        body.message_id,
        ONDCActionType::Confirm,
    );
    let task_4 = send_ondc_payload(
        &order.bpp.uri,
        &ondc_confirm_payload_str,
        &header,
        ONDCActionType::Confirm,
    );
    futures::future::join(task_3, task_4).await.1?;

    Ok(web::Json(GenericResponse::success(
        "Successfully send confirm request",
        Some(()),
    )))
}

#[utoipa::path(
    post,
    path = "/order/status",
    tag = "Order",
    description="This API generates the ONDC status request based on user input.",
    summary= "Order Status Request",
    request_body(content = OrderStatusRequest, description = "Request Body"),
    responses(
        (status=200, description= "Order Status Response", body= GenericResponse<TupleUnit>),
    )
)]
#[tracing::instrument(name = "order status", skip(pool), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn order_status(
    body: OrderStatusRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCSetting>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let order = match fetch_order_by_id(&pool, body.transaction_id).await {
        Ok(Some(order_detail)) => order_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not found in datbase",
                &body.transaction_id
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching Order detail".to_string(),
                e,
            ));
        }
    };

    let bap_detail = match get_np_detail(&pool, &meta_data.domain_uri, &ONDCNPType::Buyer).await {
        Ok(Some(bap_detail)) => bap_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                meta_data.domain_uri
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching NP credentials".to_string(),
                e,
            ));
        }
    };

    let ondc_status_payload = get_ondc_status_payload(&order, &body)?;
    let confirm_json_obj = serde_json::to_value(&ondc_status_payload)?;
    let ondc_status_payload_str = serde_json::to_string(&ondc_status_payload).map_err(|e| {
        GenericError::SerializationError(format!("Failed to serialize ONDC status payload: {}", e))
    })?;
    let header = create_authorization_header(&ondc_status_payload_str, &bap_detail, None, None)?;
    let task_3 = save_ondc_order_request(
        &pool,
        &user_account,
        &business_account,
        &meta_data,
        &confirm_json_obj,
        body.transaction_id,
        body.message_id,
        ONDCActionType::Status,
    );
    let task_4 = send_ondc_payload(
        &order.bpp.uri,
        &ondc_status_payload_str,
        &header,
        ONDCActionType::Status,
    );
    futures::future::join(task_3, task_4).await.1?;

    Ok(web::Json(GenericResponse::success(
        "Successfully send status request",
        Some(()),
    )))
}

#[tracing::instrument(name = "order cancel", skip(pool), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn order_cancel(
    body: OrderCancelRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCSetting>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let order = match fetch_order_by_id(&pool, body.transaction_id).await {
        Ok(Some(order_detail)) => order_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not found in datbase",
                &body.transaction_id
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching Order detail".to_string(),
                e,
            ));
        }
    };

    let bap_detail = match get_np_detail(&pool, &meta_data.domain_uri, &ONDCNPType::Buyer).await {
        Ok(Some(bap_detail)) => bap_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                meta_data.domain_uri
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching NP credentials".to_string(),
                e,
            ));
        }
    };

    let ondc_status_payload = get_ondc_cancel_payload(&order, &body)?;
    let confirm_json_obj = serde_json::to_value(&ondc_status_payload)?;
    let ondc_status_payload_str = serde_json::to_string(&ondc_status_payload).map_err(|e| {
        GenericError::SerializationError(format!("Failed to serialize ONDC status payload: {}", e))
    })?;
    let header = create_authorization_header(&ondc_status_payload_str, &bap_detail, None, None)?;
    let task_3 = save_ondc_order_request(
        &pool,
        &user_account,
        &business_account,
        &meta_data,
        &confirm_json_obj,
        body.transaction_id,
        body.message_id,
        ONDCActionType::Cancel,
    );
    let task_4 = send_ondc_payload(
        &order.bpp.uri,
        &ondc_status_payload_str,
        &header,
        ONDCActionType::Cancel,
    );
    futures::future::join(task_3, task_4).await.1?;

    Ok(web::Json(GenericResponse::success(
        "Successfully send status request",
        Some(()),
    )))
}
