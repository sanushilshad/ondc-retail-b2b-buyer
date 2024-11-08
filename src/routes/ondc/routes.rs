use actix_web::web;

use super::handlers::{on_cancel, on_confirm, on_init, on_search, on_select, on_status};

pub fn ondc_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/on_search").route(web::post().to(on_search)));
    cfg.service(web::resource("/on_select").route(web::post().to(on_select)));
    cfg.service(web::resource("/on_init").route(web::post().to(on_init)));
    cfg.service(web::resource("/on_confirm").route(web::post().to(on_confirm)));
    cfg.service(web::resource("/on_status").route(web::post().to(on_status)));
    cfg.service(web::resource("/on_cancel").route(web::post().to(on_cancel)));
}
