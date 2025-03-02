use crate::{
    routes::{ondc::ONDCItemUOM, order::schemas::FulfillmentStatusType},
    schemas::{CountryCode, CurrencyType, ONDCNetworkType},
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use uuid::Uuid;

use super::schemas::{
    AutoCompleteItem, CategoryDomain, CredentialType, PaymentType, WSSearchBPP,
    WSSearchProviderContact, WSSearchProviderCredential, WSSearchProviderDescription,
    WSSearchProviderID, WSSearchProviderTerms,
};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchProviderContactModel {
    pub mobile_no: String,
    pub email: Option<String>,
}

impl WSSearchProviderContactModel {
    pub fn get_schema(self) -> WSSearchProviderContact {
        WSSearchProviderContact {
            mobile_no: self.mobile_no,
            email: self.email,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchProviderTermsModel {
    pub gst_credit_invoice: bool,
}

impl WSSearchProviderTermsModel {
    pub fn get_schema(self) -> WSSearchProviderTerms {
        WSSearchProviderTerms {
            gst_credit_invoice: self.gst_credit_invoice,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchProviderCredentialModel {
    pub id: String,
    pub r#type: CredentialType,
    pub desc: String,
    pub url: Option<String>,
}

impl SearchProviderCredentialModel {
    pub fn get_schema(self) -> WSSearchProviderCredential {
        WSSearchProviderCredential {
            id: self.id,
            r#type: self.r#type,
            desc: self.desc,
            url: self.url,
        }
    }
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
    pub images: sqlx::types::Json<Vec<String>>,
    pub created_on: DateTime<Utc>,
}
impl ESNetworkParticipantModel {
    pub fn get_ws_bpp(self) -> WSSearchBPP {
        WSSearchBPP {
            name: self.name,
            code: None,
            subscriber_id: self.subscriber_id,
            short_desc: self.short_desc,
            long_desc: self.long_desc,
            images: self.images.to_vec(),
        }
    }
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
    pub images: sqlx::types::Json<Vec<String>>,
    pub rating: Option<f32>,
    pub ttl: String,
    pub credentials: Value,
    pub contact: Value,
    pub terms: Value,
    pub identifications: Value,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
}

impl ESProviderModel {
    pub fn get_ws_provider(self) -> WSSearchProviderDescription {
        let contact_models =
            serde_json::from_value::<WSSearchProviderContactModel>(self.contact).unwrap();
        let credential_models =
            serde_json::from_value::<Vec<SearchProviderCredentialModel>>(self.credentials).unwrap();
        let identifications =
            serde_json::from_value::<Vec<WSSearchProviderID>>(self.identifications).unwrap();
        let terms = serde_json::from_value::<WSSearchProviderTermsModel>(self.terms).unwrap();
        WSSearchProviderDescription {
            id: self.provider_id,
            rating: self.rating,
            name: self.name,
            code: self.code,
            short_desc: self.short_desc,
            long_desc: self.long_desc,
            images: self.images.to_vec(),
            ttl: self.ttl,
            contact: contact_models.get_schema(),
            credentials: credential_models
                .into_iter()
                .map(|e| e.get_schema())
                .collect(),
            identifications,
            terms: terms.get_schema(),
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESProviderItemModel {
    pub provider_cache_id: Uuid,
    pub network_participant_cache_id: Uuid,
    pub id: Uuid,
    pub country_code: CountryCode,
    pub domain_code: CategoryDomain,
    pub long_desc: String,
    pub short_desc: String,
    pub item_id: String,
    pub item_code: String,
    pub item_name: String,
    pub currency: CurrencyType,
    pub price_with_tax: BigDecimal,
    pub price_without_tax: BigDecimal,
    pub offered_price: Option<BigDecimal>,
    pub maximum_price: BigDecimal,
    pub tax_rate: BigDecimal,
    pub variant_cache_id: Option<Uuid>,
    pub recommended: bool,
    pub matched: bool,
    pub attributes: Value,
    pub images: Value,
    pub videos: Value,
    pub price_slabs: Option<Value>,
    pub fulfillment_options: Value,
    pub payment_options: Value,
    pub categories: Value,
    pub qty: Value,
    pub creator: Value,
    pub time_to_ship: String,
    pub country_of_origin: Option<String>,
    pub validity: Value,
    pub replacement_terms: Value,
    pub return_terms: Value,
    pub cancellation_terms: Value,
    pub created_on: DateTime<Utc>,
    pub location_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ESAutoCompleteProviderItemModel {
    pub provider_cache_id: Uuid,
    pub network_participant_cache_id: Uuid,
    pub id: Uuid,
    pub item_id: String,
    pub item_code: Option<String>,
    pub item_name: String,
}
impl ESAutoCompleteProviderItemModel {
    pub fn into_schema(self) -> AutoCompleteItem {
        AutoCompleteItem {
            id: self.id,
            item_name: self.item_name,
            item_id: self.item_id,
            item_code: self.item_code,
        }
    }
}
