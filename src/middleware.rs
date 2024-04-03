use actix_web::dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::PayloadError;
use actix_web::web::{Bytes, BytesMut};
use actix_web::{body, http, web, Error, HttpMessage, HttpResponseBuilder, ResponseError};
use futures::future::LocalBoxFuture;
use futures::{Stream, StreamExt};
use sqlx::PgPool;
use std::cell::RefCell;
use std::future::{self, ready, Ready};
use std::pin::Pin;
use std::rc::Rc;
use tracing::instrument;

use crate::configuration::SecretSetting;
use crate::routes::user::errors::AuthError;
use crate::routes::user::utils::get_user;
use crate::schemas::Status;
use crate::utils::decode_token;
use actix_web::body::{EitherBody, MessageBody};
use std::str;

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}
use crate::routes::user::schemas::UserAccount;
impl<S> Service<ServiceRequest> for AuthMiddleware<S>
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
        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        // If token is missing, return unauthorized error
        let jwt_secret = &req
            .app_data::<web::Data<SecretSetting>>()
            .unwrap()
            .jwt
            .secret;

        if token.is_none() {
            let (request, _pl) = req.into_parts();
            let json_error =
                AuthError::ValidationStringError("Authorization token is missing".to_string());
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        }

        // Decode token and handle errors
        let user_id = match decode_token(&token.unwrap(), jwt_secret) {
            Ok(id) => id,
            Err(e) => {
                return Box::pin(async move {
                    let (request, _pl) = req.into_parts();
                    Ok(ServiceResponse::from_err(
                        AuthError::InvalidJWT(e.to_string()),
                        request,
                    ))
                });
            }
        };
        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let db_pool = &req.app_data::<web::Data<PgPool>>().unwrap();
            let user = get_user(vec![&user_id.to_string()], &db_pool)
                .await
                .map_err(|e| AuthError::UnexpectedError(e))?;
            if user.is_active == Status::Inactive {
                let (request, _pl) = req.into_parts();
                let json_error = AuthError::ValidationStringError(
                    "User is Inactive. Please contact customer support".to_string(),
                );
                return Ok(ServiceResponse::from_err(json_error, request));
            } else if user.is_deleted == true {
                let (request, _pl) = req.into_parts();
                let json_error = AuthError::ValidationStringError(
                    "User is in deleted. Please contact customer support".to_string(),
                );
                return Ok(ServiceResponse::from_err(json_error, request));
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

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

use actix_web::http::header::UPGRADE;
use futures_util::stream;
pub struct SaveRequestResponse;

impl<S, B> Transform<S, ServiceRequest> for SaveRequestResponse
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    <B as MessageBody>::Error: ResponseError + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
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

pub struct ReadReqResMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for ReadReqResMiddleware<S>
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

        if req.headers().contains_key(UPGRADE) && req.headers().get(UPGRADE).unwrap() == "websocket"
        {
            Box::pin(async move {
                let fut: ServiceResponse<B> = svc.call(req).await?;
                return Ok(fut.map_into_left_body());
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
                            // Successfully parsed as JSON
                            tracing::info!({%request_json}, "HTTP Response");
                        } else {
                            // Not JSON, log as a string
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

                let res_status = fut.status().clone();
                let res_headers = fut.headers().clone();
                let new_request = fut.request().clone();
                let mut new_response = HttpResponseBuilder::new(res_status);
                let body_bytes = body::to_bytes(fut.into_body()).await?;
                match str::from_utf8(&body_bytes) {
                    Ok(response_str) => {
                        if let Ok(response_json) =
                            serde_json::from_str::<serde_json::Value>(response_str)
                        {
                            // Successfully parsed as JSON
                            tracing::info!({%response_json}, "HTTP Response");
                            // Record the response JSON in the current span
                            tracing::Span::current()
                                .record("Response body", &tracing::field::display(&response_json));

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
