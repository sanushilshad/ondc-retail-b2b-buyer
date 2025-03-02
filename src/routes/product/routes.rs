use super::handlers::{
    cached_network_participant_list, cached_product_read, cached_provider_list,
    product_autocomplete, realtime_product_search,
};
use crate::middleware::{BusinessAccountValidation, RequireAuth};
use crate::user_client::CustomerType;
use actix_web::web;

pub fn product_route(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::resource("/inventory/fetch").route(web::post().to(fetch_inventory)));
    cfg.service(
        web::resource("/search/realtime").route(
            web::post()
                .to(realtime_product_search)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
    cfg.service(
        web::resource("/search/cache").route(
            web::post()
                .to(cached_product_read)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
    cfg.service(
        web::resource("/autocomplete").route(
            web::post()
                .to(product_autocomplete)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
    cfg.service(
        web::resource("/network_participant/fetch").route(
            web::post()
                .to(cached_network_participant_list)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
    cfg.service(
        web::resource("/provider/fetch").route(
            web::post()
                .to(cached_provider_list)
                .wrap(BusinessAccountValidation {
                    business_type_list: vec![CustomerType::RetailB2bBuyer],
                })
                .wrap(RequireAuth),
        ),
    );
}
