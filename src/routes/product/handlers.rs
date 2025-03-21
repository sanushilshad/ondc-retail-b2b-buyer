use actix_http::StatusCode;
use actix_web::{web, HttpResponse};
use utoipa::TupleUnit;
// use anyhow::Context;
use super::schemas::{
    AutoCompleteItemRequest, AutoCompleteItemResponseData, ItemCacheResponseData,
    NetworkParticipantListReq, NetworkParticipantListResponse, ProductCacheSearchRequest,
    ProductSearchRequest, ProviderFetchReq, ProviderListResponse,
};
use super::utils::{
    get_auto_complete_product_data, get_full_item_data_from_es, get_network_participant_from_es,
    get_provider_from_es, insert_subscribed_search_location, save_search_request,
};
use crate::configuration::ONDCConfig;
use crate::elastic_search_client::ElasticSearchClient;
use crate::errors::GenericError;
use crate::kafka_client::KafkaClient;
use crate::routes::ondc::utils::{get_ondc_search_payload, send_ondc_payload};
use crate::routes::ondc::ONDCActionType;
use crate::user_client::{BusinessAccount, UserAccount};
use crate::utils::{create_authorization_header, get_np_detail};

use crate::schemas::{GenericResponse, ONDCNetworkType, RequestMetaData, StartUpMap};
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
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(name = "Product Search", skip(pool, kafka_client), fields(transaction_id=body.transaction_id.to_string()))]
pub async fn realtime_product_search(
    body: ProductSearchRequest,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCConfig>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
    maps: web::Data<StartUpMap>,
    kafka_client: web::Data<KafkaClient>,
) -> Result<HttpResponse, GenericError> {
    let np_detail = match get_np_detail(
        &pool,
        &maps,
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
    let task_1 = save_search_request(&pool, &user_account, &business_account, &meta_data, &body);
    let header = create_authorization_header(&ondc_search_payload_str, &np_detail, None, None)?;
    let task_2 = send_ondc_payload(
        &ondc_obj.gateway_uri,
        &ondc_search_payload_str,
        &header,
        &ONDCActionType::Search,
        &kafka_client,
        &np_detail.subscriber_id,
        body.transaction_id,
        ondc_obj.observability.is_enabled,
    );
    let task_3 = insert_subscribed_search_location(
        &pool,
        &body.city_code,
        &body.country_code,
        &body.domain_category_code,
    );
    futures::future::join3(task_1, task_2, task_3).await.1?;
    Ok(HttpResponse::Accepted().json(GenericResponse::success(
        "Successfully Sent Product Search Request",
        StatusCode::ACCEPTED,
        Some(()),
    )))
}

#[utoipa::path(
    post,
    path = "/product/network_participant/fetch",
    tag = "Product",
    description="This API is used for listing all Network participants.",
    summary= "Network Participant list API",
    request_body(content = NetworkParticipantListReq, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<NetworkParticipantListResponse>),
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
) -> Result<web::Json<GenericResponse<NetworkParticipantListResponse>>, GenericError> {
    let data = get_network_participant_from_es(&es_client, body)
        .await
        .map_err(GenericError::UnexpectedError)?;

    Ok(web::Json(GenericResponse::success(
        "Successfully Fetched network participants.",
        StatusCode::OK,
        data,
    )))
}

#[utoipa::path(
    post,
    path = "/product/provider/fetch",
    tag = "Product",
    description="This API is used for listing all providers.",
    summary= "Provider list API",
    request_body(content = ProviderFetchReq, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<ProviderListResponse>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>)
    )
)]
#[tracing::instrument(name = "Cache Provider List API", skip(), fields())]
#[allow(unreachable_code)]
pub async fn cached_provider_list(
    body: ProviderFetchReq,
    user_account: UserAccount,
    es_client: web::Data<ElasticSearchClient>,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<ProviderListResponse>>, GenericError> {
    let data = get_provider_from_es(&es_client, body)
        .await
        .map_err(GenericError::UnexpectedError)?;

    Ok(web::Json(GenericResponse::success(
        "Successfully Fetched providers.",
        StatusCode::OK,
        data,
    )))
}

#[utoipa::path(
    post,
    path = "/product/autocomplete",
    tag = "Product",
    description="This API is used for listing products with minimal data.",
    summary= "Product AutoComplete API",
    request_body(content = AutoCompleteItemRequest, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<Vec<AutoCompleteItemResponseData>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>)
    )
)]
#[tracing::instrument(name = "Product AutoComplete API", skip(), fields())]
#[allow(unreachable_code)]
pub async fn product_autocomplete(
    body: AutoCompleteItemRequest,
    es_client: web::Data<ElasticSearchClient>,
) -> Result<web::Json<GenericResponse<AutoCompleteItemResponseData>>, GenericError> {
    let data = get_auto_complete_product_data(&es_client, &body)
        .await
        .map_err(GenericError::UnexpectedError)?;

    Ok(web::Json(GenericResponse::success(
        "Successfully Fetched autocomplete data",
        StatusCode::OK,
        Some(data),
    )))
}

#[utoipa::path(
    post,
    path = "/product/search/cache",
    tag = "Product",
    description="This API searches product in cache.",
    summary= "Cached Product Search Request",
    request_body(content = ProductCacheSearchRequest, description = "Request Body"),
    responses(
        (status=200, description= "Realtime Product Search", body= GenericResponse<ItemCacheResponseData>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>),
	    (status=501, description= "Not Implemented", body= GenericResponse<TupleUnit>)
    )
)]
#[tracing::instrument(name = "Cached Product Search", skip(), fields())]
pub async fn cached_product_req(
    body: ProductCacheSearchRequest,
    es_client: web::Data<ElasticSearchClient>,
) -> Result<web::Json<GenericResponse<ItemCacheResponseData>>, GenericError> {
    let data = get_full_item_data_from_es(&es_client, &body)
        .await
        .map_err(GenericError::UnexpectedError)?;
    Ok(web::Json(GenericResponse::success(
        "Successfully Fetched Product Detail",
        StatusCode::OK,
        data,
    )))
}
