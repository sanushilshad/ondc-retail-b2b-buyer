use crate::{
    routes::{ondc::ONDCItemUOM, order::schemas::FulfillmentStatusType},
    schemas::{CountryCode, ONDCNetworkType},
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use uuid::Uuid;

use super::schemas::{CategoryDomain, CredentialType, PaymentType};
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
    pub url: Option<String>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReplacementTermModel {
    pub replace_within: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnTimeModel {
    pub duration: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnLocationModel {
    pub address: String,
    pub gps: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnTermModel {
    pub fulfillment_state: FulfillmentStatusType,
    pub return_eligible: bool,
    pub return_time: WSItemReturnTimeModel,
    pub return_location: WSItemReturnLocationModel,
    pub fulfillment_managed_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum WSItemCancellationFeeModel {
    Percent { percentage: f64 },
    Amount { amount: f64 },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemCancellationTermModel {
    pub fulfillment_state: FulfillmentStatusType,
    pub reason_required: bool,
    pub fee: WSItemCancellationFeeModel,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemCancellationModel {
    pub is_cancellable: bool,
    pub terms: Vec<WSItemCancellationTermModel>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemAttributeModel {
    pub label: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
#[skip_serializing_none]
#[serde(rename_all = "snake_case")]
pub struct WSPriceSlabModel {
    pub min: BigDecimal,
    pub max: Option<BigDecimal>,
    pub price_with_tax: BigDecimal,
    pub price_without_tax: BigDecimal,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSCreatorContactDataModel {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSProductCreatorModel {
    pub name: String,
    pub contact: WSCreatorContactDataModel,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSProductCategoryModel {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemValidityModel {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQtyModel {
    pub measure: WSSearchItemQtyMeasureModel,
    pub count: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQtyMeasureModel {
    pub unit: ONDCItemUOM,
    pub value: BigDecimal,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQuantityModel {
    pub unitized: WSSearchItemQtyMeasureModel,
    pub available: WSSearchItemQtyModel,
    pub maximum: WSSearchItemQtyModel,
    pub minimum: Option<WSSearchItemQtyModel>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ESLocationModel {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESHyperlocalServicabilityModel {
    pub id: Uuid,
    pub location_cache_id: Uuid,
    pub domain_code: CategoryDomain,
    pub category_code: Option<String>,
    pub radius: f64,
    pub location: ESLocationModel,
    pub created_on: DateTime<Utc>,
    pub provider_cache_id: Uuid,
    pub network_participant_cache_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESCountryServicabilityModel {
    pub id: Uuid,
    pub location_cache_id: Uuid,
    pub domain_code: CategoryDomain,
    pub category_code: Option<String>,
    pub country_code: CountryCode,
    pub created_on: DateTime<Utc>,
    pub provider_cache_id: Uuid,
    pub network_participant_cache_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESInterCityServicabilityModel {
    pub id: Uuid,
    pub location_cache_id: Uuid,
    pub domain_code: CategoryDomain,
    pub category_code: Option<String>,
    pub pincode: String,
    pub created_on: DateTime<Utc>,
    pub provider_cache_id: Uuid,
    pub network_participant_cache_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESGeoJsonServicabilityModel {
    pub id: Uuid,
    pub location_cache_id: Uuid,
    pub domain_code: CategoryDomain,
    pub category_code: Option<String>,
    pub coordinates: Value,
    pub created_on: DateTime<Utc>,
    pub provider_cache_id: Uuid,
    pub network_participant_cache_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESNetworkParticipantModel {
    pub id: Uuid,
    pub subscriber_id: String,
    pub name: String,
    pub short_desc: String,
    pub long_desc: String,
    pub images: Value,
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESProviderLocationModel {
    pub id: Uuid,
    pub provider_cache_id: Uuid,
    pub location_id: String,
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
    pub address: String,
    pub city_code: String,
    pub city_name: String,
    pub state_code: String,
    pub state_name: Option<String>,
    pub country_code: CountryCode,
    pub country_name: Option<String>,
    pub area_code: String,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESProviderModel {
    pub id: Uuid,
    pub provider_id: String,
    pub network_participant_cache_id: Uuid,
    pub name: String,
    pub code: String,
    pub short_desc: String,
    pub long_desc: String,
    pub images: Value,
    pub rating: Option<f32>,
    pub ttl: String,
    pub credentials: Value,
    pub contact: Value,
    pub terms: Value,
    pub identifications: Value,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESProviderItemVariantModel {
    pub id: Uuid,
    pub provider_cache_id: Uuid,
    pub variant_id: String,
    pub variant_name: String,
    pub attributes: Value,

    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
}
