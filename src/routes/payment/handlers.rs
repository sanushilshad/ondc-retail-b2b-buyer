use super::schemas::{
    CreatePaymentOrderRequest, PaymentNotificationRequest, PaymentOrderData, WSPayment,
};
use super::utils::{
    get_commerce_payments_with_lock, get_payment_order_id, get_payment_ws_params,
    update_payment_status, validate_order_for_payment,
};
use crate::routes::order::schemas::PaymentStatus;
use crate::routes::order::utils::fetch_minimal_commerce_data_model;
use crate::schemas::RequestMetaData;
// use crate::routes::order::utils::update_order_update_field;
use crate::user_client::UserClient;
use crate::websocket_client::{WebSocketActionType, WebSocketClient};
use crate::{
    errors::GenericError,
    payment_client::PaymentClient,
    schemas::GenericResponse,
    user_client::{AllowedPermission, BusinessAccount, PermissionType, UserAccount},
};
use actix_web::web;

use anyhow::Context;
use sqlx::PgPool;
use utoipa::TupleUnit;
#[utoipa::path(
    post,
    path = "/payment/order/create",
    tag = "Payment",
    description="This API Creates a Payment order for a transaction.",
    summary= "Payment Order Creation Request",
    request_body(content = CreatePaymentOrderRequest, description = "Request Body"),
    responses(
        (status=200, description= "Order Update Response", body= GenericResponse<PaymentOrderData>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>),
    )
)]
#[tracing::instrument(name = "payment order creation", skip(pool), fields(body.transaction_id))]
pub async fn payment_order_creation(
    body: CreatePaymentOrderRequest,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    allowed_permission: AllowedPermission,
    payment_client: web::Data<PaymentClient>,
    user_client: web::Data<UserClient>,
) -> Result<web::Json<GenericResponse<PaymentOrderData>>, GenericError> {
    let order = fetch_minimal_commerce_data_model(&pool, body.transaction_id)
        .await
        .map_err(|e| GenericError::DatabaseError("Failed to fetch order".to_string(), e))?;

    if !allowed_permission.validate_commerce_self(
        order.created_by,
        order.buyer_id,
        PermissionType::CreateOrderSelf,
    ) {
        return Err(GenericError::InsufficientPrevilegeError(
            "You do not have sufficent preveliege to read the order".to_owned(),
        ));
    }

    validate_order_for_payment(&order).map_err(|e| GenericError::ValidationError(e.to_string()))?;
    let order_data = get_payment_order_id(
        &pool,
        &payment_client,
        &user_client,
        &order,
        &business_account,
        &user_account,
    )
    .await?;
    Ok(web::Json(GenericResponse::success(
        "Successfully created orders",
        Some(order_data),
    )))
}

#[utoipa::path(
    post,
    path = "/payment/notification",
    tag = "Payment",
    description="This API Is used by the payment service to send the status of payment.",
    summary= "Payment Notification Service",
    request_body(content = PaymentNotificationRequest, description = "Request Body"),
    responses(
        (status=200, description= "Order Update Response", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>),
    )
)]
#[tracing::instrument(name = "payment status notification", skip(pool), fields(body.transaction_id))]
pub async fn payment_notification(
    body: PaymentNotificationRequest,
    pool: web::Data<PgPool>,
    websocket_srv: web::Data<WebSocketClient>,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let order = fetch_minimal_commerce_data_model(&pool, body.transaction_id)
        .await
        .map_err(|e| GenericError::DatabaseError("Failed to fetch order".to_string(), e))?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    let payment_data = get_commerce_payments_with_lock(&mut transaction, order.id)
        .await
        .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    if payment_data.payment_status != Some(PaymentStatus::Paid) {
        update_payment_status(
            &mut transaction,
            body.transaction_id,
            &meta_data.device_id,
            body.status.payment_status(),
            &body.payment_id,
            &body.payment_order_id,
        )
        .await
        .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
        let ws_obj = WSPayment {
            message: format!(
                "Payment for Order {} is: {}",
                order.urn,
                body.status.payment_status()
            ),
        };
        let ws_json = serde_json::to_value(ws_obj).unwrap();
        websocket_srv
            .send_msg(
                get_payment_ws_params(&order),
                WebSocketActionType::OrderPayment,
                ws_json,
                None,
            )
            .await
            .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    }

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;

    Ok(web::Json(GenericResponse::success(
        "Successfully recieved notification",
        Some(()),
    )))
}
