use crate::configuration::{ApplicationConfig, SecretConfig};
use crate::errors::GenericError;
use crate::schemas::{RequestMetaData, Status};
use crate::user_client::{AllowedPermission, BusinessAccount, CustomerType, UserAccount};
use crate::user_client::{PermissionType, UserClient};
use crate::utils::{
    bytes_to_payload, decode_token, get_header_value, validate_business_account_active,
};
// use actix_http::body::BoxBody;
use actix_web::body::{BoxBody, EitherBody, MessageBody};
use actix_web::dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::PayloadError;
use actix_web::http::header::UPGRADE;
use actix_web::web::{Bytes, BytesMut};
use actix_web::{body, http, web, Error, HttpMessage, HttpResponseBuilder, ResponseError};
use futures::future::LocalBoxFuture;
use futures::{Stream, StreamExt};
use futures_util::stream::{self};
use secrecy::SecretString;
use std::cell::RefCell;
use std::future::{self, ready, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::str;
use tracing::instrument;
use uuid::Uuid;

// Middlware for saving the request and response into the tracing
pub struct ReadReqResMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S> Service<ServiceRequest> for ReadReqResMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    #[instrument(skip(self), name = "Request Response Payload", fields(path = %req.path()))]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        //
        let is_websocket = req.headers().contains_key(UPGRADE)
            && req.headers().get(UPGRADE).unwrap() == "websocket";
        let is_on_search = req.path().ends_with("on_search");
        let is_non_json_req_res =
            req.path().contains("/docs/") || req.path().contains("/api-docs/");
        if is_websocket || is_non_json_req_res {
            Box::pin(async move {
                let fut = svc.call(req).await?;
                Ok(fut)
            })
        } else {
            Box::pin(async move {
                if !is_on_search {
                    let request_str: String = req.extract::<String>().await?;
                    tracing::info!({%request_str}, "HTTP Response");
                    req.set_payload(bytes_to_payload(web::Bytes::from(request_str)));
                }
                let fut = svc.call(req).await?;

                let (req, res) = fut.into_parts();
                let (res, body) = res.into_parts();
                let body_bytes = body::to_bytes(body).await.ok().unwrap();
                let response_str = match std::str::from_utf8(&body_bytes) {
                    Ok(response_body) => {
                        tracing::info!({%response_body}, "");
                        response_body.to_string()
                    }
                    Err(_) => {
                        tracing::error!("Error decoding response body");
                        String::from("")
                    }
                };

                let res = res.set_body(BoxBody::new(response_str));
                let res = ServiceResponse::new(req, res);
                Ok(res)
            })
        }
    }
}

pub struct SaveRequestResponse;

impl<S> Transform<S, ServiceRequest> for SaveRequestResponse
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ReadReqResMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReadReqResMiddleware {
            service: Rc::new(RefCell::new(service)),
        }))
    }
}

// Middleware to validate the header in incoming requests
pub struct HeaderMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for HeaderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = get_header_value(&req, "x-request-id");
        let device_id = get_header_value(&req, "x-device-id");
        // let hostname = get_header_value(&req, "Host");

        if request_id.is_none() || device_id.is_none() {
            let error_message = match (request_id.is_none(), device_id.is_none()) {
                (true, _) => "x-request-id is missing".to_string(),
                (_, true) => "x-device-id is missing".to_string(),
                _ => "".to_string(), // Default case, if none of the conditions are met
            };
            let (request, _pl) = req.into_parts();
            let json_error: GenericError = GenericError::ValidationError(error_message);
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        } else {
            let meta_data: RequestMetaData = RequestMetaData {
                request_id: request_id.unwrap().to_owned(),
                device_id: device_id.unwrap().to_owned(),
                // domain_uri: hostname.unwrap().to_owned(),
            };
            req.extensions_mut().insert::<RequestMetaData>(meta_data);
        }

        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

/// Middleware factory for requiring authentication.
pub struct HeaderValidation;

impl<S> Transform<S, ServiceRequest> for HeaderValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = HeaderMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(HeaderMiddleware {
            service: Rc::new(service),
        }))
    }
}

// Middleware the verfify ONDC requests coming from the seller networks

