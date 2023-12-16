use super::errors::UserRegistrationError;
use super::schemas::{AuthenticateRequest, CreateUserAccount};
use super::{errors::AuthError, utils::validate_credentials};
use crate::schemas::GenericResponse;
// use crate::session_state::TypedSession;
use actix_web::{web, Result};
use sqlx::PgPool;

#[tracing::instrument(ret(Debug), err, name = "Authenticate User", skip(pool), fields())]
pub async fn authenticate(
    body: web::Json<AuthenticateRequest>,
    pool: web::Data<PgPool>,
) -> Result<web::Json<GenericResponse<()>>, AuthError> {
    // tracing::Span::current().record("request_body", &tracing::field::debug(&body));
    tracing::Span::current().record("identifier", &tracing::field::display(&body.identifier));
    match validate_credentials(body.0, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
        }
        Err(e) => {
            tracing::error!("Failed to authenticate user: {:?}", e);
            return Err(e);
        }
    }
    Ok(web::Json(GenericResponse::success("BRUHHH", Some(()))))
}

#[tracing::instrument(ret(Debug), err, name = "Register User", skip(_pool), fields())]
pub async fn register(
    body: web::Json<CreateUserAccount>,
    _pool: web::Data<PgPool>,
) -> Result<web::Json<GenericResponse<()>>, UserRegistrationError> {
    Ok(web::Json(GenericResponse::success(
        "Sucessfully Registered User",
        Some(()),
    )))
}
