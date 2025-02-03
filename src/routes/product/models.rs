use serde::Serialize;

use super::schemas::CredentialType;
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchProviderContactModel {
    pub mobile_no: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchProviderTermsModel {
    pub gst_credit_invoice: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchProviderCredentialModel {
    pub id: String,
    pub r#type: CredentialType,
    pub desc: String,
    pub url: String,
}
