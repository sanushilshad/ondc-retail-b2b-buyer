use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error};
use futures::future::LocalBoxFuture;

use std::future::{ready, Ready};
use std::rc::Rc;

use crate::errors::GenericError;
use crate::utils::get_header_value;

use crate::routes::ondc::ONDCContext;
use crate::utils::{bytes_to_payload, get_ondc_params_from_header};

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
                let json_error =
                    GenericError::ValidationError("Authorization is missing".to_string());
                let (request, _pl) = req.into_parts();
                return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
            }
        };

        let _ondc_auth_params = match get_ondc_params_from_header(authorization_header) {
            Ok(params) => params,
            Err(e) => {
                let json_error = GenericError::ValidationError(format!("{}", e));
                let (request, _pl) = req.into_parts();
                return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
            }
        };
        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let data: String = req.extract::<String>().await?;
            let request_body = serde_json::from_str::<serde_json::Value>(&data).unwrap();
            let _context: ONDCContext = serde_json::from_value(request_body)?;
            req.set_payload(bytes_to_payload(web::Bytes::from(data)));
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
