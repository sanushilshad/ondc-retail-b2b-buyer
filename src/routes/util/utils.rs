use actix_web::web;
use sqlx::PgPool;

use crate::routes::util::models::RapidorCustomer;

#[tracing::instrument(name = "Fetching customer data from database.", skip(pool), fields())]
pub async fn get_customer_dbs(
    pool: web::Data<PgPool>,
) -> Result<Vec<RapidorCustomer>, sqlx::Error> {
    let rapidor_customers =
        sqlx::query_as::<_, RapidorCustomer>("SELECT domain, database FROM customer_customer")
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
                // Using the `?` operator to return early
                // if the function failed, returning a sqlx::Error
                // We will talk about error handling in depth later!
            })?;
    tracing::info!("successfully fetched databases from database.");
    Ok(rapidor_customers)
}
