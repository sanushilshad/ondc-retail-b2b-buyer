// use crate::routes::{schemas::CustomerType, BusinessAccountValidation, RequireAuth};
use crate::routes::order::handlers::order_select;
use crate::routes::user::schemas::CustomerType;
use crate::routes::user::{BusinessAccountValidation, RequireAuth};
use actix_web::web;
pub fn order_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    cfg.service(
        web::resource("/select").route(
            web::post()
                .to(order_select)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::Buyer, CustomerType::Seller],
                })
                .wrap(RequireAuth),
        ),
    );

    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
