use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Debug, Display};

fn fmt_json<T: Serialize>(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", serde_json::to_string(value).unwrap())
}

macro_rules! impl_serialize_format {
    ($struct_name:ident, $trait_name:path) => {
        impl $trait_name for $struct_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt_json(self, f)
            }
        }
    };
}

impl_serialize_format!(InventoryRequest, Display);
#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct ProductInventory {
    #[sqlx(rename = "code")]
    product_code: String,
    #[sqlx(rename = "no_of_items")]
    qty: BigDecimal,
}

impl_serialize_format!(MyResponse, Display);
#[derive(Serialize, Deserialize)]
pub struct MyResponse {
    pub status: bool,
    pub customer_message: String,
    pub success_code: String,
    pub data: Vec<ProductInventory>,
}

impl_serialize_format!(InventoryRequest, Debug);
#[derive(Deserialize, Serialize)]
pub struct InventoryRequest {
    username: String,
    session_id: String,
    product_codes: Vec<String>,
}
