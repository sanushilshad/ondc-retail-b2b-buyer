use actix_http::StatusCode;
use actix_web::{web, HttpResponse};
use utoipa::TupleUnit;
// use anyhow::Context;
use super::schemas::{
    MinimalItemData, NetworkParticipantListReq, ProductList, ProductSearchRequest, WSSearch,
    WSSearchBPP,
};
use super::utils::{get_network_participant_from_es, save_search_request};
use crate::configuration::ONDCConfig;
use crate::elastic_search_client::ElasticSearchClient;
use crate::errors::GenericError;
use crate::routes::ondc::utils::{get_ondc_search_payload, send_ondc_payload};
use crate::routes::ondc::ONDCActionType;
use crate::user_client::{BusinessAccount, UserAccount};
use crate::utils::{create_authorization_header, get_np_detail};

use crate::schemas::{GenericResponse, ONDCNetworkType, RequestMetaData};
use sqlx::PgPool;
#[utoipa::path(
    post,
    path = "/product/search/realtime",
    tag = "Product",
    description="This API generates the ONDC search request based on user input.",
    summary= "Realtime Product Search Request",
    request_body(content = ProductSearchRequest, description = "Request Body"),
    responses(
        (status=202, description= "Realtime Product Search", body= GenericResponse<TupleUnit>),
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
) -> Result<HttpResponse, GenericError> {
    let np_detail = match get_np_detail(
        &pool,
        &business_account.subscriber_id,
        &ONDCNetworkType::Bap,
    )
    .await
    {
        Ok(Some(np_detail)) => np_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                &business_account.subscriber_id,
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
    Ok(HttpResponse::Accepted().json(GenericResponse::success(
        "Successfully Sent Product Search Request",
        StatusCode::ACCEPTED,
        Some(()),
    )))
}

#[utoipa::path(
    post,
    path = "/product/search/cache/read",
    tag = "Product",
    description="This API searches product in cache.",
    summary= "Cached Product Search Request",
    request_body(content = ProductSearchRequest, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<WSSearch>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>)
    )
)]
#[tracing::instrument(name = "Cached Product Search", skip(pool), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn cached_product_read(
    body: ProductSearchRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCConfig>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let _np_detail = match get_np_detail(
        &pool,
        &business_account.subscriber_id,
        &ONDCNetworkType::Bap,
    )
    .await
    {
        Ok(Some(np_detail)) => np_detail,
        Ok(None) => {
            return Err(GenericError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                &business_account.subscriber_id,
            )))
        }
        Err(e) => {
            return Err(GenericError::DatabaseError(
                "Something went wrong while fetching NP credentials".to_string(),
                e,
            ));
        }
    };

    Ok(web::Json(GenericResponse::success(
        "Successfully Fetched Product Detail",
        StatusCode::OK,
        Some(()),
    )))
}

#[utoipa::path(
    post,
    path = "/product/search/cache/list",
    tag = "Product",
    description="This API is used for listing products with minimal data.",
    summary= "Product Minimal Data list API",
    request_body(content = ProductList, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<Vec<MinimalItemData>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>)
    )
)]
#[tracing::instrument(name = "Cache Product List API", skip(_pool), fields())]
#[allow(unreachable_code)]
pub async fn cached_product_list(
    body: ProductList,
    _pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<MinimalItemData>>>, GenericError> {
    let _data = MinimalItemData {
        bpp: todo!(),
        providers: todo!(),
    };
    Ok(web::Json(GenericResponse::success(
        "Successfully Fetched Product list",
        StatusCode::OK,
        Some(vec![_data]),
    )))
}

#[utoipa::path(
    post,
    path = "/product/network_participant/",
    tag = "Product",
    description="This API is used for listing all Network participants.",
    summary= "Network Participant list API",
    request_body(content = NetworkParticipantListReq, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<Vec<WSSearchBPP>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>)
    )
)]
#[tracing::instrument(name = "Cache Network Participant List API", skip(), fields())]
#[allow(unreachable_code)]
pub async fn cached_network_participant_list(
    body: NetworkParticipantListReq,
    user_account: UserAccount,
    es_client: web::Data<ElasticSearchClient>,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<WSSearchBPP>>>, GenericError> {
    let data = get_network_participant_from_es(&es_client, body)
        .await
        .map_err(GenericError::UnexpectedError)?;
    let final_data = data.map_or_else(Vec::new, |models| {
        models.into_iter().map(|a| a.get_ws_bpp()).collect()
    });

    Ok(web::Json(GenericResponse::success(
        "Successfully Fetched network participants.",
        StatusCode::OK,
        Some(final_data),
    )))
}
