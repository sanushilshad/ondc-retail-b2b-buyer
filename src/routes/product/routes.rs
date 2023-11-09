use super::views::fetch_inventory;
use actix_web::web;
pub fn product_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));

    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
