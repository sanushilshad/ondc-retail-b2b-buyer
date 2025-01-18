use actix_web::web;
use utoipa::TupleUnit;
// use anyhow::Context;
use super::schemas::ProductSearchRequest;
use super::utils::save_search_request;
use crate::configuration::ONDCConfig;
use crate::errors::GenericError;
use crate::routes::ondc::utils::{get_ondc_search_payload, send_ondc_payload};
use crate::routes::ondc::ONDCActionType;
use crate::user_client::{BusinessAccount, UserAccount};
use crate::utils::{create_authorization_header, get_np_detail};

use crate::schemas::{GenericResponse, ONDCNetworkType, RequestMetaData};
use sqlx::PgPool;
#[utoipa::path(
    post,
    path = "/product/realtime/search",
    tag = "Product",
    description="This API generates the ONDC search request based on user input.",
    summary= "Realtime Product Search",
    request_body(content = ProductSearchRequest, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>)
    )
)]
#[tracing::instrument(name = "Product Search", skip(pool), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn realtime_product_search(
    body: ProductSearchRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCConfig>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let np_detail = match get_np_detail(&pool, &meta_data.domain_uri, &ONDCNetworkType::Bap).await {
        Ok(Some(np_detail)) => np_detail,
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
    let ondc_search_payload =
        get_ondc_search_payload(&user_account, &business_account, &body, &np_detail)?;
    let ondc_search_payload_str = serde_json::to_string(&ondc_search_payload).map_err(|e| {
        GenericError::SerializationError(format!("Failed to serialize ONDC search payload: {}", e))
    })?;
    let task1 = save_search_request(&pool, &user_account, &business_account, &meta_data, &body);
    let header = create_authorization_header(&ondc_search_payload_str, &np_detail, None, None)?;
    let task2 = send_ondc_payload(
        &ondc_obj.gateway_uri,
        &ondc_search_payload_str,
        &header,
        ONDCActionType::Search,
    );
    futures::future::join(task1, task2).await.1?;
    Ok(web::Json(GenericResponse::success(
        "Successfully Send Product Search Request",
        Some(()),
    )))
}
