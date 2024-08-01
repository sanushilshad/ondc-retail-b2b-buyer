use super::handlers::{on_search, on_select};
use super::middlewares::SellerHeaderVerification;
use actix_web::web;
pub fn ondc_buyer_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    // cfg.service(web::resource("/info").route(web::post().to(product_search)));
    cfg.service(
        web::resource("/on_search")
            .route(web::post().to(on_search))
            .wrap(SellerHeaderVerification),
    );
    cfg.service(
        web::resource("/on_select")
            .route(web::post().to(on_select))
            .wrap(SellerHeaderVerification), // .wrap(SellerHeaderVerification),
    );
}
