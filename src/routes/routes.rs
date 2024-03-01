use crate::routes::{notification_route, product_route, user_route, util_route};
use actix_web::web;

use super::ondc::ondc_route;

pub fn main_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/notification").configure(notification_route))
        .service(web::scope("/utils").configure(util_route))
        .service(web::scope("/product").configure(product_route))
        .service(web::scope("/user").configure(user_route))
        .service(web::scope("/ondc").configure(ondc_route));
}