pub struct ReadReqResMiddleware2<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for ReadReqResMiddleware2<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    <B as MessageBody>::Error: ResponseError + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[instrument(skip(self), name = "Request Response Payload", fields(path = %req.path()))]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        //
        let is_websocket = req.headers().contains_key(UPGRADE)
            && req.headers().get(UPGRADE).unwrap() == "websocket";
        let is_on_search = req.path().ends_with("on_search");
        if is_websocket || is_on_search {
            Box::pin(async move {
                let fut: ServiceResponse<B> = svc.call(req).await?;
                Ok(fut.map_into_left_body())
            })
        } else {
            Box::pin(async move {
                // let route = req.path().to_owned();

                let mut request_body = BytesMut::new();

                while let Some(chunk) = req.take_payload().next().await {
                    request_body.extend_from_slice(&chunk?);
                }
                let body = request_body.freeze();
                match str::from_utf8(&body) {
                    Ok(request_str) => {
                        if let Ok(request_json) =
                            // tracing::Span::current().record("Request body", &tracing::field::display("Apple"));
                            serde_json::from_str::<serde_json::Value>(request_str)
                        {
                            tracing::info!({%request_json}, "HTTP Response");
                        } else {
                            tracing::info!("Non-JSON request: {}", request_str);
                            request_str.to_string();
                        }
                    }

                    Err(_) => {
                        tracing::error!("Something went wrong in request body parsing middleware");
                    }
                }
                let single_part: Result<Bytes, PayloadError> = Ok(body);
                let in_memory_stream = stream::once(future::ready(single_part));
                let pinned_stream: Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>>>> =
                    Box::pin(in_memory_stream);
                let in_memory_payload: Payload = pinned_stream.into();
                req.set_payload(in_memory_payload);
                let fut = svc.call(req).await?;

                let res_status = fut.status();
                let res_headers = fut.headers().clone();
                let new_request = fut.request().clone();
                let mut new_response = HttpResponseBuilder::new(res_status);
                let body_bytes = body::to_bytes(fut.into_body()).await?;
                match str::from_utf8(&body_bytes) {
                    Ok(response_str) => {
                        if let Ok(response_json) =
                            serde_json::from_str::<serde_json::Value>(response_str)
                        {
                            tracing::info!({%response_json}, "HTTP Response");
                            tracing::Span::current()
                                .record("Response body", tracing::field::display(&response_str));

                            response_str.to_string()
                        } else {
                            // Not JSON, log as a string
                            tracing::info!("Non-JSON response: {}", response_str);
                            response_str.to_string()
                        }
                    }
                    Err(_) => {
                        tracing::error!("Something went wrong in response body parsing middleware");
                        "Something went wrong in response response body parsing middleware".into()
                    }
                };
                for (header_name, header_value) in res_headers {
                    new_response.insert_header((header_name.as_str(), header_value));
                }
                let new_response = new_response.body(body_bytes.to_vec());
                // Create the new ServiceResponse
                Ok(ServiceResponse::new(
                    new_request,
                    new_response.map_into_right_body(),
                ))

                // }
            })
        }
    }
}

pub struct SaveRequestResponse2;

impl<S, B> Transform<S, ServiceRequest> for SaveRequestResponse2
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    <B as MessageBody>::Error: ResponseError + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ReadReqResMiddleware2<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReadReqResMiddleware2 {
            service: Rc::new(RefCell::new(service)),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = req
            .cookie("token")
            .map(|c| SecretString::from(c.value().to_string()))
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .and_then(|h| {
                        let auth_header = h.to_str().ok()?;
                        if auth_header.starts_with("Bearer ") {
                            Some(SecretString::from(auth_header.split_at(7).1.to_string()))
                        } else {
                            None
                        }
                    })
            });

        if token.is_none() {
            let error_message = "Authorization header is missing/invalid".to_string();
            let (request, _pl) = req.into_parts();
            let json_error = GenericError::ValidationError(error_message);
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        }

        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let user_client = req.app_data::<web::Data<UserClient>>().unwrap();
            let user = user_client.get_user_account(token.as_ref(), None).await?;
            if user.is_active == Status::Inactive {
                return Err(GenericError::ValidationError(
                    "User is Inactive. Please contact customer support".to_string(),
                ))?;
            } else if user.is_deleted {
                return Err(GenericError::ValidationError(
                    "User is in deleted. Please contact customer support".to_string(),
                ))?;
            }

            req.extensions_mut().insert::<UserAccount>(user);

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

/// Middleware factory for requiring authentication.
pub struct RequireAuth;

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

