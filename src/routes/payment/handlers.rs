use super::schemas::PaymentOrderCreationRequest;
use crate::{
    errors::GenericError,
    routes::order::{schemas::OrderListFilter, utils::get_order_list},
    schemas::GenericResponse,
    user_client::{AllowedPermission, BusinessAccount, UserAccount},
};

use actix_web::web;
use sqlx::PgPool;
use utoipa::TupleUnit;
#[utoipa::path(
    post,
    path = "/payment/order/create",
    tag = "Payment",
    description="This API Creates a Payment order for a transaction.",
    summary= "Payment Order Creation Request",
    request_body(content = PaymentOrderCreationRequest, description = "Request Body"),
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
#[tracing::instrument(name = "payment order creation", skip(pool), fields(body.transaction_id))]
pub async fn payment_order_creation(
    body: PaymentOrderCreationRequest,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    allowed_permission: AllowedPermission,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    // let list_filter = OrderListFilter::new(body, allowed_permission);
    // let data = get_order_list(&pool, list_filter).await.map_err(|e| {
    //     tracing::error!("Database error while fetching order list: {:?}", e);
    //     GenericError::DatabaseError("Failed to fetch order list".to_string(), e)
    // })?;
    Ok(web::Json(GenericResponse::success(
        "Successfully created orders",
        Some(()),
    )))
}
