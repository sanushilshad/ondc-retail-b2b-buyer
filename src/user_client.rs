use crate::schemas::GenericResponse;
use anyhow::anyhow;
use reqwest::{Client, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessFetchRequest {
    id: Uuid,
    customer_type_list: Vec<CustomerType>,
}

impl BusinessFetchRequest {
    fn new(id: Uuid, customer_type_list: Vec<CustomerType>) -> Self {
        Self {
            id,
            customer_type_list,
        }
    }
}

use crate::routes::ondc::schemas::ONDCBuyerIdType;
use crate::schemas::{KycStatus, Status};

use crate::errors::GenericError;
use crate::utils::pascal_to_snake_case;
use actix_web::{FromRequest, HttpMessage};
use serde::Deserialize;
use std::fmt::Debug;
use std::future::{ready, Ready};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
    Guest,
    User,
    Member,
    Agent,
    Superadmin,
    Admin,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MaskingType {
    NA,
    Encrypt,
    PartialMask,
    FullMask,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum PermissionType {
    #[serde(rename = "create:order")]
    CreateOrder,
    #[serde(rename = "create:order:self")]
    CreateOrderSelf,
    #[serde(rename = "update:order:self")]
    UpdateOrderSelf,
    #[serde(rename = "update:order")]
    UpdateOrder,
    #[serde(rename = "cancel:order")]
    CancelOrder,
    #[serde(rename = "cancel:order:self")]
    CancelOrderSelf,
    #[serde(rename = "read:order:self")]
    ReadOrderSelf,
    #[serde(rename = "list:order:self")]
    ListOrderSelf,
    #[serde(rename = "read:order")]
    ReadOrder,
    #[serde(rename = "list:order")]
    ListOrder,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
    ImportLicenseNo,
}

impl std::fmt::Display for VectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
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

pub trait VectorHasKey {
    fn key(&self) -> &VectorType;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserVector {
    pub key: VectorType,
    pub value: String,
    pub masking: MaskingType,
    pub verified: bool,
}
impl VectorHasKey for UserVector {
    fn key(&self) -> &VectorType {
        &self.key
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccount {
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CustomerType {
    RetailB2bBuyer,
    RetailB2bSeller,
    LogisticB2bSeller,
    LogisticB2bBuyer,
    CreditBuyer,
    CreditSeller,
    PaymentAggregator,
    VirtualOperator,
    ExternalPartner,
}

impl std::fmt::Display for CustomerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proof {
    pub key: VectorType,
    pub kyc_id: String,
    pub value: Vec<String>,
}

impl VectorHasKey for Proof {
    fn key(&self) -> &VectorType {
        &self.key
    }
}

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
    pub proofs: Vec<Proof>,
    pub subscriber_id: String,
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
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SettingKey {
    OrderNoPrefix,
    PaymentServiceId,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FetchSettingRequest {
    pub keys: Vec<SettingKey>,
}

impl FetchSettingRequest {
    fn new(keys: Vec<SettingKey>) -> Self {
        Self { keys }
    }
}

#[derive(Deserialize, Debug)]
pub struct SettingValue {
    pub value: String,
    pub id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct Setting {
    pub key: SettingKey,
    pub label: String,
    pub enum_id: Option<Uuid>,
    pub is_editable: bool,
    pub global_level: Vec<SettingValue>,
    pub user_level: Vec<SettingValue>,
    pub business_level: Vec<SettingValue>,
}

impl Setting {
    pub fn compute_setting(&self) -> Option<String> {
        if !self.user_level.is_empty() {
            return self.user_level.first().map(|obj| obj.value.to_owned());
        }
        if !self.business_level.is_empty() {
            return self.business_level.first().map(|obj| obj.value.to_owned());
        }
        if !self.global_level.is_empty() {
            return self.global_level.first().map(|obj| obj.value.to_owned());
        }
        None
    }
}

#[derive(Deserialize, Debug)]
pub struct SettingData {
    pub settings: Vec<Setting>,
}

impl SettingData {
    pub fn get_setting(&self, key: SettingKey) -> Option<String> {
        self.settings
            .iter()
            .find(|setting| setting.key == key)
            .and_then(|setting| setting.compute_setting())
    }
}

#[derive(Debug)]
pub struct UserClient {
    http_client: Client,
    base_url: String,
    auth_token: SecretString,
}

impl UserClient {
    #[tracing::instrument]
    pub fn new(base_url: String, auth_token: SecretString, timeout: std::time::Duration) -> Self {
        tracing::info!("Establishing connection to the user server.");
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            auth_token,
        }
    }
    fn get_auth_token(&self, user_auth_token: Option<&SecretString>) -> String {
        format!(
            "Bearer {}",
            user_auth_token.unwrap_or(&self.auth_token).expose_secret()
        )
    }

    #[tracing::instrument]
    pub async fn get_user_account(
        &self,
        user_auth_token: Option<&SecretString>,
        user_id: Option<Uuid>,
    ) -> Result<UserAccount, GenericError> {
        let url = format!("{}/user/fetch", self.base_url);
        let mut request = self
            .http_client
            .post(&url)
            .header("Authorization", self.get_auth_token(user_auth_token))
            .header("x-request-id", "internal".to_string())
            .header("x-device-id", "internal".to_string());

        if user_auth_token.is_none() {
            if let Some(user_id) = user_id {
                request = request.header("x-user-id", user_id.to_string())
            }
        }

        let response = request
            .send()
            .await
            .map_err(|err| GenericError::UnexpectedError(anyhow!("Request error: {}", err)))?;

        let status = response.status();
        let response_body: GenericResponse<UserAccount> = response.json().await.map_err(|err| {
            GenericError::SerializationError(format!("Failed to parse response: {}", err))
        })?;

        if status.is_success() {
            response_body
                .data
                .ok_or_else(|| GenericError::DataNotFound("User account not found".to_string()))
        } else {
            let error_message = match status {
                StatusCode::BAD_REQUEST => {
                    GenericError::ValidationError(response_body.customer_message)
                }
                StatusCode::GONE => GenericError::DataNotFound(response_body.customer_message),
                _ => GenericError::UnexpectedCustomError(response_body.customer_message),
            };
            Err(error_message)
        }
    }
    #[tracing::instrument]
    pub async fn get_business_account(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        customer_type_list: Vec<CustomerType>,
    ) -> Result<Option<BusinessAccount>, GenericError> {
        let url = format!("{}/business/fetch", self.base_url);
        let body = BusinessFetchRequest::new(business_id, customer_type_list);

        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .header("Authorization", self.get_auth_token(None))
            .header("x-user-id", user_id.to_string())
            .header("x-request-id", "internal".to_string())
            .header("x-device-id", "internal".to_string())
            .send()
            .await
            .map_err(|err| GenericError::UnexpectedError(anyhow!("Request error: {}", err)))?;

        let status = response.status();
        let response_body: GenericResponse<BusinessAccount> =
            response.json().await.map_err(|err| {
                GenericError::SerializationError(format!("Failed to parse response: {}", err))
            })?;

        if status.is_success() {
            Ok(response_body.data)
        } else {
            let error_message = match status {
                StatusCode::BAD_REQUEST => {
                    GenericError::ValidationError(response_body.customer_message)
                }
                StatusCode::GONE => GenericError::DataNotFound(response_body.customer_message),
                _ => GenericError::UnexpectedCustomError(response_body.customer_message),
            };
            Err(error_message)
        }
    }

    #[tracing::instrument]
    pub async fn permission_validation(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        permission_list: Vec<PermissionType>,
    ) -> Result<Vec<PermissionType>, GenericError> {
        let url = format!("{}/business/permission", self.base_url);
        let body = BusinessPermissionRequest::new(business_id, permission_list);

        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .header("Authorization", self.get_auth_token(None))
            .header("x-user-id", user_id.to_string())
            .header("x-request-id", "internal".to_string())
            .header("x-device-id", "internal".to_string())
            .send()
            .await
            .map_err(|err| GenericError::UnexpectedError(anyhow!("Request error: {}", err)))?;

        let status = response.status();
        let response_body: GenericResponse<Vec<PermissionType>> =
            response.json().await.map_err(|err| {
                GenericError::SerializationError(format!("Failed to parse response: {}", err))
            })?;
        if status.is_success() {
            Ok(response_body.data.unwrap())
        } else {
            let error_message = match status {
                StatusCode::BAD_REQUEST => {
                    GenericError::ValidationError(response_body.customer_message)
                }
                StatusCode::FORBIDDEN => {
                    GenericError::InsufficientPrevilegeError(response_body.customer_message)
                }
                StatusCode::GONE => GenericError::DataNotFound(response_body.customer_message),
                _ => GenericError::UnexpectedCustomError(response_body.customer_message),
            };
            return Err(error_message);
        }
    }

    #[tracing::instrument]
    pub async fn fetch_setting(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        keys: Vec<SettingKey>,
    ) -> Result<SettingData, anyhow::Error> {
        let url = format!("{}/setting/fetch", self.base_url);
        let body = FetchSettingRequest::new(keys);
        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .header("Authorization", self.get_auth_token(None))
            .header("x-user-id", user_id.to_string())
            .header("x-request-id", "internal".to_string())
            .header("x-device-id", "internal".to_string())
            .header("x-business-id", business_id.to_string())
            .header("x-customer-type", CustomerType::RetailB2bBuyer.to_string())
            .send()
            .await
            .map_err(|err| GenericError::UnexpectedError(anyhow!("Request error: {}", err)))?;

        let status = response.status();
        let response_body: GenericResponse<SettingData> = response.json().await.map_err(|err| {
            GenericError::SerializationError(format!("Failed to parse response: {}", err))
        })?;
        if status.is_success() {
            Ok(response_body.data.unwrap())
        } else {
            return Err(anyhow!(response_body.customer_message));
        }
    }
}

pub fn get_vector_val_from_list<'a, T>(vector_type: &'a VectorType, items: &'a [T]) -> Option<&'a T>
where
    T: VectorHasKey,
{
    items.iter().find(|&item| item.key() == vector_type)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessPermissionRequest {
    business_id: Uuid,
    action_list: Vec<PermissionType>,
}

impl BusinessPermissionRequest {
    fn new(business_id: Uuid, action_list: Vec<PermissionType>) -> Self {
        Self {
            business_id,
            action_list,
        }
    }
}
#[derive(Debug, Clone)]
pub struct AllowedPermission {
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub permission_list: Vec<PermissionType>,
}

impl AllowedPermission {
    pub fn new(user_id: Uuid, business_id: Uuid, permission_list: Vec<PermissionType>) -> Self {
        AllowedPermission {
            user_id,
            business_id,
            permission_list,
        }
    }

    pub fn validate_commerce_self(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        permission_type: PermissionType,
    ) -> bool {
        if !self.permission_list.contains(&permission_type) {
            return true;
        }

        self.user_id == user_id && self.business_id == business_id
    }
}

impl FromRequest for AllowedPermission {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<AllowedPermission>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedError(anyhow!(
                "Something went wrong while parsing allowed_permission data".to_string()
            ))),
        };

        ready(result)
    }
}
