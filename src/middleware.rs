// use actix_web::{dev::ServiceRequest, web::BytesMut, HttpMessage};

// fn call(&self, mut req: ServiceRequest) -> Self::Future {
//     let svc = self.service.clone();

//     Box::pin(async move {
//         let mut body = BytesMut::new();
//         let mut stream = req.take_payload();

//         while let Some(chunk) = stream.next().await {
//             body.extend_from_slice(&chunk?);
//         }

//         let obj = serde_json::from_slice::<MyObj>(&body)?;
//         log::info!("{:?}", &obj);

//         //------- Reset the Payload data ----------
//         let (_, mut payload) = Payload::create(true);
//         payload.unread_data(body.into());
//         req.set_payload(payload.into());
//         // ----------------------------------------

// use std::cell::RefCell;
// use std::future::{ready, Future, Ready};
// use std::pin::Pin;
// use std::rc::Rc;

// use actix_web::body::{EitherBody, MessageBody};
// use actix_web::dev::{Payload, Transform};
// //         let res = svc.call(req).await?;
// //         Ok(res)
// //     })
// use actix_web::middleware::ErrorHandlerResponse;
// use actix_web::web::{Bytes, BytesMut};
// use actix_web::HttpResponseBuilder;
// use actix_web::{dev, http::header, Result};
// use actix_web::{
//     dev::{Service, ServiceRequest, ServiceResponse},
//     Error,
// };
// use tracing::{span, Instrument, Level};
// pub fn add_error_header<B>(mut res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
//     res.response_mut().headers_mut().insert(
//         header::CONTENT_TYPE,
//         header::HeaderValue::from_static("Error"),
//     );

//     Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
// }

// async fn tracing_middleware<S>(req: ServiceRequest, srv: &S) -> Result<ServiceResponse, Error>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
// {
//     // Create a span for the request
//     let span = span!(
//         Level::INFO,
//         "api_request",
//         method = %req.method(),
//         path = %req.uri().path(),
//     );

//     // Execute the service within the span
//     let mut response = srv.call(req).instrument(span).await?;

//     // Create a span for the response
//     let span = span!(
//         Level::INFO,
//         "api_response",
//         status = %response.status(),
//     );

//     // Log the response status code
//     tracing::info!(status = %response.status(), "API response status");

//     // Modify the response body if needed
//     let body_bytes = Bytes::from("Modified response body"); // You can modify this as needed
//     response
//         .response_mut()
//         .replace_body(actix_web::dev::ResponseBody::Body(body_bytes));

//     // Attach the span to the response and return it
//     Ok(response.instrument(span));
//     Ok(response)
// }

// pub struct LogEvents;

// impl<S: 'static, B> Transform<S, ServiceRequest> for LogEvents
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
//     B: MessageBody + 'static,
// {
//     type Response = ServiceResponse<EitherBody<B>>;
//     type Error = Error;
//     type Transform = LogEventsMiddleware<S>;
//     type InitError = ();
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;

//     fn new_transform(&self, service: S) -> Self::Future {
//         ok(LogEventsMiddleware {
//             service: Rc::new(RefCell::new(service)),
//         })
//     }
// }

// pub struct LogEventsMiddleware<S> {
//     service: Rc<RefCell<S>>,
// }

// impl<S: 'static, B> Service<ServiceRequest> for LogEventsMiddleware<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
//     B: MessageBody + 'static,
// {
//     type Response = ServiceResponse<EitherBody<B>>;
//     type Error = Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     actix_web::dev::forward_ready!(service);

//     fn call(&self, mut req: ServiceRequest) -> Self::Future {
//         let svc = self.service.clone();
//         let log_level = "info".to_string();

//         Box::pin(async move {
//             match log_level.as_str() {
//                 "debug" | "trace" | "info" => {
//                     let route = req.path().to_owned();

//                     /* we only process requests that are json */
//                     if !MiddlewareUtility::is_json_request(&req) {
//                         let res: ServiceResponse = svc.call(req).await?.map_into_boxed_body();
//                         return Ok(res.map_into_right_body());
//                     }

//                     /* extract and log the request */
//                     let mut request_body = BytesMut::new();
//                     while let Some(chunk) = req.take_payload().next().await {
//                         request_body.extend_from_slice(&chunk?);
//                     }

