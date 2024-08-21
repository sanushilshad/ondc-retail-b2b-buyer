use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{http, web, Error, HttpMessage};
use futures::future::LocalBoxFuture;
use sqlx::PgPool;
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;

use crate::configuration::SecretSetting;
use crate::errors::GenericError;
use crate::routes::user::schemas::{BusinessAccount, CustomerType, UserAccount};
use crate::routes::user::utils::{
    get_business_account_by_customer_type, get_user, validate_business_account_active,
};
use crate::schemas::Status;
use crate::utils::{decode_token, get_header_value};

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
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        let jwt_secret = &req
            .app_data::<web::Data<SecretSetting>>()
            .unwrap()
            .jwt
            .secret;

        if token.is_none() {
            let error_message = "x-device-id is missing".to_string();
            let (request, _pl) = req.into_parts();
            let json_error = GenericError::ValidationError(error_message);
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        }

        let user_id = match decode_token(token.unwrap(), jwt_secret) {
            Ok(id) => id,
            Err(e) => {
                return Box::pin(async move {
                    let (request, _pl) = req.into_parts();
                    Ok(ServiceResponse::from_err(
                        GenericError::InvalidJWT(e.to_string()),
                        request,
                    ))
                });
            }
        };
        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let db_pool = &req.app_data::<web::Data<PgPool>>().unwrap();
            let user = get_user(vec![&user_id.to_string()], db_pool)
                .await
                .map_err(GenericError::UnexpectedError)?;
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
            let db_pool = req.app_data::<web::Data<PgPool>>().unwrap();
            let user_account = req
                .extensions()
                .get::<UserAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError("User Account doesn't exist".to_string())
                })?
                .to_owned();

            let business_account = get_business_account_by_customer_type(
                &user_account.id,
                &business_id,
                customer_type_list,
                db_pool,
            )
            .await
            .map_err(GenericError::UnexpectedError)?;
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

// Middlware for verifying the permission
// pub struct UserBusinessPermissionMiddleware<S> {
//     service: Rc<S>,
//     pub permission_list: Vec<String>,
// }
// impl<S> Service<ServiceRequest> for UserBusinessPermissionMiddleware<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
//         + 'static,
// {
//     type Response = ServiceResponse<actix_web::body::BoxBody>;
//     type Error = Error;
//     type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

//     forward_ready!(service);

//     /// Handles incoming requests.
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         println!("Hi from start. You requested: {}", req.path());

//         let fut = self.service.call(req);

//         Box::pin(async move {
//             let res = fut.await?;

//             println!("Hi from response");
//             Ok(res)
//         })
//     }
// }

// // Middleware factory for business account validation.
// pub struct UserBusinessPermissionValidation {
//     pub permission_list: Vec<String>,
// }

// impl<S> Transform<S, ServiceRequest> for UserBusinessPermissionValidation
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
//         + 'static,
// {
//     type Response = ServiceResponse<actix_web::body::BoxBody>;
//     type Error = Error;
//     type Transform = UserBusinessPermissionMiddleware<S>;
//     type InitError = ();
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;

//     /// Creates and returns a new AuthMiddleware wrapped in a Result.
//     fn new_transform(&self, service: S) -> Self::Future {
//         // Wrap the AuthMiddleware instance in a Result and return it.
//         ready(Ok(UserBusinessPermissionMiddleware {
//             service: Rc::new(service),
//             permission_list: self.permission_list.clone(),
//         }))
//     }
// }
