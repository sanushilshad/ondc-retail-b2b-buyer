use super::handlers::product_search;
use crate::{
    middleware::{BusinessAccountValidation, RequireAuth},
    routes::schemas::CustomerType,
};
use actix_web::web;

pub fn product_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    cfg.service(
        web::resource("/search").route(
            web::post()
                .to(product_search)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::Buyer, CustomerType::Seller],
                })
                .wrap(RequireAuth),
        ),
    );

    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
