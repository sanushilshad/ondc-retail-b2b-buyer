use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

use crate::{errors::GenericError, general_utils::pascal_to_snake_case, schemas::CountryCode};

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
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCDomain {
    #[serde(rename = "ONDC:RET10")]
    Grocery,
}

// impl FromStr for ONDCDomain {
//     type Err = GenericError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "ONDC:RET10" => Ok(ONDCDomain::Grocery),
//             _ => Err(GenericError::ValidationStringError(
//                 "Invalid Domain".to_string(),
//             )),
//         }
//     }
// }

impl ONDCDomain {
    pub fn get_ondc_domain(domain_category_code: &str) -> Result<ONDCDomain, GenericError> {
        // serde_json::from_str(&format!("ONDC:{}", domain_category_code))
        // domain_category_code.parse::<ONDCDomain>()
        match domain_category_code {
            "RET10" => Ok(ONDCDomain::Grocery),
            _ => Err(GenericError::ValidationStringError(
                "Invalid domain category code".to_owned(),
            )),
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
#[derive(Debug, Deserialize)]
pub enum ONDCResponseErrorCode {
    #[serde(rename = "10000")]
    InvalidRequest,
    #[serde(rename = "10001")]
    InvalidSignature,
    #[serde(rename = "10002")]
    InvalidCityCode,
}
#[derive(Debug, Deserialize)]
pub enum ONDCResponseErrorType {
    #[serde(rename = "Gateway")]
    Gateway,
}

#[derive(Debug, Deserialize)]
pub struct ONDCResponseAck {
    pub status: ONDCResponseStatusType,
}

#[derive(Debug, Deserialize)]
pub struct ONDCResponseMessage {
    pub ack: ONDCResponseAck,
}

#[derive(Debug, Deserialize)]
pub struct ONDCResponseError {
    pub r#type: ONDCResponseErrorType,
    pub code: ONDCResponseErrorCode,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ONDCResponse {
    pub context: ONDCContext,
    pub message: ONDCResponseMessage,
    pub error: Option<ONDCResponseError>,
}
