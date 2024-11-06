use crate::middleware::{BusinessAccountValidation, RequireAuth};
use crate::routes::order::handlers::order_select;
use crate::user_client::CustomerType;
use actix_web::web;

use super::handlers::{order_confirm, order_init, order_status};
pub fn order_route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/select").route(
            web::post()
                .to(order_select)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
    cfg.service(
        web::resource("/init").route(
            web::post()
                .to(order_init)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );

    cfg.service(
        web::resource("/confirm").route(
            web::post()
                .to(order_confirm)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );

    cfg.service(
        web::resource("/status")
            .route(web::post().to(order_status))
            .wrap(BusinessAccountValidation {
                business_type_list: vec![CustomerType::RetailB2bBuyer],
            })
            .wrap(RequireAuth),
    );
    cfg.service(
        web::resource("/cancel")
            .route(web::post().to(order_status))
            .wrap(BusinessAccountValidation {
                business_type_list: vec![CustomerType::RetailB2bBuyer],
            })
            .wrap(RequireAuth),
    );
}
