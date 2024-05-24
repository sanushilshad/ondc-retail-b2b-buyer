use actix_web::web;

// use anyhow::Context;
use crate::configuration::ONDCSetting;
use crate::routes::ondc::buyer::utils::get_ondc_search_payload;
use crate::routes::product::errors::ProductSearchError;
use crate::routes::schemas::{BusinessAccount, UserAccount};
use crate::schemas::GenericResponse;
use sqlx::PgPool;

// use crate::routes::product::schemas::InventoryRequest;
// #[tracing::instrument(ret(Debug), name = "Fetching Inventory List", skip(_pool), fields())]
// pub async fn fetch_inventory(
//     _body: web::Json<InventoryRequest>,
//     _pool: web::Data<PgPool>,
// ) -> Result<HttpResponse, InventoryError> {
//     // let rapidor_invetory = sqlx::query_as::<_, ProductInventory>(
//     //     "SELECTs code, no_of_items FROM product_product limit 100",
//     // )
//     // .fetch_all(pool.get_ref())
//     // .await
//     // .context("Failed to fetch data from database")?;

//     // let _response = MyResponse {
//     //     status: true,
//     //     customer_message: "Inventory Fetched Sucessfully".to_string(),
//     //     data: rapidor_invetory,
//     //     success_code: "PYWS0000".to_string(),
//     // };
//     // web::Json(response)
//     Ok(HttpResponse::Ok().finish())
// }

use super::schemas::ProductSearchRequest;
#[tracing::instrument(name = "Product Search", skip(_pool), fields())]
pub async fn product_search(
    body: web::Json<ProductSearchRequest>,
    _pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCSetting>,
    user_account: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, ProductSearchError> {
    let ondc_search_payload =
        get_ondc_search_payload(&user_account, &business_account, &body.0, &ondc_obj)?;
    let ondc_search_payload_str = serde_json::to_string(&ondc_search_payload);
    println!("{}", ondc_search_payload_str.unwrap());
    Ok(web::Json(GenericResponse::success(
        "Successfully Send Product Search Request",
        Some(()),
    )))
}
