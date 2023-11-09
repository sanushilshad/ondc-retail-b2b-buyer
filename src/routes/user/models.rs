use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub struct RapidorCustomer {
    domain: String,
    database: String,
}
