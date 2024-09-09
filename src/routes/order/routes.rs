use crate::routes::order::handlers::order_select;
use crate::routes::user::schemas::CustomerType;
use crate::routes::user::{BusinessAccountValidation, RequireAuth};
use actix_web::web;

use super::handlers::order_init;
pub fn order_route(cfg: &mut web::ServiceConfig) {
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
    cfg.service(
        web::resource("/init").route(
            web::post()
                .to(order_init)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::Buyer, CustomerType::Seller],
                })
                .wrap(RequireAuth),
        ),
    );
}
