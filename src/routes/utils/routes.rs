// use crate::routes::utils::{get_customer_dbs_api, health_check};
use super::views::{get_customer_dbs_api, health_check};
use actix_web::web;
pub fn util_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/customer/database").route(web::post().to(get_customer_dbs_api)))
        .route("/health_check", web::get().to(health_check));
}