//                     match str::from_utf8(&request_body.to_vec().as_slice()) {
//                         Ok(str) => {
//                             /* identify routes that we will redact the body from,
//                             these are items that contain sensitive information we do not want to log
//                              */
//                             match route.as_str() {
//                                 "/x/protected_endpoint" => {
//                                     tracing::info!({ body = "Redacted" }, "HTTP Request");
//                                 }
//                                 _ => {
//                                     tracing::info!({body = %str}, "HTTP Request");
//                                 }
//                             }
//                         }
//                         Err(_) => {}
//                     };

//                     let (payload_sender, mut orig_payload) = Payload::create(true);
//                     orig_payload.unread_data(request_body.freeze());
//                     req.set_payload(Payload::from(orig_payload));

//                     /* extract and log the response */
//                     let res: ServiceResponse = svc.call(req).await?.map_into_boxed_body();
//                     if !MiddlewareUtility::is_json_response(&res) {
//                         return Ok(res.map_into_right_body());
//                     }

//                     let res_status = res.status().clone();
//                     let res_headers = res.headers().clone();
//                     let new_request = res.request().clone();
//                     let body_bytes = body::to_bytes(res.into_body()).await?;
//                     match str::from_utf8(&body_bytes) {
//                         Ok(str) => {
//                             tracing::info!({body = %str}, "HTTP Response");
//                             str
//                         }
//                         Err(_) => "Unknown",
//                     };

//                     /* build an identical response */
//                     let mut new_response = HttpResponseBuilder::new(res_status);
//                     for (header_name, header_value) in res_headers {
//                         new_response.insert_header((header_name.as_str(), header_value));
//                     }
//                     let new_response = new_response.body(body_bytes.to_vec());

//                     Ok(ServiceResponse::new(
//                         new_request,
//                         new_response.map_into_right_body(),
//                     ))
//                 }
//                 _ => {
//                     let res: ServiceResponse = svc.call(req).await?.map_into_boxed_body();
//                     Ok(res.map_into_right_body())
//                 }
//             }
//         })
//     }
// }

// pub struct TracingMiddleware;

// impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone,
//     S::Future: 'static,
//     B: MessageBody + 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Transform = TracingMiddlewareImpl<S>;
//     type InitError = ();
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;

//     fn new_transform(&self, service: S) -> Self::Future {
//         ok(TracingMiddlewareImpl { service })
//     }
// }

// pub struct TracingMiddlewareImpl<S> {
//     service: S,
// }

// impl<S, B> Service<ServiceRequest> for TracingMiddlewareImpl<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone,
//     S::Future: 'static,
//     B: MessageBody + 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     actix_web::dev::forward_ready!(service);

//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         let method = req.method().to_string();
//         let path = req.path().to_string();

//         let request_span = span!(
//             Level::INFO,
//             "http_request",
//             method = %method,
//             path = %path,
//         );

//         let response_span = span!(
//             Level::INFO,
//             "http_response",
//             method = %method,
//             path = %path,
//         );

//         let service = self.service.clone();
//         let fut = async move {
//             // Clone request for response logging
//             let cloned_request = req.clone();

//             let response = service.call(req).instrument(request_span).await?;

//             // Extract and log request body
//             let (req_parts, req_body) = cloned_request.into_parts();
//             let req_body_bytes =
//                 actix_web::web::Bytes::from(actix_web::body::to_bytes(req_body).await?);

//             tracing::info!(
//                 request_body = %String::from_utf8_lossy(&req_body_bytes),
//                 "Request Body"
//             );

//             // Extract and log response body
//             let (res_parts, res_body) = response.into_parts();
//             let res_body_bytes =
//                 actix_web::web::Bytes::from(actix_web::body::to_bytes(res_body).await?);

//             tracing::info!(
//                 response_body = %String::from_utf8_lossy(&res_body_bytes),
//                 "Response Body"
//             );

//             Ok(ServiceResponse::from_parts(res_parts, res_body))
//         };

//         Box::pin(fut.instrument(response_span))
//     }
// }

// pub struct TracingMiddleware;

