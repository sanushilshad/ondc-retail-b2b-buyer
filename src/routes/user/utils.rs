use super::errors::{AuthError, UserRegistrationError};
use super::models::{AuthMechanismModel, UserAccountModel};
use super::schemas::{
    AuthData, AuthMechanism, AuthenticateRequest, AuthenticationScope, CreateUserAccount,
    JWTClaims, MaskingType, UserAccount, UserType, UserVectors,
};
use crate::schemas::Status;
use crate::utils::spawn_blocking_with_tracing;
use anyhow::{anyhow, Context};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm as JWTAlgorithm, EncodingKey, Header};
use secrecy::{ExposeSecret, Secret};
use sqlx::types::chrono::DateTime;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

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
    let row: Option<_> = sqlx::query!(
        r#"SELECT a.id as id, user_id, auth_identifier, secret, a.is_active as is_active, auth_scope as "auth_scope: AuthenticationScope", valid_upto from auth_mechanism
        as a inner join user_account as b on a.user_id = b.id where (b.username = $1 OR b.mobile_no = $1 OR  b.email = $1)  AND auth_scope = $2"#,
        username,
        scope as &AuthenticationScope
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let secret_string: Option<String> = row.secret;
        let secret = secret_string.map(Secret::new);

        Ok(Some(AuthMechanism {
            id: row.id,
            user_id: row.user_id,
            auth_scope: row.auth_scope,
            auth_identifier: row.auth_identifier,
            secret,
            is_active: row.is_active,
            valid_upto: row.valid_upto,
        }))
    } else {
        Ok(None)
    }
}

pub async fn verify_password(
    password: Secret<String>,
    auth_mechanism: AuthMechanism,
) -> Result<(), AuthError> {
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );
    if auth_mechanism.secret.is_some() {
        expected_password_hash = auth_mechanism.secret.unwrap_or(expected_password_hash);
    }

    spawn_blocking_with_tracing(move || verify_password_hash(expected_password_hash, password))
        .await
        .context("Failed to spawn blocking task.")?
}
pub async fn reset_otp(pool: &PgPool, auth_mechanism: &AuthMechanism) -> Result<(), anyhow::Error> {
    let _updated_tutor_result = sqlx::query!(
        r#"UPDATE auth_mechanism SET
        valid_upto = $1, secret = $2
        WHERE id = $3"#,
        None::<DateTime<Utc>>,
        None::<String>,
        auth_mechanism.id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute update query: {:?}", e);
        anyhow::anyhow!("Database error")
    })?;
    Ok(())
}

pub async fn verify_otp(
    pool: &PgPool,
    secret: Secret<String>,
    auth_mechanism: AuthMechanism,
) -> Result<(), AuthError> {
    let otp = auth_mechanism
        .secret
        .as_ref()
        .ok_or_else(|| AuthError::InvalidStringCredentials("Please Send the OTP".to_string()))?;

    if let Some(valid_upto) = auth_mechanism.valid_upto {
        if valid_upto <= Utc::now() {
            return Err(AuthError::InvalidStringCredentials(
                "OTP has expired".to_string(),
            ));
        }
    }
    if otp.expose_secret() != secret.expose_secret() {
        return Err(AuthError::InvalidStringCredentials(
            "Invalid OTP".to_string(),
        ))?;
    }
    reset_otp(pool, &auth_mechanism).await.map_err(|e| {
        AuthError::DatabaseError("Something went wrong while resetting OTP".to_string())
    })?;
    Ok(())
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: AuthenticateRequest,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;

    if let Some(auth_mechanism) =
        get_stored_credentials(&credentials.identifier, &credentials.scope, pool).await?
    {
        if !auth_mechanism.is_active {
            return Err(AuthError::InvalidStringCredentials(format!(
                "{:?} is not enabled for {}",
                credentials.scope, credentials.identifier
            )));
        }
        user_id = Some(auth_mechanism.user_id);
        match credentials.scope {
            AuthenticationScope::Password => {
                verify_password(credentials.secret, auth_mechanism).await?;
            }
            AuthenticationScope::Otp => {
                verify_otp(pool, credentials.secret, auth_mechanism).await?;
            }
            _ => {
                // Handle other cases if needed
            }
        }
    }

    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username"))
        .map_err(AuthError::InvalidCredentials)
}

fn get_user_account_from_model(user_model: UserAccountModel) -> Result<UserAccount, anyhow::Error> {
    let vectors_option: Vec<Option<UserVectors>> = user_model.vectors.0; // Extract the inner Option<Vec<UserVectors>>
    return Ok(UserAccount {
        id: user_model.id,
        mobile_no: user_model.mobile_no,
        username: user_model.username,
        email: user_model.email,
        is_active: user_model.is_active,
        display_name: user_model.display_name,
        vectors: vectors_option,
        user_type: user_model.user_type,
        international_dialing_code: user_model.international_dialing_code,
        user_account_number: user_model.user_account_number,
        alt_user_account_number: user_model.alt_user_account_number,
        is_test_user: user_model.is_test_user,
    });
}

