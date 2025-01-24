use crate::middleware::{BusinessAccountValidation, BusinessPermissionValidation, RequireAuth};
use crate::user_client::{CustomerType, PermissionType};
use actix_web::web;

use super::handlers::{
    order_cancel, order_confirm, order_fetch, order_init, order_list, order_select, order_status,
    order_update,
};
pub fn order_route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/select").route(
            web::post()
                .to(order_select)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![
                        PermissionType::CreateOrder,
                        PermissionType::CreateOrderSelf,
                    ],
                })
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
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![
                        PermissionType::CreateOrder,
                        PermissionType::CreateOrderSelf,
                    ],
                })
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
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![
                        PermissionType::CreateOrder,
                        PermissionType::CreateOrderSelf,
                    ],
                })
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );

    cfg.service(
        web::resource("/status")
            .route(web::post().to(order_status))
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateOrder],
            })
            .wrap(BusinessAccountValidation {
                business_type_list: vec![CustomerType::RetailB2bBuyer],
            })
            .wrap(RequireAuth),
    );
    cfg.service(
        web::resource("/cancel")
            .route(web::post().to(order_cancel))
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CancelOrder, PermissionType::CancelOrderSelf],
            })
            .wrap(BusinessAccountValidation {
                business_type_list: vec![CustomerType::RetailB2bBuyer],
            })
            .wrap(RequireAuth),
    );

    cfg.service(
        web::resource("/update")
            .route(web::post().to(order_update))
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::UpdateOrder, PermissionType::UpdateOrderSelf],
            })
            .wrap(BusinessAccountValidation {
                business_type_list: vec![CustomerType::RetailB2bBuyer],
            })
            .wrap(RequireAuth),
    );
    cfg.service(
        web::resource("/read")
            .route(web::post().to(order_fetch))
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::ReadOrder, PermissionType::ReadOrderSelf],
            })
            .wrap(BusinessAccountValidation {
                business_type_list: vec![CustomerType::RetailB2bBuyer],
            })
            .wrap(RequireAuth),
    );
    cfg.service(
        web::resource("/list")
            .route(web::post().to(order_list))
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::ListOrder, PermissionType::ListOrderSelf],
            })
            .wrap(BusinessAccountValidation {
                business_type_list: vec![CustomerType::RetailB2bBuyer],
            })
            .wrap(RequireAuth),
    );
}
