use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schemas::CountryCode;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCDomain {
    #[serde(rename = "ONDC:RET10")]
    GROCERY,
}

impl ONDCDomain {
    pub fn get_ondc_domain(domain_category_id: &str) -> Result<ONDCDomain, serde_json::Error> {
        serde_json::from_str(&format!("ONDC:{}", domain_category_id))
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