#[tracing::instrument(name = "Get user Account", skip(pool))]
pub async fn fetch_user(
    value_list: Vec<&str>,
    pool: &PgPool,
) -> Result<Option<UserAccountModel>, anyhow::Error> {
    let val_list: Vec<String> = value_list.iter().map(|&s| s.to_string()).collect();

    let row: Option<UserAccountModel> = sqlx::query_as!(
        UserAccountModel,
        r#"SELECT id, username, is_test_user, mobile_no, email, is_active as "is_active!:Status", user_type as "user_type!:UserType", vectors as "vectors!:sqlx::types::Json<Vec<Option<UserVectors>>>", display_name, international_dialing_code, user_account_number, alt_user_account_number from user_account where email  = ANY($1) OR mobile_no  = ANY($1) OR id::text  = ANY($1)"#,
        &val_list
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub fn get_auth_data(
    user_model: UserAccountModel,
    jwt_secret: &Secret<String>,
) -> Result<AuthData, anyhow::Error> {
    let user_account = get_user_account_from_model(user_model)?;

    let user_id = user_account.id;
    let token = generate_jwt_token_for_user(user_id, None, jwt_secret)?;

    Ok(AuthData {
        user: user_account,
        token: token,
        business_account_list: vec![],
    })
}

// #[tracing::instrument(name = "Get stored credentials", skip(pool))]
// async fn fetch_user_by_uuid(
//     uuid: Uuid,
//     pool: &PgPool,
// ) -> Result<Option<UserAccountModel>, anyhow::Error> {
//     println!("('{}')", value_list.join("','"));

//     // let value_list_str =  format!("'{}'", value_list.join("','")) ;
//     let row: Option<UserAccountModel> = sqlx::query_as!(
//         UserAccountModel,
//         r#"SELECT id, username, mobile_no, email, is_active, vectors as "vectors!:sqlx::types::Json<Option<Vec<UserVectors>>>" from user_account where email  = ANY($1) OR mobile_no  = ANY($1)"#,
//         &val_list
//     )
//     .fetch_optional(pool)
//     .await?;

//     Ok(row)
// }

#[tracing::instrument(name = "create user account")]
pub fn create_vector_from_create_account(
    user_account: &CreateUserAccount,
) -> Result<Vec<UserVectors>, anyhow::Error> {
    let vector_list = vec![
        UserVectors {
            key: "email".to_string(),
            value: user_account.email.get().to_string(),
            masking: MaskingType::NA,
            verified: false,
        },
        UserVectors {
            key: "mobile_no".to_string(),
            value: user_account.mobile_no.to_string(),
            masking: MaskingType::NA,
            verified: false,
        },
    ];
    return Ok(vector_list);
}

#[tracing::instrument(name = "create user account", skip(transaction))]
pub async fn save_user(
    transaction: &mut Transaction<'_, Postgres>,
    user_account: &CreateUserAccount,
) -> Result<Uuid, anyhow::Error> {
    let user_id = Uuid::new_v4();
    let user_account_number = user_account.display_name.replace(" ", "-").to_lowercase();
    let vector_list = create_vector_from_create_account(user_account)?;
    let query = sqlx::query!(
        r#"
        INSERT INTO user_account (id, username, email, mobile_no, created_by, created_on, display_name, vectors, is_active, is_test_user, user_account_number, alt_user_account_number, user_type, international_dialing_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        user_id,
        user_account.username,
        user_account.email.get(),
        user_account.mobile_no,
        user_id,
        Utc::now(),
        user_account.display_name,
        sqlx::types::Json(vector_list) as sqlx::types::Json<Vec<UserVectors>>,
        Status::Active as Status,
        user_account.is_test_user,
        user_account_number,
        user_account_number,
        &user_account.user_type as &UserType,
        user_account.international_dialing_code
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure occured while saving user account".to_string(),
        )
    })?;
    Ok(user_id)
}

fn generate_jwt_token_for_user(
    user_id: Uuid,
    expiry_date: Option<DateTime<Utc>>,
    secret: &Secret<String>,
) -> Result<Secret<String>, anyhow::Error> {
    let expiration = match expiry_date {
        Some(expiry) => expiry.timestamp() as usize,
        None => Utc::now()
            .checked_add_signed(Duration::minutes(60))
            .expect("valid timestamp")
            .timestamp() as usize,
    };
    let claims: JWTClaims = JWTClaims {
        sub: user_id,
        exp: expiration as usize,
    };
    let header = Header::new(JWTAlgorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
    let token: String = encode(&header, &claims, &encoding_key).expect("Failed to generate token");
    return Ok(Secret::new(token));
}

// #[tracing::instrument(name = "register user", skip(pool))]
// pub async fn register_user(
//     user_account: CreateUserAccount,
//     pool: &PgPool,
// ) -> Result<uuid::Uuid, super::errors::UserRegistrationError> {
//     let mut transaction = pool
//         .begin()
//         .await
//         .context("Failed to acquire a Postgres connection from the pool")?;
//     match fetch_user(vec![user_account.email.get(),  &user_account.mobile_no],  pool).await{
//         Ok(Some(existing_user_obj)) => {
//             if user_account.mobile_no == existing_user_obj.mobile_no{
//                 tracing::error!("User Already exists with the given mobile number: {:?}", user_account.mobile_no);
//                 return Err(anyhow!("User Already exists with the given  mobile number")).map_err(UserRegistrationError::DuplicateMobileNo)?;
//             }

//             else {
//                 tracing::error!("User Already exists with the given  email: {:?}", user_account.email);
//                 return Err(anyhow!("User Already exists with given email")).map_err(UserRegistrationError::DuplicateEmail)?;
//             }

//         }
//         Ok(None) => {
//             tracing::info!("Successfully validated Email");
//             match save_user(&mut transaction, user_account).await{
//                 Ok(uuid) =>{
//                     tracing::info!("Successfully created user account {}", uuid);
//                     transaction
//                     .commit()
//                     .await
//                     .context("Failed to commit SQL transaction to store a new subscriber.")?;
//                     return  Ok(uuid);
//                 }
//                 Err(e)=>{
//                     let error = anyhow::Error::from(e);
//                     tracing::error!("Something went wrong while registering user: {:?}", error);
//                     return Err(anyhow!("Internal Server Error")).map_err(UserRegistrationError::UnexpectedError)?;
//                 }

//             }
//         }
//         Err(e) => {
//             tracing::error!("Something went wrong while validating user id: {:?}", e);
//             return Err(e).map_err(UserRegistrationError::UnexpectedError)?;
//         }
//     }
// }

fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(Secret::new(password_hash))
}

#[tracing::instrument(name = "save auth mechanism", skip(transaction))]
pub async fn save_auth_mechanism(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    user_account: CreateUserAccount,
) -> Result<(), anyhow::Error> {
    let current_utc = Utc::now();
    let password_hash =
        spawn_blocking_with_tracing(move || compute_password_hash(user_account.password))
            .await?
            .context("Failed to hash password")?;
    // let password_hash = compute_password_hash(user_account.password)?;
    let id = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let user_id_list = vec![user_id, user_id, user_id];
    let auth_scope = vec![
        AuthenticationScope::Password,
        AuthenticationScope::Otp,
        AuthenticationScope::Email,
    ];
    let auth_identifier = vec![
        user_account.username.clone(),
        user_account.mobile_no.clone(),
        user_account.email.get().to_string(),
    ];
    let secret = vec![password_hash.expose_secret().to_string()];
    let is_active = vec![true, true, true];
    let created_on = vec![current_utc, current_utc, current_utc];
    let created_by = vec![user_id, user_id, user_id];
    let query = sqlx::query!(
        r#"
        INSERT INTO auth_mechanism (id, user_id, auth_scope, auth_identifier, secret, is_active, created_at, created_by)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::user_auth_identifier_scope[], $4::text[], $5::text[], $6::bool[], $7::TIMESTAMP[], $8::text[])
        "#,
        &id[..] as &[Uuid],
        &user_id_list[..] as &[Uuid],
        &auth_scope[..] as &[AuthenticationScope],
        &auth_identifier[..],
        &secret[..],
        &is_active[..],
        &created_on[..] as &[DateTime<Utc>],
        &created_by[..] as &[Uuid]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure was encountered while saving auth mechanisms".to_string(),
        )
    })?;
    Ok(())
}

