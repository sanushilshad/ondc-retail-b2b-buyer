use actix_web::web;

use super::handlers::ondc_seller_sample;
pub fn ondc_seller_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    cfg.service(web::resource("/search").route(web::post().to(ondc_seller_sample)));

    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
