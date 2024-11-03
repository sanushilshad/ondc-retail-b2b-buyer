use super::ondc::{ondc_route, SellerHeaderVerification};
use crate::middleware::HeaderValidation;
use crate::openapi::ApiDoc;
use crate::routes::{notification_route, order_route, product_route, util_route};
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn main_route(cfg: &mut web::ServiceConfig) {
    let openapi = ApiDoc::openapi();
    cfg.service(web::scope("/notification").configure(notification_route))
        .service(web::scope("/util").configure(util_route))
        .service(
            web::scope("/order")
                .configure(order_route)
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/product")
                .configure(product_route)
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/v1/ondc/buyer")
                .configure(ondc_route)
                .wrap(SellerHeaderVerification),
        )
        .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", openapi.clone()));
}
