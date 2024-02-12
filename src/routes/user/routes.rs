use actix_web::web;

use super::views::{authenticate, register};

pub fn user_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/register").route(web::post().to(register)));
    cfg.service(web::resource("/authenticate").route(web::post().to(authenticate)));
}
