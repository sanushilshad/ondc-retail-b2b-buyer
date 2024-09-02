use sqlx::PgPool;
use crate::routes:: user::schemas::{BusinessAccount, UserAccount};
use crate::routes::product::schemas::{FulfillmentType, PaymentType, ProductSearchType};
use crate::schemas::RequestMetaData;
use super::schemas::ProductSearchRequest;
use chrono::Utc;

#[tracing::instrument(name = "Save Product Search Request", skip(pool))]
pub async fn save_search_request(
    pool: &PgPool,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    meta_data: &RequestMetaData,
    search_request: &ProductSearchRequest,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO search_request (message_id, transaction_id, device_id, business_id,  user_id, created_on, update_cache, query, payment_type, domain_category_code, search_type, fulfillment_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        &search_request.message_id,
        &search_request.transaction_id,
        &meta_data.device_id,
        &business_account.id,
        &user_account.id,
        Utc::now(),
        &search_request.update_cache,
        &search_request.query,
        &search_request.payment_type as &Option<PaymentType>, 
        &search_request.domain_category_code.to_string(),
        &search_request.search_type as &ProductSearchType,
        &search_request.fulfillment_type as &Option<FulfillmentType>

    )
    .execute(pool).await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving ONDC search request")
    })?;
    Ok(())
}