// impl<S, B> actix_web::dev::Transform<S, ServiceRequest> for TracingMiddleware
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
//     S::Future: 'static,
//     B: actix_web::body::MessageBody + 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Transform = TracingMiddlewareImpl<S>;
//     type InitError = ();
//     type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;

//     fn new_transform(&self, service: S) -> Self::Future {
//         Box::pin(ready(Ok(TracingMiddlewareImpl { service })))
//     }
// }

// pub struct TracingMiddlewareImpl<S> {
//     service: S,
// }

// impl<S, B> Service<ServiceRequest> for TracingMiddlewareImpl<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
//     S::Future: 'static,
//     B: actix_web::body::MessageBody + 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     actix_web::dev::forward_ready!(service);

//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         let method = req.method().to_string();
//         let path = req.path().to_string();

//         let request_span = span!(
//             Level::INFO,
//             "http_request",
//             method = %method,
//             path = %path,
//         );

//         let service = self.service.clone();
//         let fut = async move {
//             // Clone request for response logging
//             let cloned_request = req.clone();

//             let response = service.call(req).instrument(request_span).await?;

//             // Extract and log request body
//             let (req_parts, req_body) = cloned_request.into_parts();
//             let req_body_bytes = Bytes::from(actix_web::body::to_bytes(req_body).await?);

//             tracing::info!(
//                 request_body = %String::from_utf8_lossy(&req_body_bytes),
//                 "Request Body"
//             );

//             // Extract and log response body
//             let (res_parts, res_body) = response.into_parts();
//             let res_body_bytes = Bytes::from(actix_web::body::to_bytes(res_body).await?);

//             tracing::info!(
//                 response_body = %String::from_utf8_lossy(&res_body_bytes),
//                 "Response Body"
//             );

//             Ok(ServiceResponse::from_parts(res_parts, res_body))
//         };

//         Box::pin(fut)
//     }
// }

