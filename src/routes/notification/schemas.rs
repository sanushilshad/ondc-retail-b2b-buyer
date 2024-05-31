use serde::Deserialize;

use crate::domain::{subscriber_email::deserialize_subscriber_email, EmailObject};

#[derive(Deserialize, Debug, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum OTPScope {
    Email,
    Sms,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct OTPRequestBody {
    #[serde(deserialize_with = "deserialize_subscriber_email")]
    pub identifier: EmailObject,
    // pub service_type: CommunicationType,
}
