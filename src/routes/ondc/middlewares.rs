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
            Err(_) => {
                let json_error = ONDCBuyerError::InvalidSignatureError { path: None };
                let (request, _pl) = req.into_parts();
                return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
            }
        };
        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let request_body_str: String = req.extract::<String>().await?;
            let (parts, _body) = req.parts();
            let db_pool = req.app_data::<web::Data<PgPool>>().unwrap();
            let request_body =
                serde_json::from_str::<serde_json::Value>(&request_body_str).unwrap();
            let registry_base_url = &req
                .app_data::<web::Data<ONDCSetting>>()
                .unwrap()
                .registry_base_url;

            let context = request_body.get("context").ok_or_else(|| {
                ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: "Missing context".to_owned(),
                }
            })?;
            let context_obj: ONDCContext =
                serde_json::from_value(context.clone()).map_err(|_| {
                    ONDCBuyerError::InvalidResponseError {
                        path: None,
                        message: "Invalid context".to_owned(),
                    }
                })?;
            let look_up_data_obj = fetch_lookup_data(
                db_pool,
                &ondc_auth_params.subscriber_id,
                &ONDCNetworkType::Bpp,
                &context_obj.domain,
                registry_base_url,
            )
            .await
            .map_err(|_| ONDCBuyerError::InvalidResponseError {
                path: None,
                message: "Invalid BPP id".to_owned(),
            })?;

            let lookup_data =
                look_up_data_obj.ok_or_else(|| ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: "Invalid Subscriber id".to_owned(),
                })?;

            let digest = &hash_message(&request_body_str);
            let verfiy_res = verify_response(
                &ondc_auth_params.signature,
                &create_signing_string(
                    digest,
                    Some(ondc_auth_params.created_time),
                    Some(ondc_auth_params.expires_time),
                ),
                &lookup_data.signing_public_key,
            );
            if verfiy_res.is_err() {
                return Ok(ServiceResponse::from_err(
                    ONDCBuyerError::InvalidSignatureError { path: None },
                    parts.clone(),
                ));
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
