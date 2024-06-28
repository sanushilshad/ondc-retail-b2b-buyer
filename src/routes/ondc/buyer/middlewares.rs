use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error};
use futures::future::LocalBoxFuture;
use sqlx::PgPool;

use std::future::{ready, Ready};
use std::rc::Rc;

use crate::configuration::ONDCSetting;
use crate::routes::ondc::utils::fetch_lookup_data;
use crate::schemas::ONDCNetworkType;
use crate::utils::{create_signing_string, get_header_value, hash_message, verify_response};

use crate::routes::ondc::ONDCContext;
use crate::utils::{bytes_to_payload, get_ondc_params_from_header};

use super::errors::ONDCBuyerError;

pub struct SellerHeaderVerificationMiddleware<S> {
    service: Rc<S>,
}
impl<S> Service<ServiceRequest> for SellerHeaderVerificationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    /// Handles incoming requests.
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let authorization_header = match get_header_value(&req, "Authorization") {
            Some(authorization_header) => authorization_header,
            None => {
                let json_error = ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: "Authorization is missing".to_string(),
                };
                let (request, _pl) = req.into_parts();
                return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
            }
        };

        let ondc_auth_params = match get_ondc_params_from_header(authorization_header) {
            Ok(params) => params,
            Err(e) => {
                let json_error = ONDCBuyerError::InvalidSignatureError { path: None };
                let (request, _pl) = req.into_parts();
                return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
            }
        };
        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let request_body_str: String = req.extract::<String>().await?;
            let db_pool = req.app_data::<web::Data<PgPool>>().unwrap();
            let request_body =
                serde_json::from_str::<serde_json::Value>(&request_body_str).unwrap();
            let registry_base_url = &req
                .app_data::<web::Data<ONDCSetting>>()
                .unwrap()
                .registry_base_url;
            if let Some(context_value) = request_body.get("context") {
                let context: ONDCContext = serde_json::from_value(context_value.clone())?;
                if let Some(bpp_id) = context.bpp_id {
                    match fetch_lookup_data(
                        db_pool,
                        &bpp_id,
                        &ONDCNetworkType::Bpp,
                        &context.domain,
                        &registry_base_url,
                    )
                    .await
                    {
                        Ok(data) => {
                            if let Some(data) = data {
                                let digest = &hash_message(&request_body_str);
                                let verfiy_res = verify_response(
                                    &ondc_auth_params.signature,
                                    &create_signing_string(
                                        digest,
                                        Some(ondc_auth_params.created_time),
                                        Some(ondc_auth_params.expires_time),
                                    ),
                                    &data.signing_public_key,
                                );
                                if let Err(err) = verfiy_res {
                                    let a = err.to_string();
                                    ONDCBuyerError::InvalidSignatureError { path: None };
                                }
                            } else {
                                ONDCBuyerError::InvalidResponseError {
                                    path: None,
                                    message: "Invalid BPP id".to_owned(),
                                };
                            }
                        }
                        Err(err) => {
                            ONDCBuyerError::InvalidResponseError {
                                path: None,
                                message: "Invalid BPP id".to_owned(),
                            };
                        }
                    }
                } else {
                    ONDCBuyerError::InvalidResponseError {
                        path: None,
                        message: "Missing BPP id".to_owned(),
                    };
                }
            } else {
                ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: "Missing context".to_owned(),
                };
            }

            req.set_payload(bytes_to_payload(web::Bytes::from(request_body_str)));
            let res = srv.call(req).await?;

            Ok(res)
        })
    }
}

// Middleware factory for ONDC Seller Auth validation.
pub struct SellerHeaderVerification;

impl<S> Transform<S, ServiceRequest> for SellerHeaderVerification
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = SellerHeaderVerificationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SellerHeaderVerificationMiddleware {
            service: Rc::new(service),
        }))
    }
}
