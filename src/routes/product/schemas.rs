// impl_serialize_format!(InventoryRequest, Display);
// #[derive(sqlx::FromRow, Serialize, Deserialize)]
// pub struct ProductInventory {
//     #[sqlx(rename = "code")]
//     product_code: String,
//     #[sqlx(rename = "no_of_items")]
//     qty: BigDecimal,
// }

// impl_serialize_format!(MyResponse, Display);
// #[derive(Serialize, Deserialize)]
// pub struct MyResponse {
//     pub status: bool,
//     pub customer_message: String,
//     pub success_code: String,
//     pub data: Vec<ProductInventory>,
// }

// impl_serialize_format!(InventoryRequest, Debug);
// #[derive(Deserialize, Serialize)]
// pub struct InventoryRequest {
//     username: String,
//     session_id: String,
//     product_codes: Vec<String>,
// }

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchRequest {
    pub query: String,
    pub latitude: f64,
    pub longitude: f64,
    pub domain_category_id: String,
}
