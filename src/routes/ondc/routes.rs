use actix_web::web;

use super::{buyer::ondc_buyer_route, seller::ondc_seller_route};
pub fn ondc_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    cfg.service(web::scope("/buyer").configure(ondc_buyer_route))
        .service(web::scope("/seller").configure(ondc_seller_route));
    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
