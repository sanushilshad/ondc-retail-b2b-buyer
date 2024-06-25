use super::{LookupData, LookupRequest, ONDCDomain, OndcUrl};
use crate::schemas::{NetworkCall, ONDCNetworkType};
use chrono::Utc;
use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(name = "Call lookup API", skip())]
pub async fn call_lookup_api(
    payload: &str,
    lookup_url: &str,
) -> Result<Option<LookupData>, anyhow::Error> {
    let client = Client::new();
    let network_call = NetworkCall { client };
    let result = network_call
        .async_post_call_with_retry(lookup_url, Some(payload), None)
        .await?;
    match result {
        serde_json::Value::Array(data) => {
            if data.is_empty() {
                return Ok(None);
            }
            let lookup_data_value = data.get(0).expect("Expected non-empty array");
            let lookup_data: LookupData = serde_json::from_value(lookup_data_value.clone())?;
            Ok(Some(lookup_data))
        }
        _ => {
            return Err(anyhow::format_err!("Error while parsing looup"));
        }
    }
}
#[tracing::instrument(name = "Get lookup for subscriber", skip())]
pub async fn get_lookup_for_subscriber_by_api(
    subscriber_id: &str,
    np_type: &ONDCNetworkType,
    domain: &ONDCDomain,
    lookup_uri: &str,
) -> Result<Option<LookupData>, anyhow::Error> {
    let look_up_request = LookupRequest {
        subscriber_id: subscriber_id,
        domain,
        r#type: np_type,
    };
    let request_str = serde_json::to_string(&look_up_request).unwrap();
    let url = format!("{}{}", lookup_uri, OndcUrl::LookUp);
    let look_up_data = call_lookup_api(&request_str, &url).await?;
    Ok(look_up_data)
}

#[tracing::instrument(name = "Get lookup data from db", skip(pool))]
pub async fn get_lookup_data_from_db(
    pool: &PgPool,
    subscriber_id: &str,
    np_type: &ONDCNetworkType,
    domain: &ONDCDomain,
) -> Result<Option<LookupData>, anyhow::Error> {
    let row = sqlx::query_as!(
        LookupData,
        r#"SELECT br_id, subscriber_id, signing_public_key, subscriber_url, encr_public_key, uk_id, domain as "domain!: ONDCDomain", type as "type!: ONDCNetworkType"  FROM network_participant
        WHERE subscriber_id = $1 AND type = $2 AND domain = $3
        "#,
        subscriber_id,
        np_type as &ONDCNetworkType,
        domain.to_string()
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

#[tracing::instrument(name = "Save lookup data to db", skip(pool))]
pub async fn save_lookup_data_to_db(pool: &PgPool, data: &LookupData) -> Result<(), anyhow::Error> {
    let uuid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO network_participant (id, subscriber_id, br_id, subscriber_url, signing_public_key, domain, encr_public_key, type, uk_id, created_on)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        &uuid,
        &data.subscriber_id,
        &data.br_id,
        &data.subscriber_url,
        &data.signing_public_key,
        &data.domain.to_string(),
        &data.encr_public_key,
        &data.r#type as &ONDCNetworkType,
        &data.uk_id,
        Utc::now(),
    )
    .execute(pool).await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving look up data")
    })?;
    Ok(())
}

#[tracing::instrument(name = "Fetch lookup data", skip(pool))]
pub async fn fetch_lookup_data(
    pool: &PgPool,
    subscriber_id: &str,
    np_type: &ONDCNetworkType,
    domain: &ONDCDomain,
    lookup_uri: &str,
) -> Result<Option<LookupData>, anyhow::Error> {
    let look_up_data = get_lookup_data_from_db(pool, subscriber_id, np_type, domain).await?;
    if look_up_data.is_some() {
        return Ok(look_up_data);
    }

    let look_up_data_from_api =
        get_lookup_for_subscriber_by_api(subscriber_id, np_type, domain, lookup_uri).await?;

    if let Some(ref data) = look_up_data_from_api {
        save_lookup_data_to_db(pool, data).await?;
    }

    Ok(look_up_data_from_api)
}
