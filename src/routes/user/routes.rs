use actix_web::web;

use crate::middleware::{HeaderValidation, RequireAuth};

use super::views::{authenticate, register_business_account, register_user_account};

pub fn user_route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/register")
            .route(web::post().to(register_user_account))
            .wrap(HeaderValidation),
    );
    cfg.service(web::resource("/authenticate").route(web::post().to(authenticate)));
    cfg.service(
        web::resource("/register/business")
            .route(web::post().to(register_business_account))
            .wrap(RequireAuth)
            .wrap(HeaderValidation),
    );
}
