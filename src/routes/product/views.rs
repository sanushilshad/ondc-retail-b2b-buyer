use actix_web::{web, HttpResponse};

// use anyhow::Context;
use sqlx::PgPool;

use crate::routes::product::errors::InventoryError;

use crate::routes::product::schemas::InventoryRequest;
#[tracing::instrument(ret(Debug), name = "Fetching Inventory List", skip(_pool), fields())]
pub async fn fetch_inventory(
    _body: web::Json<InventoryRequest>,
    _pool: web::Data<PgPool>,
) -> Result<HttpResponse, InventoryError> {
    // let rapidor_invetory = sqlx::query_as::<_, ProductInventory>(
    //     "SELECTs code, no_of_items FROM product_product limit 100",
    // )
    // .fetch_all(pool.get_ref())
    // .await
    // .context("Failed to fetch data from database")?;

    // let _response = MyResponse {
    //     status: true,
    //     customer_message: "Inventory Fetched Sucessfully".to_string(),
    //     data: rapidor_invetory,
    //     success_code: "PYWS0000".to_string(),
    // };
    // web::Json(response)
    Ok(HttpResponse::Ok().finish())
}
