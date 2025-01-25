use crate::{
    middleware::{BusinessAccountValidation, RequireAuth},
    user_client::CustomerType,
};

use super::handlers::{payment_notification, payment_order_creation};
use crate::middleware::BusinessPermissionValidation;
use crate::user_client::PermissionType;
use actix_web::web;
pub fn payment_route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/order/create").route(
            web::post()
                .to(payment_order_creation)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::CreateOrder],
                })
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
    cfg.service(web::resource("/notification").route(web::post().to(payment_notification)));
}
