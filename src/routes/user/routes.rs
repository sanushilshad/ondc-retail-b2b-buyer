use actix_web::web;

use super::views::authenticate;

pub fn user_route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/authenticate").route(web::post().to(authenticate)));
}
