use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;
use uuid::Uuid;

use super::utils::serialize_timestamp_without_nanos;
use crate::{
    routes::product::schemas::CategoryDomain,
    schemas::{CountryCode, ONDCNetworkType},
    utils::pascal_to_snake_case,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCVersion {
    #[serde(rename = "2.0.2")]
    V2point2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ONDCActionType {
    Search,
    OnSearch, //Working on it.
    Select,
    OnSelect, // Pending
    Init,
    OnInit, // Pending
    Confirm,
    OnConfirm, // Pending
    Cancel,
    OnCancel, // Pending
    Status,
    OnStatus, // Pending
    Track,
    OnTrack, // Pending
    Update,
    OnUpdate, // Pending
    Issue,
    OnIssue, // Pending
    IssueStatus,
    OnIssueStatus, // Pending
}

impl Display for ONDCActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
pub enum ONDCDomain {
    #[serde(rename = "ONDC:RET10")]
    #[sqlx(rename = "ONDC:RET10")]
    Grocery,
    #[serde(rename = "ONDC:RET12")]
    #[sqlx(rename = "ONDC:RET12")]
    Fashion,
    #[serde(rename = "ONDC:RET13")]
    #[sqlx(rename = "ONDC:RET13")]
    Bpc,
    #[serde(rename = "ONDC:RET14")]
    #[sqlx(rename = "ONDC:RET14")]
    Electronics,
    #[serde(rename = "ONDC:RET15")]
    #[sqlx(rename = "ONDC:RET15")]
    Appliances,
    #[serde(rename = "ONDC:RET16")]
    #[sqlx(rename = "ONDC:RET16")]
    HomeAndKitchen,
    #[serde(rename = "ONDC:RET1A")]
    #[sqlx(rename = "ONDC:RET1A")]
    AutoComponentsAndAccessories,
    #[serde(rename = "ONDC:RET1B")]
    #[sqlx(rename = "ONDC:RET1B")]
    HardwareAndIndustrialEquipments,
    #[serde(rename = "ONDC:RET1C")]
    #[sqlx(rename = "ONDC:RET1C")]
    BuildingAndConstructionSupplies,
}

impl ONDCDomain {
    pub fn get_category_domain(&self) -> CategoryDomain {
        match self {
            ONDCDomain::Grocery => CategoryDomain::Grocery,
            ONDCDomain::Fashion => CategoryDomain::Fashion,
            ONDCDomain::Bpc => CategoryDomain::Bpc,
            ONDCDomain::Electronics => CategoryDomain::Electronics,
            ONDCDomain::Appliances => CategoryDomain::Appliances,
            ONDCDomain::HomeAndKitchen => CategoryDomain::HomeAndKitchen,
            ONDCDomain::AutoComponentsAndAccessories => {
                CategoryDomain::AutoComponentsAndAccessories
            }
            ONDCDomain::HardwareAndIndustrialEquipments => {
                CategoryDomain::HardwareAndIndustrialEquipments
            }
            ONDCDomain::BuildingAndConstructionSupplies => {
                CategoryDomain::BuildingAndConstructionSupplies
            }
        }
    }
}

impl Display for ONDCDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ONDC:{}", self.get_category_domain())
    }
}

