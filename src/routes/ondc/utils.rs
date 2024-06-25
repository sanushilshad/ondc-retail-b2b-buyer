use crate::schemas::{NetworkCall, NetworkError, NetworkResponse, ONDCNetworkType};
use chrono::Utc;
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

use super::{LookupData, LookupRequest, ONDCDomain, OndcUrl};
#[tracing::instrument(name = "Call lookup API", skip())]
pub async fn call_lookup_api(
    payload: &str,
    lookup_url: &str,
) -> Result<NetworkResponse, NetworkError> {
    let client = Client::new();
    let network_call = NetworkCall { client };
    network_call
        .async_post_call(lookup_url, Some(payload), None)
        .await
}
#[tracing::instrument(name = "Get lookup for subscriber", skip())]
pub async fn get_lookup_for_subscriber(
    subscriber_id: &str,
    np_type: ONDCNetworkType,
    domain: ONDCDomain,
    lookup_uri: &str,
) -> Result<LookupData, anyhow::Error> {
    let look_up_request = LookupRequest {
        subscriber_id: subscriber_id.to_string(),
        domain,
        r#type: np_type,
    };
    let request_str = serde_json::to_string(&look_up_request).unwrap();
    let url = format!("{} {}", lookup_uri, OndcUrl::LookUp);
    let response = call_lookup_api(&request_str, &url).await?;
    let my_struct_instance: LookupData = serde_json::from_str(response.get_body())?;
    Ok(my_struct_instance)
}

#[tracing::instrument(name = "Get lookup data from db", skip(pool))]
pub async fn get_lookup_data_from_db(
    pool: &PgPool,
    subscriber_id: &str,
    np_type: ONDCNetworkType,
    domain: ONDCDomain,
) -> Result<Option<LookupData>, anyhow::Error> {
    let row = sqlx::query_as!(
        LookupData,
        r#"SELECT br_id,subscriber_id, signing_public_key, subscriber_url, encr_public_key, unique_key_id, domain as "domain!:ONDCDomain", type as "type!: ONDCNetworkType"  FROM network_participant
        WHERE subscriber_id = $1 AND type = $2 AND domain = $3
        "#,
        subscriber_id,
        np_type as ONDCNetworkType,
        domain.to_string()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

#[tracing::instrument(name = "Save lookup data to db", skip(pool))]
pub async fn save_lookup_data_to_db(
    pool: &PgPool,
    lookup_req: LookupRequest,
    data: LookupData,
) -> Result<(), anyhow::Error> {
    let uuid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO network_participant (id, subscriber_id, br_id, subscriber_url, signing_public_key, domain, encr_public_key, type, unique_key_id, created_on)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        &uuid,
        &data.subscriber_id,
        &data.br_id,
        &data.subscriber_url,
        &data.signing_public_key,
        &data.domain.to_string(),
        &data.encr_public_key,
        &lookup_req.r#type as &ONDCNetworkType,
        &data.unique_key_id,
        Utc::now(),
    )
    .execute(pool).await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving look up data")
    })?;
    Ok(())
}
