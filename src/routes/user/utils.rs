use core::str;

use super::errors::{AuthError, BusinessAccountError, UserRegistrationError};
use super::models::{AuthMechanismModel, BusinessAccountModel, UserAccountModel, UserBusinessRelationAccountModel, UserRoleModel};
use super::schemas::{
    AccountRole, AuthContextType, AuthData, AuthMechanism, AuthenticateRequest, AuthenticationScope, BasicBusinessAccount, BulkAuthMechanismInsert, BusinessAccount, CreateBusinessAccount, CreateUserAccount, DataSource, KYCProof, MaskingType, UserAccount, UserType, UserVector, VectorType,
};
use crate::configuration::JWT;
use crate::routes::schemas::{CustomerType, MerchantType, TradeType};
use crate::schemas::{Status, KycStatus};
use crate::general_utils::{generate_jwt_token_for_user, spawn_blocking_with_tracing};
use anyhow::{anyhow, Context};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use chrono::Utc;
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

#[tracing::instrument(name = "Get Auth Mechanism model", skip(username, pool))]
async fn get_auth_mechanism_model(username: &str,
    scope: &AuthenticationScope,
    pool: &PgPool,
    auth_context: AuthContextType
) -> Result<Option<AuthMechanismModel>, anyhow::Error> {
    let row: Option<AuthMechanismModel> = sqlx::query_as!(AuthMechanismModel, 
        r#"SELECT a.id as id, user_id, auth_identifier, secret, a.is_active as "is_active: Status", auth_scope as "auth_scope: AuthenticationScope", auth_context as "auth_context: AuthContextType", valid_upto from auth_mechanism
        as a inner join user_account as b on a.user_id = b.id where (b.username = $1 OR b.mobile_no = $1 OR  b.email = $1)  AND auth_scope = $2 AND auth_context = $3"#,
        username,
        scope as &AuthenticationScope,
        &auth_context as &AuthContextType
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    scope: &AuthenticationScope,
    pool: &PgPool,
    auth_context: AuthContextType
) -> Result<Option<AuthMechanism>, anyhow::Error> {


    if let Some(row) = get_auth_mechanism_model(username, scope, pool, auth_context).await? {
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
            auth_context: row.auth_context
        }))
    } else {
        Ok(None)
    }
}
#[tracing::instrument(name = "Verify Password")]
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

#[tracing::instrument(name = "Reset OTP")]
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

