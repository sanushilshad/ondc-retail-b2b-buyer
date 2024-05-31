use actix_web::web;

// use anyhow::Context;
use super::schemas::ProductSearchRequest;
use crate::configuration::ONDCSetting;
use crate::routes::ondc::buyer::utils::{get_ondc_search_payload, send_ondc_payload};
use crate::routes::ondc::ONDCActionType;
use crate::routes::product::errors::ProductSearchError;
use crate::routes::schemas::{BusinessAccount, UserAccount};
use crate::schemas::{GenericResponse, ONDCNPType, RequestMetaData};
use crate::utils::{create_authorization_header, get_np_detail};
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

#[tracing::instrument(name = "Product Search", skip(pool), fields())]
pub async fn product_search(
    body: web::Json<ProductSearchRequest>,
    pool: web::Data<PgPool>,
    ondc_obj: web::Data<ONDCSetting>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    meta_data: RequestMetaData,
) -> Result<web::Json<GenericResponse<()>>, ProductSearchError> {
    let np_detail = match get_np_detail(&pool, &meta_data.domain_uri, &ONDCNPType::Buyer).await {
        Ok(Some(np_detail)) => np_detail,
        Ok(None) => {
            return Err(ProductSearchError::ValidationError(format!(
                "{} is not a registered ONDC registered domain",
                meta_data.domain_uri
            )))
        }
        Err(e) => {
            println!("AAA {:?}", e);
            return Err(ProductSearchError::DatabaseError(
                "Something went wrong while fetching NP credentials".to_string(),
                e,
            ));
        }
    };
    let ondc_search_payload =
        get_ondc_search_payload(&user_account, &business_account, &body.0, &np_detail)?;
    let ondc_search_payload_str = serde_json::to_string(&ondc_search_payload)?;
    println!("{}", ondc_search_payload_str);
    let header = create_authorization_header(&ondc_search_payload_str, &np_detail, None, None)?;
    println!("{}", header);
    let response = send_ondc_payload(
        &ondc_obj.gateway_uri,
        &ondc_search_payload_str,
        &header,
        ONDCActionType::Search,
    )
    .await?;
    println!("{}", response);
    Ok(web::Json(GenericResponse::success(
        "Successfully Send Product Search Request",
        Some(()),
    )))
}
