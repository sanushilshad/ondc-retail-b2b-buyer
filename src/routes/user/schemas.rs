use crate::domain::{subscriber_email::deserialize_subscriber_email, EmailObject};
use crate::errors::GenericError;
use crate::routes::ondc::buyer::schemas::ONDCBuyerIdType;
use crate::schemas::{KycStatus, Status};
use crate::utils::pascal_to_snake_case;

use actix_web::{FromRequest, HttpMessage};
use anyhow::anyhow;
use chrono::{DateTime, NaiveTime, Utc};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgHasArrayType;
use std::fmt::{self, Debug};
use std::future::{ready, Ready};
use utoipa::ToSchema;
use uuid::Uuid;

// macro_rules! impl_serialize_format {
//     ($struct_name:ident, $trait_name:path) => {
//         impl $trait_name for $struct_name {
//             fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                 fmt_json(self, f)
//             }
//         }
//     };
// }

// impl_serialize_format!(AuthenticateRequest, Display);
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub scope: AuthenticationScope,
    pub identifier: String,
    // #[serde(with = "SecretString")]
    #[schema(value_type = String)]
    pub secret: Secret<String>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserType {
    Guest,
    User,
    Member,
    Agent,
    Superadmin,
    Admin,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl UserType {
    pub fn to_lowercase_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CreateUserType {
    Guest,
    User,
    Member,
    Agent,
}
impl From<CreateUserType> for UserType {
    fn from(create_user_type: CreateUserType) -> Self {
        match create_user_type {
            CreateUserType::Guest => UserType::Guest,
            CreateUserType::User => UserType::User,
            CreateUserType::Member => UserType::Member,
            CreateUserType::Agent => UserType::Agent,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "user_auth_identifier_scope", rename_all = "snake_case")]
pub enum AuthenticationScope {
    Otp,
    Password,
    Google,
    Facebook,
    Microsoft,
    Apple,
    Token,
    AuthApp,
    Qr,
    Email,
}

impl PgHasArrayType for AuthenticationScope {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_user_auth_identifier_scope")
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserAccount {
    pub username: String,
    pub mobile_no: String,
    pub international_dialing_code: String,
    #[schema(value_type = String)]
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_subscriber_email")]
    pub email: EmailObject, //NOTE: email_address crate can be used if needed,
    pub display_name: String,
    pub is_test_user: bool,
    pub user_type: UserType,
    pub source: DataSource,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, ToSchema)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MaskingType {
    NA,
    Encrypt,
    PartialMask,
    FullMask,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, Clone, ToSchema)]
#[sqlx(type_name = "vector_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VectorType {
    PanCardNo,
    Gstin,
    AadhaarCardNo,
    MobileNo,
    Email,
    InternationalDialingCode,
    UpiId,
    BankAccountNumber,
    IfscCode,
    LicenseNumber,
    PassportNo,
    VoterIdNo,
    Ssn,
    Tin,
    ExportLicenseNo,
    FssaiLicenseNumber,
}

impl std::fmt::Display for VectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     let s = match self {
    //         VectorType::PanCardNo => "pan_card_no",
    //         VectorType::Gstin => "gstin",
    //         VectorType::AadhaarCardNo => "aadhaar_card_no",
    //         VectorType::MobileNo => "mobile_no",
    //         VectorType::Email => "email",
    //         VectorType::InternationalDialingCode => "international_dialing_code",
    //         VectorType::UpiId => "upi_id",
    //         VectorType::BankAccountNumber => "bank_account_number",
    //         VectorType::IfscCode => "ifsc_code",
    //         VectorType::LicenseNumber => "license_number",
    //         VectorType::PassportNo => "passport_no",
    //         VectorType::VoterIdNo => "voter_id_no",
    //         VectorType::Ssn => "ssn",
    //         VectorType::Tin => "tin",
    //         VectorType::ExportLicenseNo => "export_license_no",
    //         VectorType::FssaiLicenseNumber => "fssai_license_number",
    //     };
    //     write!(f, "{}", s)
    // }
}

impl VectorType {
    pub fn get_ondc_vector_type(&self) -> Result<ONDCBuyerIdType, anyhow::Error> {
        match self {
            VectorType::PanCardNo => Ok(ONDCBuyerIdType::Pan),
            VectorType::Gstin => Ok(ONDCBuyerIdType::Gst),
            VectorType::Tin => Ok(ONDCBuyerIdType::Tin),
            VectorType::AadhaarCardNo => Ok(ONDCBuyerIdType::Aadhaar),
            VectorType::MobileNo => Ok(ONDCBuyerIdType::Mobile),
            VectorType::Email => Ok(ONDCBuyerIdType::Email),
            _ => Err(anyhow!("Invalid User Vectory Mapping")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, ToSchema)]
#[sqlx(type_name = "vectors")]
pub struct UserVector {
    pub key: VectorType,
    pub value: String,
    pub masking: MaskingType,
    pub verified: bool,
}

impl PgHasArrayType for UserVector {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_vectors")
    }
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct UserAccount {
    #[schema(value_type = String)]
    pub id: Uuid,
    pub username: String,
    pub mobile_no: String,
    pub email: String,
    pub is_active: Status,
    pub display_name: String,
    pub vectors: Vec<Option<UserVector>>,
    pub international_dialing_code: String,
    pub user_account_number: String,
    pub alt_user_account_number: String,
    pub is_test_user: bool,
    pub is_deleted: bool,
    pub user_role: String,
}

impl FromRequest for UserAccount {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<UserAccount>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedCustomError(
                "Something went wrong while parsing user account detail".to_string(),
            )),
        };

        ready(result)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BasicBusinessAccount {
    pub company_name: String,
    #[schema(value_type = String)]
    pub id: Uuid,
    pub customer_type: CustomerType,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthData {
    pub user: UserAccount,
    #[serde(serialize_with = "round_serialize")]
    #[schema(value_type = String)]
    pub token: Secret<String>,
    pub business_account_list: Vec<BasicBusinessAccount>,
}

fn round_serialize<S>(x: &Secret<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(x.expose_secret())
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "auth_context_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthContextType {
    UserAccount,
    BusinessAccount,
}

impl PgHasArrayType for AuthContextType {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_auth_context_type")
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AuthMechanism {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Option<Secret<String>>,
    pub is_active: Status,
    pub valid_upto: Option<DateTime<Utc>>,
    pub auth_context: AuthContextType,
}

#[allow(dead_code)]
pub struct AccountRole {
    pub id: Uuid,
    pub role_name: String,
    pub role_status: Status,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, Copy, Clone, ToSchema)]
#[sqlx(type_name = "customer_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CustomerType {
    NA,
    Buyer,
    Seller,
    Brand,
    LogisticPartner,
    PaymentAggregator,
    VirtualOperator,
    ExternalPartner,
}
impl std::fmt::Display for CustomerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            CustomerType::NA => "NA",
            CustomerType::Buyer => "buyer",
            CustomerType::Seller => "seller",
            CustomerType::Brand => "brand",
            CustomerType::LogisticPartner => "logistic_partner",
            CustomerType::PaymentAggregator => "payment_aggregator",
            CustomerType::VirtualOperator => "virtual_operator",
            CustomerType::ExternalPartner => "external_partner",
        };
        write!(f, "{}", display_str)
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "data_source", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    PlaceOrder,
    Ondc,
    Rapidor,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "trade_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TradeType {
    Domestic,
    Export,
}

impl PgHasArrayType for TradeType {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_trade_type")
    }
}
#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "merchant_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MerchantType {
    Fpo,
    Manufacturer,
    Restaurant,
    Grocery,
    Mall,
    Supermart,
    Store,
    Other,
}

