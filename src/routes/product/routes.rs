use super::handlers::realtime_product_search;
use crate::middleware::{BusinessAccountValidation, RequireAuth};
use crate::user_client::CustomerType;
use actix_web::web;

pub fn product_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    cfg.service(
        web::resource("/search/realtime").route(
            web::post()
                .to(realtime_product_search)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
    cfg.service(
        web::resource("/search/cache").route(
            web::post()
                .to(realtime_product_search)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );

    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