use std::cell::RefCell;
use std::future::{self, ready, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix_web::dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::PayloadError;
use actix_web::web::{Bytes, BytesMut, Json};
use actix_web::{
    body, http, web, Error, HttpMessage, HttpResponse, HttpResponseBuilder, ResponseError,
};
use futures::future::LocalBoxFuture;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::configuration::SecretSetting;
use crate::routes::user::errors::AuthError;
use crate::routes::user::utils::get_user;
use crate::schemas::Status;
use crate::utils::decode_token;
use actix_web::body::{EitherBody, MessageBody};
use std::str;
// pub struct AuthMiddleware {
//     pub user_id: uuid::Uuid,
// }

// pub struct AuthMiddleware<S> {
//     service: Rc<S>,
// }
// impl FromRequest for AuthMiddleware {
//     type Error = ActixWebError;
//     type Future = Ready<Result<Self, Self::Error>>;
//     fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
//         let data = req.app_data::<web::Data<SecretSetting>>().unwrap();

//         let token = req
//             .cookie("token")
//             .map(|c| c.value().to_string())
//             .or_else(|| {
//                 req.headers()
//                     .get(http::header::AUTHORIZATION)
//                     .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
//             });

//         if token.is_none() {
//             let json_error = ErrorResponse {
//                 status: "fail".to_string(),
//                 message: "You are not logged in, please provide token".to_string(),
//             };
//             return ready(Err(ErrorUnauthorized(json_error)));
//         }

//         let user_id =
//             match decode_token(&token.unwrap(), data.jwt.secret.expose_secret().as_bytes()) {
//                 Ok(id) => id,
//                 Err(e) => {
//                     return ready(Err(ErrorUnauthorized(ErrorResponse {
//                         status: "fail".to_string(),
//                         message: e.to_string(),
//                     })))
//                 }
//             };

//         // let user_id = uuid::Uuid::parse_str(user_id.as_str()).unwrap();
//         req.extensions_mut()
//             .insert::<uuid::Uuid>(user_id.to_owned());

//         ready(Ok(AuthMiddleware { user_id }))
//     }
// }

// pub struct AuthMiddleware<S> {
//     service: Rc<S>,
// }
// use crate::routes::user::schemas::UserAccount;
// impl<S> Service<ServiceRequest> for AuthMiddleware<S>
// where
//     S: Service<
//             ServiceRequest,
//             Response = ServiceResponse<actix_web::body::BoxBody>,
//             Error = actix_web::Error,
//         > + 'static,
// {
//     type Response = ServiceResponse<actix_web::body::BoxBody>;
//     type Error = actix_web::Error;
//     type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

//     /// Polls the readiness of the wrapped service.
//     fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.service.poll_ready(ctx)
//     }

//     /// Handles incoming requests.
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         // Attempt to extract token from cookie or authorization header
//         let token = req
//             .cookie("token")
//             .map(|c| c.value().to_string())
//             .or_else(|| {
//                 req.headers()
//                     .get(http::header::AUTHORIZATION)
//                     .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
//             });

//         // If token is missing, return unauthorized error
//         if token.is_none() {
//             let json_error =
//                 AuthError::ValidationStringError("Authorization token is missing".to_string());
//             return Box::pin((ready(Err(ErrorUnauthorized(json_error)))));
//         }

//         let secret_state = req.app_data::<web::Data<SecretSetting>>().unwrap();

//         // Decode token and handle errors
//         let user_id = match decode_token(&token.unwrap(), &secret_state.jwt.secret) {
//             Ok(id) => id,
//             Err(e) => {
//                 return Box::pin(ready(Err(ErrorUnauthorized(
//                     AuthError::InvalidCredentials(e),
//                 ))))
//             }
//         };

//         // let cloned_app_state = secret_state.clone();
//         let srv = Rc::clone(&self.service);

//         // Handle user extraction and request processing
//         async move {
//             let db_pool = &req.app_data::<web::Data<PgPool>>().unwrap();
//             let user = get_user(vec![&user_id.to_string()], &db_pool)
//                 .await
//                 .map_err(|e| AuthError::InvalidCredentials(e))?;

//             // Insert user information into request extensions
//             req.extensions_mut().insert::<UserAccount>(user);

//             // Call the wrapped service to handle the request
//             let res = srv.call(req).await?;
//             Ok(res)
//         }
//         .boxed_local()
//     }
// }

// /// Middleware factory for requiring authentication.
// pub struct RequireAuth;

// impl<S> Transform<S, ServiceRequest> for RequireAuth
// where
//     S: Service<
//             ServiceRequest,
//             Response = ServiceResponse<actix_web::body::BoxBody>,
//             Error = actix_web::Error,
//         > + 'static,
// {
//     type Response = ServiceResponse<actix_web::body::BoxBody>;
//     type Error = actix_web::Error;
//     type Transform = AuthMiddleware<S>;
//     type InitError = ();
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;

//     /// Creates and returns a new AuthMiddleware wrapped in a Result.
//     fn new_transform(&self, service: S) -> Self::Future {
//         // Wrap the AuthMiddleware instance in a Result and return it.
//         ready(Ok(AuthMiddleware {
//             service: Rc::new(service),
//         }))
//     }
// }

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

    /// Polls the readiness of the wrapped service.
    forward_ready!(service);
    // fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    //     self.service.poll_ready(ctx)
    // }

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

// async fn log_request(request: &ServiceRequest) {
//     // Log request information
//     event!(
//         Level::info,
//         "Received request: {} {}",
//         request.method(),
//         request.path()
//     );
// }

// async fn log_response(response: &ServiceResponse) {
//     // Log response information
//     event!(
//         Level::Info,
//         "Sent response: {}",
//         response.response().status()
//     );
// }
use futures_util::stream;
pub struct SaveRequestResponse;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
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
    type Transform = TracingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracingMiddleware {
            service: Rc::new(RefCell::new(service)),
        }))
    }
}

pub struct TracingMiddleware<S> {
    service: Rc<RefCell<S>>,
}
// pub struct MessageBody{

// }

impl<S, B> Service<ServiceRequest> for TracingMiddleware<S>
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

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        // let fut = self.service.call(req);

        Box::pin(async move {
            // let route = req.path().to_owned();
            let mut request_body = BytesMut::new();
            while let Some(chunk) = req.take_payload().next().await {
                request_body.extend_from_slice(&chunk?);
            }
            let body = request_body.freeze();

            println!("Request payload {:?}", body);
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
                Ok(str) => {
                    // tracing::info!({body = %str}, "HTTP Response");
                    println!("Response payload{:?}", str);
                    str
                }
                Err(_) => "Unknown".into(),
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
        })
    }
}
