use super::handlers::realtime_product_search;
use crate::routes::{schemas::CustomerType, BusinessAccountValidation, RequireAuth};
use actix_web::web;

pub fn product_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    cfg.service(
        web::resource("/realtime/search").route(
            web::post()
                .to(realtime_product_search)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::Buyer, CustomerType::Seller],
                })
                .wrap(RequireAuth),
        ),
    );

    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