#[tracing::instrument(name = "Verify OTP")]
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
        tracing::error!("Failed to execute verify_otp database query: {:?}", e);
        AuthError::DatabaseError(
            "Something went wrong while resetting OTP".to_string(),
            e,
        )
    })?;
    Ok(())
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_user_credentials(
    credentials: AuthenticateRequest,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;

    if let Some(auth_mechanism) =
        get_stored_credentials(&credentials.identifier, &credentials.scope, pool, AuthContextType::UserAccount).await?
    {
        if auth_mechanism.is_active == Status::Inactive{
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
    let vectors_option: Vec<Option<UserVector>> = user_model.vectors.0; // Extract the inner Option<Vec<UserVectors>>
    Ok(UserAccount {
        id: user_model.id,
        mobile_no: user_model.mobile_no,
        username: user_model.username,
        email: user_model.email,
        is_active: user_model.is_active,
        display_name: user_model.display_name,
        vectors: vectors_option,
        international_dialing_code: user_model.international_dialing_code,
        user_account_number: user_model.user_account_number,
        alt_user_account_number: user_model.alt_user_account_number,
        is_test_user: user_model.is_test_user,
        is_deleted: user_model.is_deleted,
        user_role: user_model.role_name
    })
}

#[tracing::instrument(name = "Get user Account", skip(pool))]
pub async fn fetch_user(
    value_list: Vec<&str>,
    pool: &PgPool,
) -> Result<Option<UserAccountModel>, anyhow::Error> {
    let val_list: Vec<String> = value_list.iter().map(|&s| s.to_string()).collect();

    let row: Option<UserAccountModel> = sqlx::query_as!(
        UserAccountModel,
        r#"SELECT 
            ua.id, username, is_test_user, mobile_no, email, is_active as "is_active!:Status", 
            vectors as "vectors!:sqlx::types::Json<Vec<Option<UserVector>>>", display_name, 
            international_dialing_code, user_account_number, alt_user_account_number, ua.is_deleted, r.role_name FROM user_account as ua
            INNER JOIN user_role ur ON ua.id = ur.user_id
            INNER JOIN role r ON ur.role_id = r.id
        WHERE ua.email = ANY($1) OR ua.mobile_no = ANY($1) OR ua.id::text = ANY($1)
        "#,
        &val_list
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}





// #[tracing::instrument(name = "Get user Account by role realation", skip(pool))]
// pub async fn fetch_user_by_role(
//     value_list: Vec<&str>,
//     pool: &PgPool,
// ) -> Result<Option<UserAccountModel>, anyhow::Error> {
//     let val_list: Vec<String> = value_list.iter().map(|&s| s.to_string()).collect();

//     let row: Option<UserAccountModel> = sqlx::query_as!(
//         UserAccountModel,
//         r#"SELECT 
//             ua.id, username, is_test_user, mobile_no, email, is_active as "is_active!:Status", 
//             vectors as "vectors!:sqlx::types::Json<Vec<Option<UserVectors>>>", display_name, 
//             international_dialing_code, user_account_number, alt_user_account_number, ua.is_deleted, r.role_name FROM  user_role ur 
//             INNER JOIN user_account as ua ON ua.id = ur.user_id
//             INNER JOIN role r ON ur.role_id = r.id
//         WHERE ua.email = ANY($1) OR ua.mobile_no = ANY($1) OR ua.id::text = ANY($1)
//         "#,
//         &val_list
//     )
//     .fetch_optional(pool)
//     .await?;

//     Ok(row)
// }


#[tracing::instrument(name = "Get User by value list")]
pub async fn get_user(value_list: Vec<&str>, pool: &PgPool) -> Result<UserAccount, anyhow::Error> {
    match fetch_user(value_list, pool).await {
        Ok(Some(user_obj)) => {
            let user_account = get_user_account_from_model(user_obj)?;
            Ok(user_account)
        }
        Ok(None)=> Err(anyhow!("User doesn't exist")),
        Err(_) => Err(anyhow!("Internal Server Error"))
    }
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
) -> Result<Vec<UserVector>, anyhow::Error> {
    let vector_list = vec![
        UserVector {
            key: VectorType::Email,
            value: user_account.email.get().to_string(),
            masking: MaskingType::NA,
            verified: false,
        },
        UserVector {
            key: VectorType::MobileNo,
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
    subscriber_id: String
) -> Result<Uuid, anyhow::Error> {
    let user_id = Uuid::new_v4();
    let user_account_number = user_account.display_name.replace(' ', "-").to_lowercase();
    let vector_list = create_vector_from_create_account(user_account)?;
    let query = sqlx::query!(
        r#"
        INSERT INTO user_account (id, username, email, mobile_no, created_by, created_on, display_name, vectors, is_active, is_test_user, user_account_number, alt_user_account_number, international_dialing_code, source, subscriber_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#,
        user_id,
        &user_account.username,
        &user_account.email.get(),
        &user_account.mobile_no,
        user_id,
        Utc::now(),
        &user_account.display_name,
        sqlx::types::Json(vector_list) as sqlx::types::Json<Vec<UserVector>>,
        Status::Active as Status,
        &user_account.is_test_user,
        &user_account_number,
        &user_account_number,
        &user_account.international_dialing_code,
        &user_account.source as &DataSource,
        &subscriber_id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure occured while saving user account".to_string(),
            e.into(),
        )
    })?;
    Ok(user_id)
}


#[tracing::instrument(name = "get_role_model", skip(pool))]
pub async fn get_role_model(pool: &PgPool, role_type: &UserType) -> Result<Option<UserRoleModel>, anyhow::Error> {
    // let  a = role_type.to_string();
    let row: Option<UserRoleModel> = sqlx::query_as!(
        UserRoleModel,
        r#"SELECT id, role_name, role_status as "role_status!:Status", created_at, created_by, is_deleted from role where role_name  = $1"#,
        role_type.to_lowercase_string()    
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
#[tracing::instrument(name = "get_role", skip(pool))]
pub async fn get_role(pool: &PgPool, role_type: &UserType) -> Result<Option<AccountRole>, anyhow::Error> {
    let role_model = get_role_model(pool, role_type).await?;
    match role_model {
        Some(role) => {
            Ok(Some(AccountRole {
                id: role.id,
                role_name: role.role_name,
                role_status: role.role_status,
                is_deleted: role.is_deleted

            }))
        }
        None => Ok(None),
    }
}

#[tracing::instrument(name = "save user account role", skip(transaction))]
pub async fn save_user_role(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<Uuid, anyhow::Error> {
    let user_role_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO user_role (id, user_id, role_id, created_at, created_by)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        user_role_id,
        user_id,
        role_id,
        Utc::now(),
        user_id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure occured while saving user account role".to_string(),
            e.into(),
        )
    })?;
    Ok(user_role_id)
}


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


#[tracing::instrument(name = "prepare auth mechanism data", skip(user_id, user_account))]
pub async fn prepare_auth_mechanism_data_for_user_account(
    user_id: Uuid,
    user_account: &CreateUserAccount,
) -> Result<BulkAuthMechanismInsert, anyhow::Error> {
    let current_utc = Utc::now();
    let password = user_account.password.clone();
    let password_hash = spawn_blocking_with_tracing(move || {
        compute_password_hash(password)
    })
    .await?
    .context("Failed to hash password")?;

    // Prepare data for auth mechanism
    let id = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let user_id_list = vec![user_id, user_id, user_id];
    let auth_scope = vec![
        AuthenticationScope::Password,
        AuthenticationScope::Otp,
        AuthenticationScope::Email,
    ];
    let auth_identifier: Vec<&str> = vec![
        &user_account.username,
        &user_account.mobile_no,
        user_account.email.get(),
    ];
    let secret = vec![password_hash.expose_secret().to_string()];
    let is_active = vec![Status::Active, Status::Active, Status::Active];
    let created_on = vec![current_utc, current_utc, current_utc];
    let created_by = vec![user_id, user_id, user_id];
    let auth_context = vec![
        AuthContextType::UserAccount,
        AuthContextType::UserAccount,
        AuthContextType::UserAccount,
    ];

    Ok(BulkAuthMechanismInsert {
        id,
        user_id_list,
        auth_scope,
        auth_identifier,
        secret,
        is_active,
        created_on,
        created_by,
        auth_context,
    })
}

#[tracing::instrument(name = "save auth mechanism", skip(transaction, auth_data))]
pub async fn save_auth_mechanism(
    transaction: &mut Transaction<'_, Postgres>,
    auth_data: BulkAuthMechanismInsert<'_>,
) -> Result<(), anyhow::Error> {
    // Save data to auth mechanism table
    let query = sqlx::query!(
        r#"
        INSERT INTO auth_mechanism (id, user_id, auth_scope, auth_identifier, secret, auth_context, is_active, created_at, created_by)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::user_auth_identifier_scope[], $4::text[], $5::text[], $6::auth_context_type[], $7::status[], $8::TIMESTAMP[], $9::text[])
        ON CONFLICT (user_id, auth_scope, auth_context) DO NOTHING;
        "#,
        &auth_data.id[..] as &[Uuid],
        &auth_data.user_id_list[..] as &[Uuid],
        &auth_data.auth_scope[..] as &[AuthenticationScope],
        &auth_data.auth_identifier[..] as &[&str],
        &auth_data.secret[..],
        &auth_data.auth_context[..] as &[AuthContextType],
        &auth_data.is_active[..] as &[Status],
        &auth_data.created_on[..] as &[DateTime<Utc>],
        &auth_data.created_by[..] as &[Uuid]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure was encountered while saving auth mechanisms".to_string(),
            e.into(),
        )
    })?;

    Ok(())
}

#[tracing::instrument(name = "register user", skip(pool))]
pub async fn register_user(
    pool: &PgPool,
    user_account: CreateUserAccount,
    subscriber_id: String
   
) -> Result<uuid::Uuid, UserRegistrationError> {
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
    let uuid = save_user(&mut transaction, &user_account, subscriber_id).await?;
    let bulk_auth_data = prepare_auth_mechanism_data_for_user_account(uuid, &user_account).await?;
    save_auth_mechanism(&mut transaction, bulk_auth_data).await?;
    if  let Some(role_obj) = get_role(pool, &user_account.user_type).await?{
        if role_obj.is_deleted || role_obj.role_status == Status::Inactive {
            return Err(UserRegistrationError::InvalidRoleError("Role is deleted / Inactive".to_string()))
        }
        save_user_role(&mut transaction, uuid, role_obj.id).await?;
    }
    else{
        tracing::info!("Invalid Role for user account {}", uuid);
        
    }

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new user account.")?;

    Ok(uuid)

    // return Err(
    //     anyhow!("Duplicate mobile number")
    // ).map_err(UserRegistrationError::DuplicateEmail)?;
    // Ok(Uuid::new_v4())
}

#[tracing::instrument(name = "create user account")]
pub fn create_vector_from_business_account(
    business_account: &CreateBusinessAccount,
) -> Result<Vec<UserVector>, BusinessAccountError> {
    let mut vector_list = vec![
        UserVector {
            key: VectorType::Email,
            value: business_account.email.get().to_string(),
            masking: MaskingType::NA,
            verified: false,
        },
        UserVector {
            key: VectorType::MobileNo,
            value: business_account.mobile_no.to_string(),
            masking: MaskingType::NA,
            verified: false,
        }
    ];
    for proof in business_account.proofs.iter(){
        vector_list.push(
            UserVector {
                key: proof.key.to_owned(),
                value: proof.kyc_id.to_string(),
                masking: MaskingType::NA,
                verified: false,
            }
        )
    }
    return Ok(vector_list)
}

#[tracing::instrument(name = "create user business relation", skip(transaction))]
pub async fn save_business_account(transaction: &mut Transaction<'_, Postgres>, user_account: &UserAccount,  create_business_obj: &CreateBusinessAccount, subscriber_id: String) -> Result<uuid::Uuid,  BusinessAccountError>{
    let business_account_id = Uuid::new_v4();
    let business_account_number = create_business_obj.company_name.replace(' ', "-").to_lowercase();
    let vector_list = create_vector_from_business_account(create_business_obj)?;
    // let proofs = vec![];
    let query = sqlx::query!(
        r#"
        INSERT INTO business_account (id, business_account_number, alt_business_account_number, company_name, vectors, proofs, customer_type, merchant_type, trade, source, created_by,  created_at, subscriber_id, default_vector_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        business_account_id,
        business_account_number,
        business_account_number,
        create_business_obj.company_name,
        sqlx::types::Json(vector_list) as sqlx::types::Json<Vec<UserVector>>,
        sqlx::types::Json(&create_business_obj.proofs) as sqlx::types::Json<&Vec<KYCProof>>,
        &create_business_obj.customer_type as &CustomerType,
        &create_business_obj.merchant_type as &MerchantType,
        &create_business_obj.trade_type as &Vec<TradeType>,
        &create_business_obj.source as &DataSource,
        user_account.id,
        Utc::now(),
        subscriber_id, 
        &create_business_obj.default_vector_type.to_string()
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        BusinessAccountError::DatabaseError(
            "A database failure occured while saving business account".to_string(),
            e.into(),
        )
    })?;
    Ok(business_account_id)

    // Ok(Uuid::new_v4())
}
#[tracing::instrument(name = "create user business relation", skip(transaction))]
pub async fn save_user_business_relation(transaction: &mut Transaction<'_, Postgres>, user_id: Uuid, business_id: Uuid, role_id: Uuid)-> Result<uuid::Uuid,  BusinessAccountError>{
    let user_role_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO business_user_relationship (id, user_id, business_id, role_id, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_role_id,
        user_id,
        business_id,
        role_id,
        user_id,
        Utc::now(),
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        BusinessAccountError::DatabaseError(
            "A database failure occured while saving user business relation".to_string(),
            e.into(),
        )
    })?;
    Ok(user_role_id)
}


