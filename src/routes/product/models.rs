use serde::Serialize;

use crate::schemas::ONDCNetworkType;

use super::schemas::{CredentialType, PaymentType};
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
pub struct SearchProviderCredentialModel {
    pub id: String,
    pub r#type: CredentialType,
    pub desc: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductVariantAttributeModel {
    pub attribute_code: String,
    pub sequence: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderPaymentOptionModel {
    pub id: String,
    pub r#type: PaymentType,
    pub collected_by: ONDCNetworkType,
}
