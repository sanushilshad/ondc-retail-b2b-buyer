use super::schemas::AuthenticateRequest;
use super::{errors::AuthError, utils::validate_credentials};
use crate::schemas::GenericResponse;
use actix_web::{web, Result};
use sqlx::PgPool;

#[tracing::instrument(ret(Debug), err, name = "Authenticate User", skip(pool), fields())]
pub async fn authenticate(
    body: web::Json<AuthenticateRequest>,
    pool: web::Data<PgPool>,
) -> Result<web::Json<GenericResponse<()>>, AuthError> {
    tracing::Span::current().record("identifier", &tracing::field::display(&body.identifier));
    match validate_credentials(body.0, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            // session.renew();
            // session
            //     .insert_user_id(user_id)
            //     .map_err(|e| AuthError::UnexpectedError(e.into()))?;
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => AuthError::InvalidCredentials(e.into()),
                AuthError::UnexpectedError(_) => AuthError::UnexpectedError(e.into()),
                AuthError::ValidationError(_) => AuthError::ValidationError(e.into()),
                AuthError::ValidationStringError(_) => {
                    AuthError::ValidationStringError("Internal Server Error".to_string())
                }
            };
            tracing::error!("Failed to authenticate user: {:?}", e);
            // return Ok(web::Json(GenericResponse::success(
            //     &e.to_string(),
            //     Some(()),
            // )));
            return Err(e);
        }
    }
    Ok(web::Json(GenericResponse::success("BRUHHH", Some(()))))
}
