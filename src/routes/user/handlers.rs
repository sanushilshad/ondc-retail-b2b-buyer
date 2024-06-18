use super::errors::{BusinessAccountError, UserRegistrationError};
use super::schemas::{
    AuthData, AuthenticateRequest, CreateBusinessAccount, CreateUserAccount, UserAccount, UserType,
};
use super::utils::{create_business_account, fetch_user, get_auth_data, register_user};
use super::{errors::AuthError, utils::validate_user_credentials};
use crate::configuration::{SecretSetting, UserSettings};
use crate::schemas::{GenericResponse, RequestMetaData};
// use crate::session_state::TypedSession;
use actix_web::{web, Result};
use sqlx::PgPool;

#[utoipa::path(
    post,
    path = "/user/authenticate",
    tag = "Authenticate User API",
    request_body(content = AuthenticateRequest, description = "Request Body"),
    responses(
        (status=200, description= "Authenticate User", body= AuthResponse),
    )
)]
#[tracing::instrument(err, name = "Authenticate User", skip(pool, body), fields())]
pub async fn authenticate(
    body: web::Json<AuthenticateRequest>,
    pool: web::Data<PgPool>,
    secret_obj: web::Data<SecretSetting>,
) -> Result<web::Json<GenericResponse<AuthData>>, AuthError> {
    tracing::Span::current().record("request_body", &tracing::field::debug(&body));
    tracing::Span::current().record("identifier", &tracing::field::display(&body.identifier));
    match validate_user_credentials(body.0, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            match fetch_user(vec![&user_id.to_string()], &pool).await {
                Ok(Some(user_obj)) => {
                    let auth_obj = get_auth_data(&pool, user_obj, &secret_obj.jwt).await?;
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

#[utoipa::path(
    post,
    path = "/user/register",
    tag = "Register User Account API",
    request_body(content = CreateUserAccount, description = "Request Body"),
    responses(
        (status=200, description= "Account created successfully", body= EmptyGenericResponse ),
    )
)]
#[tracing::instrument(
    err,
    name = "User Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_user_account(
    body: web::Json<CreateUserAccount>,
    pool: web::Data<PgPool>,
    meta_data: RequestMetaData,
    user_settings: web::Data<UserSettings>,
) -> Result<web::Json<GenericResponse<()>>, UserRegistrationError> {
    let admin_role = [UserType::Admin, UserType::Superadmin];
    if admin_role.contains(&body.user_type) && !user_settings.admin_list.contains(&body.mobile_no) {
        return Err(UserRegistrationError::InsufficientPrevilegeError(
            "Insufficient previlege to register Admin/Superadmin".to_string(),
        ));
    } else {
        match register_user(&pool, body.0, meta_data.domain_uri).await {
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

#[utoipa::path(
    post,
    path = "/user/register/business",
    tag = "Register Business Account API",
    request_body(content = CreateBusinessAccount, description = "Request Body"),
    responses(
        (status=200, description= "Business Account created successfully", body= EmptyGenericResponse),
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
      )
)]
#[tracing::instrument(
    err,
    name = "Business Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_business_account(
    body: web::Json<CreateBusinessAccount>,
    pool: web::Data<PgPool>,
    meta_data: RequestMetaData,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, BusinessAccountError> {
    // if let UserType::Admin | UserType::Superadmin = body.user_type {
    create_business_account(&pool, &user, &body, meta_data.domain_uri).await?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully Registered Business Account",
        Some(()),
    )))
}
