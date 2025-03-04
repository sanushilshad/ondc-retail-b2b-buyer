use super::schemas::{
    AutoCompleteItem, CategoryDomain, CredentialType, PaymentType, WSCreatorContactData,
    WSItemCancellation, WSItemCancellationFee, WSItemCancellationTerm, WSItemReplacementTerm,
    WSItemReturnLocation, WSItemReturnTerm, WSItemReturnTime, WSItemValidity, WSProductCategory,
    WSProductCreator, WSSearchBPP, WSSearchCity, WSSearchCountry, WSSearchItemAttribute,
    WSSearchItemQty, WSSearchItemQtyMeasure, WSSearchItemQuantity, WSSearchProviderContact,
    WSSearchProviderCredential, WSSearchProviderDescription, WSSearchProviderID,
    WSSearchProviderLocation, WSSearchProviderTerms, WSSearchState, WSSearchVariant,
};
use crate::{
    routes::{ondc::ONDCItemUOM, order::schemas::FulfillmentStatusType},
    schemas::{CountryCode, CurrencyType, ONDCNetworkType},
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use uuid::Uuid;
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReplacementTermModel {
    pub replace_within: String,
}

impl WSItemReplacementTermModel {
    pub fn get_schema(self) -> WSItemReplacementTerm {
        WSItemReplacementTerm {
            replace_within: self.replace_within,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnTimeModel {
    pub duration: String,
}
impl WSItemReturnTimeModel {
    pub fn get_schema(self) -> WSItemReturnTime {
        WSItemReturnTime {
            duration: self.duration,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnLocationModel {
    pub address: String,
    pub gps: String,
}

impl WSItemReturnLocationModel {
    pub fn get_schema(self) -> WSItemReturnLocation {
        WSItemReturnLocation {
            gps: self.gps,
            address: self.address,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemReturnTermModel {
    pub fulfillment_state: FulfillmentStatusType,
    pub return_eligible: bool,
    pub return_time: WSItemReturnTimeModel,
    pub return_location: WSItemReturnLocationModel,
    pub fulfillment_managed_by: String,
}

impl WSItemReturnTermModel {
    pub fn get_schema(self) -> WSItemReturnTerm {
        WSItemReturnTerm {
            fulfillment_state: self.fulfillment_state,
            return_eligible: self.return_eligible,
            return_time: self.return_time.get_schema(),
            return_location: self.return_location.get_schema(),
            fulfillment_managed_by: self.fulfillment_managed_by,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum WSItemCancellationFeeModel {
    Percent { percentage: f64 },
    Amount { amount: f64 },
}

impl WSItemCancellationFeeModel {
    pub fn get_schema(self) -> WSItemCancellationFee {
        match self {
            WSItemCancellationFeeModel::Percent { percentage } => {
                WSItemCancellationFee::Percent { percentage }
            }
            WSItemCancellationFeeModel::Amount { amount } => {
                WSItemCancellationFee::Amount { amount }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemCancellationTermModel {
    pub fulfillment_state: FulfillmentStatusType,
    pub reason_required: bool,
    pub fee: WSItemCancellationFeeModel,
}

impl WSItemCancellationTermModel {
    pub fn get_schema(self) -> WSItemCancellationTerm {
        WSItemCancellationTerm {
            reason_required: self.reason_required,
            fee: self.fee.get_schema(),
            fulfillment_state: self.fulfillment_state,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemCancellationModel {
    pub is_cancellable: bool,
    pub terms: Vec<WSItemCancellationTermModel>,
}

impl WSItemCancellationModel {
    pub fn get_schema(self) -> WSItemCancellation {
        WSItemCancellation {
            is_cancellable: self.is_cancellable,
            terms: self.terms.into_iter().map(|a| a.get_schema()).collect(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemAttributeModel {
    pub label: String,
    pub key: String,
    pub value: String,
}

impl WSSearchItemAttributeModel {
    pub fn get_schema(self) -> WSSearchItemAttribute {
        WSSearchItemAttribute {
            label: self.label,
            key: self.key,
            value: self.value,
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSCreatorContactDataModel {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
}
impl WSCreatorContactDataModel {
    pub fn get_schema(self) -> WSCreatorContactData {
        WSCreatorContactData {
            name: self.name,
            address: self.address,
            phone: self.phone,
            email: self.email,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSProductCreatorModel {
    pub name: String,
    pub contact: WSCreatorContactDataModel,
}

impl WSProductCreatorModel {
    pub fn get_schema(self) -> WSProductCreator {
        WSProductCreator {
            name: self.name,
            contact: self.contact.get_schema(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSProductCategoryModel {
    pub code: String,
    pub name: String,
}

impl WSProductCategoryModel {
    pub fn get_schema(self) -> WSProductCategory {
        WSProductCategory {
            code: self.code,
            name: self.name,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSItemValidityModel {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl WSItemValidityModel {
    pub fn get_schema(self) -> WSItemValidity {
        WSItemValidity {
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQtyModel {
    pub measure: WSSearchItemQtyMeasureModel,
    pub count: u32,
}

impl WSSearchItemQtyModel {
    pub fn get_schema(self) -> WSSearchItemQty {
        WSSearchItemQty {
            measure: self.measure.get_schema(),
            count: self.count,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQtyMeasureModel {
    pub unit: ONDCItemUOM,
    pub value: BigDecimal,
}

impl WSSearchItemQtyMeasureModel {
    pub fn get_schema(self) -> WSSearchItemQtyMeasure {
        WSSearchItemQtyMeasure {
            unit: self.unit,
            value: self.value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WSSearchItemQuantityModel {
    pub unitized: WSSearchItemQtyMeasureModel,
    pub available: WSSearchItemQtyModel,
    pub maximum: WSSearchItemQtyModel,
    pub minimum: Option<WSSearchItemQtyModel>,
}

impl WSSearchItemQuantityModel {
    pub fn get_schema(self) -> WSSearchItemQuantity {
        WSSearchItemQuantity {
            unitized: self.unitized.get_schema(),
            available: self.available.get_schema(),
            maximum: self.maximum.get_schema(),
            minimum: self.minimum.map(|a| a.get_schema()),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, FromRow)]
#[serde(rename_all = "snake_case")]
pub struct ESLocationModel {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
    pub fn get_schema(self) -> WSSearchBPP {
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

impl ESProviderLocationModel {
    pub fn get_schema(self) -> WSSearchProviderLocation {
        WSSearchProviderLocation {
            id: self.location_id,
            gps: format!("{},{}", self.latitude, self.longitude),
            address: self.address,
            city: WSSearchCity {
                code: self.city_code,
                name: self.city_name,
            },
            country: WSSearchCountry {
                code: self.country_code,
                name: self.country_name,
            },
            state: WSSearchState {
                code: self.state_code,
                name: self.state_name,
            },
            area_code: self.area_code,
        }
    }
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

impl ESProviderItemVariantModel {
    pub fn get_schema(self) -> WSSearchVariant {
        WSSearchVariant {
            id: self.variant_id,
            name: self.variant_name,
            attributes: serde_json::from_value(self.attributes).unwrap(),
        }
    }
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
    pub item_code: Option<String>,
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
    pub validity: Option<Value>,
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

#[derive(Debug, Serialize, FromRow)]
pub struct SearchLocationModel {
    pub country_code: CountryCode,
    pub city_code: String,
    pub domain_category_code: CategoryDomain,
}
