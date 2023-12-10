use anyhow::Context;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use super::errors::AuthError;
use super::models::AuthMechanism;
use super::schemas::{AuthenticateRequest, AuthenticationScope};
use crate::utils::spawn_blocking_with_tracing;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

#[tracing::instrument(
    name = "Validate credentials",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    scope: &AuthenticationScope,
    pool: &PgPool,
) -> Result<Option<AuthMechanism>, anyhow::Error> {
    let row = sqlx::query_as!(
        AuthMechanism,
        r#"SELECT user_id, auth_identifier, secret,  auth_scope as "auth_scope: AuthenticationScope" from auth_mechanism
        as a inner join user_account as b on a.user_id = b.id where b.username = $1 AND auth_scope = $2"#,
        username,
        scope as &AuthenticationScope
        
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)

    // let secret_tuple: (uuid::Uuid, Secret<String>) =
    //     (Uuid::new_v4(), Secret::new("asxa".to_string()));
    // Ok(Some(secret_tuple))
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: AuthenticateRequest,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some(auth_mechanism) =
        get_stored_credentials(&credentials.identifier, &credentials.scope, pool).await?
    {
        user_id = Some(auth_mechanism.user_id);
        expected_password_hash = auth_mechanism.secret;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.secret)
    })
    .await
    .context("Failed to spawn blocking task.")??;

    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username."))
        .map_err(AuthError::InvalidCredentials)
}