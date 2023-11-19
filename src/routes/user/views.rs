use super::schema::AuthenticateRequest;
use crate::{schemas::GenericResponse, utils::error_chain_fmt};
use actix_web::http::StatusCode;
use actix_web::{web, Responder, ResponseError, Result};
use sqlx::PgPool;

#[derive(thiserror::Error)]
pub enum AuthError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AuthError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(ret(Debug), err, name = "Authenticate User", skip(_pool), fields())]
pub async fn authenticate(
    _body: web::Json<AuthenticateRequest>,
    _pool: web::Data<PgPool>,
) -> Result<impl Responder, AuthError> {
    // let _request_span = tracing::info_span!("starting fetching of databases.");
    tracing::info!("Authenticating user.");
    // Ok(web::Json({}))
    Ok(web::Json(GenericResponse::success("Fuck You")))
}