#[derive(Debug, Serialize)]
pub struct BulkAuthMechanismInsert<'a> {
    pub id: Vec<Uuid>,
    pub user_id_list: Vec<Uuid>,
    pub auth_scope: Vec<AuthenticationScope>,
    #[serde(borrow)]
    pub auth_identifier: Vec<&'a str>,
    pub secret: Vec<String>,
    pub is_active: Vec<Status>,
    pub created_on: Vec<DateTime<Utc>>,
    pub created_by: Vec<Uuid>,
    pub auth_context: Vec<AuthContextType>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KYCProof {
    pub key: VectorType,
    pub kyc_id: String,
    pub value: Vec<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBusinessAccount {
    pub company_name: String,
    pub is_test_account: bool,
    pub customer_type: CustomerType,
    pub source: DataSource,
    pub mobile_no: String,
    pub email: EmailObject,
    pub trade_type: Vec<TradeType>,
    pub merchant_type: MerchantType,
    pub opening_time: Option<NaiveTime>,
    pub closing_time: Option<NaiveTime>,
    pub proofs: Vec<KYCProof>,
    pub default_vector_type: VectorType,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BusinessAccount {
    pub id: Uuid,
    pub company_name: String,
    pub vectors: Vec<UserVector>,
    pub kyc_status: KycStatus,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
    pub default_vector_type: VectorType,
}

impl FromRequest for BusinessAccount {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<BusinessAccount>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedError(anyhow!(
                "Something went wrong while parsing Business Account data".to_string()
            ))),
        };

        ready(result)
    }
}
