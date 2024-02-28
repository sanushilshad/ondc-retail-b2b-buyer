use super::errors::{BusinessRegistrationError, UserRegistrationError};
use super::schemas::{
    AuthData, AuthenticateRequest, CreateBusinessAccount, CreateUserAccount, UserAccount, UserType,
};
use super::utils::{create_business_account, fetch_user, get_auth_data, register_user};
use super::{errors::AuthError, utils::validate_user_credentials};
use crate::configuration::{SecretSetting, UserSettings};
use crate::schemas::GenericResponse;
// use crate::session_state::TypedSession;
use actix_web::{web, Result};
use sqlx::PgPool;

#[tracing::instrument(err, name = "Authenticate User", skip(pool, body), fields())]
pub async fn authenticate(
    body: web::Json<AuthenticateRequest>,
    pool: web::Data<PgPool>,
    secret: web::Data<SecretSetting>,
) -> Result<web::Json<GenericResponse<AuthData>>, AuthError> {
    tracing::Span::current().record("request_body", &tracing::field::debug(&body));
    tracing::Span::current().record("identifier", &tracing::field::display(&body.identifier));
    match validate_user_credentials(body.0, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            match fetch_user(vec![&user_id.to_string()], &pool).await {
                Ok(Some(user_obj)) => {
                    let auth_obj = get_auth_data(&pool, user_obj, &secret.jwt.secret).await?;
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

#[tracing::instrument(
    err,
    name = "User Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_user_account(
    body: web::Json<CreateUserAccount>,
    pool: web::Data<PgPool>,
    user_settings: web::Data<UserSettings>,
) -> Result<web::Json<GenericResponse<()>>, UserRegistrationError> {
    // if let UserType::Admin | UserType::Superadmin = body.user_type {
    let admin_role = vec![UserType::Admin, UserType::Superadmin];
    if admin_role.contains(&body.user_type) && !user_settings.admin_list.contains(&body.mobile_no) {
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

#[tracing::instrument(
    err,
    name = "Business Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_business_account(
    body: web::Json<CreateBusinessAccount>,
    pool: web::Data<PgPool>,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, BusinessRegistrationError> {
    // if let UserType::Admin | UserType::Superadmin = body.user_type {
    create_business_account(&pool, &user, &body).await?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully Registered Business Account",
        Some(()),
    )))
}
