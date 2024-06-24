use super::{handlers::on_search, middleware::SellerHeaderVerification};
use actix_web::web;
pub fn ondc_buyer_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    // cfg.service(web::resource("/info").route(web::post().to(product_search)));
    cfg.service(
        web::resource("/on_search")
            .route(web::post().to(on_search))
            .wrap(SellerHeaderVerification),
    );

    // cfg.route("/customer/database", web::post().to(get_customer_dbs_api))
}
