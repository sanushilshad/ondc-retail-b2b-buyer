use super::errors::UserRegistrationError;
use super::schemas::{AuthData, AuthenticateRequest, CreateUserAccount, UserType};
use super::utils::{fetch_user, get_auth_data, register_user};
use super::{errors::AuthError, utils::validate_credentials};
use crate::configuration::{SecretSetting, UserSettings};
use crate::schemas::GenericResponse;
// use crate::session_state::TypedSession;
use actix_web::{web, Result};
use sqlx::PgPool;

#[tracing::instrument(ret(Debug), err, name = "Authenticate User", skip(pool), fields())]
pub async fn authenticate(
    body: web::Json<AuthenticateRequest>,
    pool: web::Data<PgPool>,
    secret: web::Data<SecretSetting>,
) -> Result<web::Json<GenericResponse<AuthData>>, AuthError> {
    tracing::Span::current().record("request_body", &tracing::field::debug(&body));
    tracing::Span::current().record("identifier", &tracing::field::display(&body.identifier));
    match validate_credentials(body.0, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            match fetch_user(vec![&user_id.to_string()], &pool).await {
                Ok(Some(user_obj)) => {
                    let auth_obj = get_auth_data(user_obj, &secret.jwt.secret)?;
                    Ok(web::Json(GenericResponse::success(
                        "Successfully Authenticated User",
                        Some(auth_obj),
                    )))
                }
                Ok(None) | Err(_) => Err(AuthError::UnexpectedStringError(
                    "Internal Server Error".to_string(),
                )),
            }
        }
        Err(e) => {
            tracing::error!("Failed to authenticate user: {:?}", e);
            return Err(e);
        }
    }
}

#[tracing::instrument(ret(Debug), err, name = "Register User", skip(pool), fields())]
pub async fn register(
    body: web::Json<CreateUserAccount>,
    pool: web::Data<PgPool>,
    user: web::Data<UserSettings>,
) -> Result<web::Json<GenericResponse<()>>, UserRegistrationError> {
    // if let UserType::Admin | UserType::Superadmin = body.user_type {
    let admin_role = vec![UserType::Admin, UserType::Superadmin];
    if admin_role.contains(&body.user_type) && !user.admin_list.contains(&body.mobile_no) {
        return Err(UserRegistrationError::InsufficientPrevilegeError(
            "Insufficient previlege to register Admin/Superadmin".to_string(),
        ));
    } else {
        match register_user(body.0, &pool).await {
            Ok(uuid) => {
                tracing::Span::current().record("user_id", &tracing::field::display(&uuid));
                Ok(web::Json(GenericResponse::success(
                    "Sucessfully Registered User",
                    Some(()),
                )))
            }
            Err(e) => {
                tracing::error!("Failed to register user: {:?}", e);
                return Err(e);
            }
        }
    }
}