#[tracing::instrument(name = "Save Auth Mechanism for Business account")]
pub async fn prepare_auth_mechanism_data_for_business_account(
    user_id: Uuid,
    user_account: &CreateBusinessAccount,
) -> Result<BulkAuthMechanismInsert<'_>, anyhow::Error> {
    let current_utc = Utc::now();

    // Prepare data for auth mechanism
    let id = vec![Uuid::new_v4(), Uuid::new_v4()];
    let user_id_list = vec![user_id, user_id];
    let auth_scope = vec![
        AuthenticationScope::Otp,
        AuthenticationScope::Email,
    ];
    let auth_identifier: Vec<&str> = vec![
        &user_account.mobile_no,
        user_account.email.get(),
    ];
    let secret =vec![];
    let is_active = vec![Status::Active, Status::Active];
    let created_on = vec![current_utc, current_utc];
    let created_by = vec![user_id, user_id];
    let auth_context = vec![
        AuthContextType::BusinessAccount,
        AuthContextType::BusinessAccount
    ];

    Ok(BulkAuthMechanismInsert {
        id,
        user_id_list,
        auth_scope,
        auth_identifier,
        secret,
        is_active,
        created_on,
        created_by,
        auth_context,
    })

    
}
#[tracing::instrument(name = "create business account", skip(pool))]
pub async fn create_business_account( pool: &PgPool, user_account: &UserAccount, create_business_obj: &CreateBusinessAccount, subscriber_id: String) -> Result<uuid::Uuid, BusinessAccountError>{
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let business_account_id = save_business_account(&mut transaction, user_account, create_business_obj, subscriber_id).await?;
    if  let Some(role_obj) = get_role(pool, &UserType::Admin).await?{
        if role_obj.is_deleted || role_obj.role_status == Status::Inactive {
            return Err(BusinessAccountError::InvalidRoleError("Role is deleted / Inactive".to_string()))
        }
        save_user_business_relation(&mut transaction, user_account.id, business_account_id, role_obj.id).await?;
    }
    else{
        tracing::info!("Invalid role for business account");
        
    }

    let bulk_auth_data = prepare_auth_mechanism_data_for_business_account(user_account.id, create_business_obj).await?;
    save_auth_mechanism(&mut transaction, bulk_auth_data).await?;


    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new business account.")?;
    Ok(Uuid::new_v4())

}


