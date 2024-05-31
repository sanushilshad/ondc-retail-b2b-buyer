use super::ondc::ondc_route;
use crate::middleware::HeaderValidation;
use crate::openapi::ApiDoc;
use crate::routes::{notification_route, product_route, user_route, util_route};
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn main_route(cfg: &mut web::ServiceConfig) {
    let openapi = ApiDoc::openapi();
    cfg.service(web::scope("/notification").configure(notification_route))
        .service(web::scope("/utils").configure(util_route))
        .service(
            web::scope("/product")
                .configure(product_route)
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/user")
                .configure(user_route)
                .wrap(HeaderValidation),
        )
        .service(web::scope("/v1/ondc").configure(ondc_route))
        .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", openapi.clone()));
}