impl ONDCDomain {
    pub fn get_ondc_domain(domain_category_code: &CategoryDomain) -> Self {
        match domain_category_code {
            CategoryDomain::Grocery => ONDCDomain::Grocery,
            CategoryDomain::Fashion => ONDCDomain::Fashion,
            CategoryDomain::Bpc => ONDCDomain::Bpc,
            CategoryDomain::Electronics => ONDCDomain::Electronics,
            CategoryDomain::Appliances => ONDCDomain::Appliances,
            CategoryDomain::HomeAndKitchen => ONDCDomain::HomeAndKitchen,
            CategoryDomain::AutoComponentsAndAccessories => {
                ONDCDomain::AutoComponentsAndAccessories
            }
            CategoryDomain::HardwareAndIndustrialEquipments => {
                ONDCDomain::HardwareAndIndustrialEquipments
            }
            CategoryDomain::BuildingAndConstructionSupplies => {
                ONDCDomain::BuildingAndConstructionSupplies
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContextCountry {
    pub code: CountryCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContextCity {
    pub code: String,
}

impl ONDCContextCity {
    fn _default() -> Self {
        ONDCContextCity {
            code: "std:080".to_string(),
        }
    }
}

impl ONDCContextCountry {
    fn _default() -> Self {
        ONDCContextCountry {
            code: CountryCode::IND,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContextLocation {
    pub city: ONDCContextCity,
    pub country: ONDCContextCountry,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContext {
    pub domain: ONDCDomain,
    pub location: ONDCContextLocation,
    pub action: ONDCActionType,
    pub version: ONDCVersion,
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub timestamp: DateTime<Utc>,
    pub bap_id: String,
    pub bap_uri: String,
    pub bpp_id: Option<String>,
    pub bpp_uri: Option<String>,
    pub ttl: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ONDCResponseStatusType {
    Ack,
    Nack,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCGateWayErrorCode {
    #[serde(rename = "10000")]
    GateWayInvalidRequest,
    #[serde(rename = "10001")]
    GateWayInvalidSignature,
    #[serde(rename = "10002")]
    GateWayInvalidCityCode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCBuyerErrorCode {
    #[serde(rename = "23001")]
    InternalErrorCode,
    #[serde(rename = "20008")]
    ResponseSequenceCode,
    #[serde(rename = "20001")]
    InvalidSignatureCode,
    #[serde(rename = "20002")]
    StaleRequestCode,
    #[serde(rename = "20006")]
    InvalidResponseCode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCSellerErrorCode {
    #[serde(rename = "30016")]
    SellerInvalidSignatureCode,
    #[serde(rename = "30022")]
    SellerStaleRequestCode,
    #[serde(rename = "30000")]
    SellerInvalidRequestCode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum LookUpErrorCode {
    #[serde(rename = "151")]
    InvalidRequestCode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ONDCErrorCode {
    GatewayError(ONDCGateWayErrorCode),
    BuyerError(ONDCBuyerErrorCode),
    SellerError(ONDCSellerErrorCode),
    LookUpError(LookUpErrorCode),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ONDErrorType {
    #[serde(rename = "Gateway")]
    Gateway,
    ContextError,
    DomainError,
    PolicyError,
    JsonSchemaError,
    CoreError,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ONDCResponseAck {
    pub status: ONDCResponseStatusType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ONDCResponseMessage {
    pub ack: ONDCResponseAck,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCResponseErrorBody<D> {
    pub r#type: ONDErrorType,
    pub code: D,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ONDCResponse<D> {
    // pub context: Option<ONDCContext>,
    pub message: ONDCResponseMessage,
    pub error: Option<ONDCResponseErrorBody<D>>,
}

impl<D> ONDCResponse<D> {
    pub fn successful_response(_context: Option<ONDCContext>) -> Self {
        Self {
            message: ONDCResponseMessage {
                ack: ONDCResponseAck {
                    status: ONDCResponseStatusType::Ack,
                },
            },
            // context: context,
            error: None,
        }
    }

    pub fn error_response(_context: Option<ONDCContext>, error: ONDCResponseErrorBody<D>) -> Self {
        Self {
            message: ONDCResponseMessage {
                ack: ONDCResponseAck {
                    status: ONDCResponseStatusType::Nack,
                },
            },
            // context: context,
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum OndcUrl {
    #[serde(rename = "/lookup")]
    LookUp,
}

impl Display for OndcUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "/{}",
            match self {
                OndcUrl::LookUp => "lookup",
            }
        )
    }
}

#[derive(Debug, Serialize)]
pub struct LookupRequest<'a> {
    pub subscriber_id: &'a str,
    pub domain: &'a ONDCDomain,
    pub r#type: &'a ONDCNetworkType,
}

#[derive(Debug, Deserialize)]
pub struct LookupData {
    pub br_id: String,
    pub subscriber_id: String,
    pub signing_public_key: String,
    pub subscriber_url: String,
    pub encr_public_key: String,
    #[serde(rename = "ukId")]
    pub uk_id: String,
    pub domain: ONDCDomain,
    pub r#type: ONDCNetworkType,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ONDCItemUOM {
    Unit,
    Dozen,
    Gram,
    Kilogram,
    Tonne,
    Litre,
    Millilitre,
}
