use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
struct ONDCContextCountry {
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCContextCity {
    code: String,
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
            code: "IND".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCContextLocation {
    city: ONDCContextCity,
    country: ONDCContextCountry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContext {
    domain: ONDCDomain,
    location: Option<ONDCContextLocation>,
    country: Option<String>,
    city: Option<String>,
    action: ONDCActionType,
    version: ONDCVersion,
    transaction_id: String,
    message_id: String,
    timestamp: DateTime<Utc>,
    bap_id: String,
    bap_uri: String,
    bpp_id: Option<String>,
    bpp_uri: Option<String>,
    ttl: String,
}