#[tracing::instrument(name = "register users", skip(pool))]
pub async fn register_user(
    user_account: CreateUserAccount,
    pool: &PgPool,
) -> Result<uuid::Uuid, super::errors::UserRegistrationError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    // Early return if user already exists
    if let Some(existing_user_obj) = fetch_user(
        vec![user_account.email.get(), &user_account.mobile_no],
        pool,
    )
    .await?
    {
        if user_account.mobile_no == existing_user_obj.mobile_no {
            let message = format!(
                "User Already exists with the given mobile number: {}",
                user_account.mobile_no
            );
            tracing::error!(message);
            return Err(anyhow!(message)).map_err(UserRegistrationError::DuplicateMobileNo);
        } else {
            let message = format!(
                "User Already exists with the given email: {}",
                user_account.email
            );
            tracing::error!(message);
            return Err(anyhow!(message)).map_err(UserRegistrationError::DuplicateEmail);
        }
    }
    let uuid = save_user(&mut transaction, &user_account).await?;
    save_auth_mechanism(&mut transaction, uuid, user_account).await?;
    tracing::info!("Successfully created user account {}", uuid);

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;

    Ok(uuid)

    // return Err(
    //     anyhow!("Duplicate mobile number")
    // ).map_err(UserRegistrationError::DuplicateEmail)?;
    // Ok(Uuid::new_v4())
}
