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

use core::fmt;
use std::future::{ready, Ready};

use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};
use secrecy::ExposeSecret;
use serde::Serialize;

use crate::configuration::SecretSetting;
use crate::utils::decode_token;

pub struct AuthMiddleware {
    pub user_id: uuid::Uuid,
}
#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
impl FromRequest for AuthMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let data = req.app_data::<web::Data<SecretSetting>>().unwrap();

        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            let json_error = ErrorResponse {
                status: "fail".to_string(),
                message: "You are not logged in, please provide token".to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        let user_id =
            match decode_token(&token.unwrap(), data.jwt.secret.expose_secret().as_bytes()) {
                Ok(id) => id,
                Err(e) => {
                    return ready(Err(ErrorUnauthorized(ErrorResponse {
                        status: "fail".to_string(),
                        message: e.to_string(),
                    })))
                }
            };

        // let user_id = uuid::Uuid::parse_str(user_id.as_str()).unwrap();
        req.extensions_mut()
            .insert::<uuid::Uuid>(user_id.to_owned());

        ready(Ok(AuthMiddleware { user_id }))
    }
}
