use super::handlers::{on_confirm, on_init, on_search, on_select};
// use super::middlewares::SellerHeaderVerification;
use actix_web::web;
pub fn ondc_buyer_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/on_search").route(web::post().to(on_search)));
    cfg.service(web::resource("/on_select").route(web::post().to(on_select)));
    cfg.service(web::resource("/on_init").route(web::post().to(on_init)));
    cfg.service(web::resource("/on_confirm").route(web::post().to(on_confirm)));
}