#[tracing::instrument(name = "Get Business Account By User id", skip(pool))]
pub async fn fetch_business_account_model_by_user_account(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<BusinessAccountModel>, anyhow::Error> {

    let row: Vec<BusinessAccountModel> = sqlx::query_as!(
        BusinessAccountModel,
        r#"SELECT 
        ba.id, ba.company_name, ba.customer_type as "customer_type: CustomerType", vectors as "vectors!:sqlx::types::Json<Vec<UserVector>>", ba.is_active as "is_active: Status",  ba.kyc_status as "kyc_status: KycStatus"
         FROM business_user_relationship as bur
            INNER JOIN business_account ba ON bur.business_id = ba.id
        WHERE bur.user_id = $1
        "#,
        user_id
    ).fetch_all(pool)
    .await?;

    Ok(row)
}

#[tracing::instrument(name = "Get Business Account By User Id and customer type", skip(pool))]
pub async fn fetch_business_account_model_by_customer_type(
    user_id: Uuid,
    business_account_id: Uuid,
    customer_type_list: Vec<CustomerType>,
    pool: &PgPool,
) -> Result<Option<UserBusinessRelationAccountModel>, anyhow::Error> {
    // let a  = serde_json::to_vec_pretty(&customer_type_list);
    //let a: String  = serde_json::from_value(serde_json::to_value(customer_type_list[0]).unwrap()).unwrap();
    let val_list: Vec<String> = customer_type_list.iter().map(|&s| s.to_string()).collect();
    //let val_list: Vec<String> = customer_type_list.iter().map(|&s| s.to_string()).collect();
    let row: Option<UserBusinessRelationAccountModel> = sqlx::query_as!(
        UserBusinessRelationAccountModel,
        r#"SELECT 
        ba.id, ba.company_name, ba.customer_type as "customer_type: CustomerType", vectors as "vectors!:sqlx::types::Json<Vec<UserVector>>", ba.is_active as "is_active: Status", 
        ba.kyc_status as "kyc_status: KycStatus", bur.verified, ba.is_deleted, ba.default_vector_type as "default_vector_type: VectorType" FROM business_user_relationship as bur
            INNER JOIN business_account ba ON bur.business_id = ba.id
        WHERE bur.user_id = $1 AND bur.business_id= $2 AND ba.customer_type::text = ANY($3)
        "#,
        user_id,
        business_account_id,
        &val_list
    ).fetch_optional(pool)
    .await?;

    Ok(row)
}

// let val_list: Vec<String> = value_list.iter().map(|&s| s.to_string()).collect();

// let row: Option<UserAccountModel> = sqlx::query_as!(
//     UserAccountModel,
//     r#"SELECT 
//         ua.id, username, is_test_user, mobile_no, email, is_active as "is_active!:Status", 
//         vectors as "vectors!:sqlx::types::Json<Vec<Option<UserVectors>>>", display_name, 
//         international_dialing_code, user_account_number, alt_user_account_number, ua.is_deleted, r.role_name FROM user_account as ua
//         INNER JOIN user_role ur ON ua.id = ur.user_id
//         INNER JOIN role r ON ur.role_id = r.id
//     WHERE ua.email = ANY($1) OR ua.mobile_no = ANY($1) OR ua.id::text = ANY($1)


#[tracing::instrument(name = "Get Business Account from model")]
pub fn get_business_account_from_model(business_model: &UserBusinessRelationAccountModel) -> Result<BusinessAccount, anyhow::Error> {
    return Ok(BusinessAccount {
        id: business_model.id,
        company_name: business_model.company_name.to_string(),
        vectors: business_model.vectors.0.to_owned(),
        kyc_status: business_model.kyc_status.to_owned(),
        is_active: business_model.is_active.to_owned(),
        is_deleted: business_model.is_deleted,
        verified: business_model.verified,
        default_vector_type: business_model.default_vector_type.to_owned()
        
    });
}


#[tracing::instrument(name = "Get Business Account by Customer Type")]
pub async fn get_business_account_by_customer_type(user_id: Uuid, business_account_id: Uuid, customer_type_list: Vec<CustomerType>, pool: &PgPool) -> Result<Option<BusinessAccount>, anyhow::Error> {
    let  business_account_model = fetch_business_account_model_by_customer_type(user_id, business_account_id, customer_type_list, pool).await?;
    match business_account_model {
        Some(model) => {
            let business_account = get_business_account_from_model(&model)?;
            Ok(Some(business_account))
        },
        None => Ok(None),
    }
    
}

#[tracing::instrument(name = "Get Basic Business Account from Business Model")]
pub fn get_basic_business_account_from_model(business_model: &BusinessAccountModel) -> Result<BasicBusinessAccount, anyhow::Error> {
    return Ok(BasicBusinessAccount {
        id: business_model.id,
        company_name: business_model.company_name.to_string(),
        customer_type: business_model.customer_type
    });
}



#[tracing::instrument(name = "Get Basic Business account by user id")]
pub async fn get_basic_business_account_by_user_id(user_id: Uuid, pool: &PgPool) -> Result<Vec<BasicBusinessAccount>, anyhow::Error> {
    let business_account_models =  fetch_business_account_model_by_user_account(user_id, pool).await?;
    let mut business_account_list = Vec::new();
    for business_account_model in business_account_models.iter(){
        let business_account = get_basic_business_account_from_model(business_account_model)?;
        business_account_list.push(business_account)
    }

    Ok(business_account_list)

}


#[tracing::instrument(name = "Get Auth Data")]
pub async fn get_auth_data(
    pool: &PgPool,
    user_model: UserAccountModel,
    jwt_obj: &JWT,
) -> Result<AuthData, anyhow::Error> {
    let user_account = get_user_account_from_model(user_model)?;
    let business_obj = get_basic_business_account_by_user_id(user_account.id, pool).await?;
    let user_id = user_account.id;
    let token = generate_jwt_token_for_user(user_id, jwt_obj.expiry, &jwt_obj.secret)?;

    Ok(AuthData {
        user: user_account,
        token,
        business_account_list: business_obj,
    })
}

pub fn validate_business_account_active(business_obj: &BusinessAccount) -> Option<String>{
    match (
        &business_obj.kyc_status,
        &business_obj.is_active,
        business_obj.is_deleted,
        business_obj.verified,
    ) {
        (KycStatus::Pending, _, _, _) => Some("KYC is still pending".to_string()),
        (KycStatus::OnHold, _, _, _) => Some("KYC is On-hold".to_string()),
        (KycStatus::Rejected, _, _, _) => Some("KYC is Rejected".to_string()),
        (_, Status::Inactive, _, _) => Some("Business Account is inactive".to_string()),
        (_, _, true, _) => Some("Business Account is deleted".to_string()),
        (_, _, _, false) => Some("Business User relation is not verified".to_string()),
        _ => None 
    }
}


pub fn get_default_vector_value<'a>(
    default_vector_type: &'a VectorType,
    vectors: &'a Vec<UserVector>,
) -> Option<&'a str> {
    for vector in vectors {
        if vector.key == *default_vector_type {
            return Some(&vector.value);
        }
    }
    None
}