//Middleware to validate the business account
pub struct BusinessAccountMiddleware<S> {
    service: Rc<S>,
    pub business_type_list: Vec<CustomerType>,
}
impl<S> Service<ServiceRequest> for BusinessAccountMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    /// Handles incoming requests.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Attempt to extract token from cookie or authorization header

        let business_id = match get_header_value(&req, "x-business-id") // Convert HeaderValue to &str
            .and_then(|value| Uuid::parse_str(value).ok())
        {
            Some(business_id) => business_id,
            None => {
                let json_error = GenericError::ValidationError(
                    "x-business-id is missing or is invalid".to_string(),
                );
                let (request, _pl) = req.into_parts();
                return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
            }
        };
        let srv = Rc::clone(&self.service);
        let customer_type_list = self.business_type_list.clone();
        Box::pin(async move {
            let user_client = req.app_data::<web::Data<UserClient>>().unwrap();

            let user_account = req
                .extensions()
                .get::<UserAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError("User Account doesn't exist".to_string())
                })?
                .to_owned();

            let business_account = user_client
                .get_business_account(user_account.id, business_id, customer_type_list)
                .await?;
            let extracted_business_account = business_account.ok_or_else(|| {
                GenericError::ValidationError("Business Account doesn't exist".to_string())
            })?;
            let error_message = validate_business_account_active(&extracted_business_account);
            if let Some(message) = error_message {
                let (request, _pl) = req.into_parts();
                let json_error = GenericError::ValidationError(message);
                return Ok(ServiceResponse::from_err(json_error, request));
            }
            req.extensions_mut()
                .insert::<BusinessAccount>(extracted_business_account);

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

pub struct BusinessAccountValidation {
    pub business_type_list: Vec<CustomerType>,
}

impl<S> Transform<S, ServiceRequest> for BusinessAccountValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = BusinessAccountMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BusinessAccountMiddleware {
            service: Rc::new(service),
            business_type_list: self.business_type_list.clone(),
        }))
    }
}

//Middlware for verifying the permission
pub struct BusinessPermissionMiddleware<S> {
    service: Rc<S>,
    pub permission_list: Vec<PermissionType>,
}
impl<S> Service<ServiceRequest> for BusinessPermissionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    /// Handles incoming requests.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);

        let permission_list = self.permission_list.clone();
        Box::pin(async move {
            let user_client = req.app_data::<web::Data<UserClient>>().unwrap();
            let user_account = req
                .extensions()
                .get::<UserAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError("User Account doesn't exist".to_string())
                })?
                .to_owned();
            let business_account = req
                .extensions()
                .get::<BusinessAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError(
                        "Business Account Account doesn't exist".to_string(),
                    )
                })?
                .to_owned();
            let permission_list = user_client
                .permission_validation(user_account.id, business_account.id, permission_list)
                .await?;

            req.extensions_mut()
                .insert::<AllowedPermission>(AllowedPermission::new(
                    user_account.id,
                    business_account.id,
                    permission_list,
                ));

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

// Middleware factory for business account validation.
pub struct BusinessPermissionValidation {
    pub permission_list: Vec<PermissionType>,
}

impl<S> Transform<S, ServiceRequest> for BusinessPermissionValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = BusinessPermissionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(BusinessPermissionMiddleware {
            service: Rc::new(service),
            permission_list: self.permission_list.clone(),
        }))
    }
}

pub struct ServiceAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for ServiceAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .and_then(|h| {
                        let auth_header = h.to_str().ok()?;
                        if auth_header.starts_with("Bearer ") {
                            Some(auth_header.split_at(7).1.to_string())
                        } else {
                            None
                        }
                    })
            });

        let jwt_secret = &req
            .app_data::<web::Data<SecretConfig>>()
            .unwrap()
            .jwt
            .secret;

        let service_id = req
            .app_data::<web::Data<ApplicationConfig>>()
            .unwrap()
            .service_id;

        if token.is_none() {
            let error_message = "Authorization header is missing/invalid".to_string();
            let (request, _pl) = req.into_parts();
            let json_error = GenericError::ValidationError(error_message);
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        }
        let decoded_user_id = match decode_token(token.unwrap(), jwt_secret) {
            Ok(id) => id,
            Err(e) => {
                return Box::pin(async move {
                    Ok(ServiceResponse::from_err(
                        GenericError::InvalidJWT(e.to_string()),
                        req.into_parts().0,
                    ))
                });
            }
        };

        if service_id != decoded_user_id {
            return Box::pin(async move {
                Ok(ServiceResponse::from_err(
                    GenericError::InvalidJWT("Invalid Token".to_owned()),
                    req.into_parts().0,
                ))
            });
        }

        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

/// Middleware factory for mirco service authentication authentication.
pub struct RequireServiceAuth;

impl<S> Transform<S, ServiceRequest> for RequireServiceAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = ServiceAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ServiceAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}
