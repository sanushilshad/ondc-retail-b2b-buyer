use actix_web::web;

use super::buyer::{middlewares::SellerHeaderVerification, ondc_buyer_route};
use super::seller::ondc_seller_route;
pub fn ondc_route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/buyer")
            .configure(ondc_buyer_route)
            .wrap(SellerHeaderVerification),
    )
    .service(web::scope("/seller").configure(ondc_seller_route));
}
