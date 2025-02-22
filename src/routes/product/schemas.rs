use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::routes::ondc::schemas::{ONDCFulfillmentType, ONDCPaymentType};
use crate::routes::ondc::ONDCItemUOM;
use crate::routes::order::schemas::FulfillmentStatusType;
use crate::schemas::{CurrencyType, ONDCNetworkType};
use crate::{errors::GenericError, schemas::CountryCode};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use std::collections::HashSet;
use utoipa::ToSchema;
use uuid::Uuid;

use super::models::{
    WSCreatorContactDataModel, WSItemCancellationFeeModel, WSItemCancellationModel,
    WSItemCancellationTermModel, WSItemReplacementTermModel, WSItemReturnLocationModel,
    WSItemReturnTermModel, WSItemReturnTimeModel, WSItemValidityModel, WSPriceSlabModel,
    WSProductCategoryModel, WSProductCreatorModel, WSSearchItemAttributeModel,
    WSSearchItemQtyMeasureModel, WSSearchItemQtyModel, WSSearchItemQuantityModel,
};
#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::Type, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "payment_type", rename_all = "snake_case")]
pub enum PaymentType {
    PrePaid,
    CashOnDelivery,
    Credit,
}

// impl PgHasArrayType for PaymentType {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_payment_type")
//     }
// }

impl PaymentType {
    pub fn get_ondc_payment(&self) -> ONDCPaymentType {
        match self {
            PaymentType::CashOnDelivery => ONDCPaymentType::OnFulfillment,
            PaymentType::PrePaid => ONDCPaymentType::PreFulfillment,
            PaymentType::Credit => ONDCPaymentType::PostFulfillment,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::Type, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "fulfillment_type", rename_all = "snake_case")]

pub enum FulfillmentType {
    Delivery,
    SelfPickup,
}

// impl PgHasArrayType for &FulfillmentType {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_fulfillment_type")
//     }
// }

impl FulfillmentType {
    pub fn get_ondc_fulfillment_type(&self) -> ONDCFulfillmentType {
        match self {
            FulfillmentType::Delivery => ONDCFulfillmentType::Delivery,
            FulfillmentType::SelfPickup => ONDCFulfillmentType::SelfPickup,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "product_search_type", rename_all = "snake_case")]
pub enum ProductSearchType {
    Item,
    Fulfillment,
    Category,
    City,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductFulFillmentLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub area_code: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductSearchRequest {
    pub query: String,
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub domain_category_code: CategoryDomain,
    pub country_code: CountryCode,
    pub payment_type: Option<PaymentType>,
    pub fulfillment_type: Option<FulfillmentType>,
    pub search_type: ProductSearchType,
    pub fulfillment_locations: Option<Vec<ProductFulFillmentLocation>>,
    pub city_code: String,
    pub update_cache: bool,
}

impl FromRequest for ProductSearchRequest {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "domain_category_type")]
pub enum CategoryDomain {
    #[serde(rename = "RET10")]
    #[sqlx(rename = "RET10")]
    Grocery,
    #[serde(rename = "RET12")]
    #[sqlx(rename = "RET12")]
    Fashion,
    #[serde(rename = "RET13")]
    #[sqlx(rename = "RET13")]
    Bpc,
    #[serde(rename = "RET14")]
    #[sqlx(rename = "RET14")]
    Electronics,
    #[serde(rename = "RET15")]
    #[sqlx(rename = "RET15")]
    Appliances,
    #[serde(rename = "RET16")]
    #[sqlx(rename = "RET16")]
    HomeAndKitchen,
    #[serde(rename = "RET1A")]
    #[sqlx(rename = "RET1A")]
    AutoComponentsAndAccessories,
    #[serde(rename = "RET1B")]
    #[sqlx(rename = "RET1B")]
    HardwareAndIndustrialEquipments,
    #[serde(rename = "RET1C")]
    #[sqlx(rename = "RET1C")]
    BuildingAndConstructionSupplies,
}

impl Display for CategoryDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CategoryDomain::Grocery => "RET10",
                CategoryDomain::Fashion => "RET12",
                CategoryDomain::Bpc => "RET13",
                CategoryDomain::Electronics => "RET14",
                CategoryDomain::Appliances => "RET15",
                CategoryDomain::HomeAndKitchen => "RET16",
                CategoryDomain::AutoComponentsAndAccessories => "RET1A",
                CategoryDomain::HardwareAndIndustrialEquipments => "RET1B",
                CategoryDomain::BuildingAndConstructionSupplies => "RET1C",
            }
        )
    }
}

#[derive(Debug, sqlx::Type, Deserialize, Serialize)]
pub struct SearchRequestModel {
    pub transaction_id: String,
    pub update_cache: bool,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub device_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItemPrice {
    pub currency: CurrencyType,
    #[schema(value_type = f64)]
    pub price_with_tax: BigDecimal,
    #[schema(value_type = f64)]
    pub price_without_tax: BigDecimal,
    #[schema(value_type =Option<f64>)]
    pub offered_value: Option<BigDecimal>,
    #[schema(value_type = f64)]
    pub maximum_price: BigDecimal,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSCreatorContactData {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
}

impl WSCreatorContactData {
    fn get_model(&self) -> WSCreatorContactDataModel {
        WSCreatorContactDataModel {
            name: self.name.to_owned(),
            address: self.address.to_owned(),
            phone: self.phone.to_owned(),
            email: self.email.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSProductCreator {
    pub name: String,
    pub contact: WSCreatorContactData,
}

impl WSProductCreator {
    pub fn get_model(&self) -> WSProductCreatorModel {
        WSProductCreatorModel {
            name: self.name.to_owned(),
            contact: self.contact.get_model(),
        }
    }
}

// #[derive(Debug, Serialize)]
// struct ProviderLocation {
//     id: String,
//     gps: String,
//     address: String,
//     city: String,
//     state: String,
//     country: CountryCode,
//     area_code: String,
// }

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItemQty {
    pub measure: WSSearchItemQtyMeasure,
    pub count: u32,
}
impl WSSearchItemQty {
    fn get_model(&self) -> WSSearchItemQtyModel {
        WSSearchItemQtyModel {
            measure: self.measure.get_model(),
            count: self.count.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItemQtyMeasure {
    pub unit: ONDCItemUOM,
    #[schema(value_type = f64)]
    pub value: BigDecimal,
}

impl WSSearchItemQtyMeasure {
    fn get_model(&self) -> WSSearchItemQtyMeasureModel {
        WSSearchItemQtyMeasureModel {
            unit: self.unit.to_owned(),
            value: self.value.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItemQuantity {
    pub unitized: WSSearchItemQtyMeasure,
    pub available: WSSearchItemQty,
    pub maximum: WSSearchItemQty,
    pub minimum: Option<WSSearchItemQty>,
}

impl WSSearchItemQuantity {
    pub fn get_model(&self) -> WSSearchItemQuantityModel {
        WSSearchItemQuantityModel {
            unitized: self.unitized.get_model(),
            available: self.available.get_model(),
            maximum: self.maximum.get_model(),
            minimum: self.minimum.as_ref().map(|f| f.get_model()),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProviderContact {
    pub mobile_no: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchProviderID {
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProviderTerms {
    pub gst_credit_invoice: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProviderCredential {
    pub id: String,
    pub r#type: CredentialType,
    pub desc: String,
    pub url: Option<String>,
}
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProductProviderDescription {
    pub id: String,
    pub rating: Option<f32>,
    pub name: String,
    pub code: String,
    pub short_desc: String,
    pub long_desc: String,
    // pub chat_link: Vec<String>,
    pub images: Vec<String>,
    pub ttl: String,
    pub contact: WSSearchProviderContact,
    pub credentials: Vec<WSSearchProviderCredential>,
    pub identifications: Vec<WSSearchProviderID>,
    pub terms: WSSearchProviderTerms,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProductNpDeatils {
    pub name: String,
    pub code: Option<String>,
    pub short_desc: String,
    pub long_desc: String,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSProductCategory {
    pub code: String,
    pub name: String,
}
impl WSProductCategory {
    pub fn get_model(&self) -> WSProductCategoryModel {
        WSProductCategoryModel {
            code: self.code.to_owned(),
            name: self.name.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WSPaymentTypes {
    pub r#type: PaymentType,
    pub collected_by: ONDCNetworkType,
}

#[derive(Debug, Serialize, ToSchema)]
#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
pub struct WSPriceSlab {
    #[schema(value_type = f64)]
    pub min: BigDecimal,
    #[schema(value_type = Option<f64>)]
    pub max: Option<BigDecimal>,
    #[schema(value_type = f64)]
    pub price_with_tax: BigDecimal,
    #[schema(value_type = f64)]
    pub price_without_tax: BigDecimal,
}

impl WSPriceSlab {
    pub fn get_model(&self) -> WSPriceSlabModel {
        WSPriceSlabModel {
            min: self.min.to_owned(),
            max: self.max.to_owned(),
            price_with_tax: self.price_with_tax.to_owned(),
            price_without_tax: self.price_without_tax.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItemAttribute {
    pub label: String,
    pub key: String,
    pub value: String,
}
impl WSSearchItemAttribute {
    pub fn get_model(&self) -> WSSearchItemAttributeModel {
        WSSearchItemAttributeModel {
            label: self.label.clone(),
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemValidity {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}
impl WSItemValidity {
    pub fn get_model(&self) -> WSItemValidityModel {
        WSItemValidityModel {
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemReplacementTerm {
    pub replace_within: String,
}
impl WSItemReplacementTerm {
    pub fn get_model(&self) -> WSItemReplacementTermModel {
        WSItemReplacementTermModel {
            replace_within: self.replace_within.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemReturnTime {
    pub duration: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemReturnLocation {
    pub address: String,
    pub gps: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemReturnTerm {
    pub fulfillment_state: FulfillmentStatusType,
    pub return_eligible: bool,
    pub return_time: WSItemReturnTime,
    pub return_location: WSItemReturnLocation,
    pub fulfillment_managed_by: String,
}

impl WSItemReturnTerm {
    pub fn get_model(&self) -> WSItemReturnTermModel {
        WSItemReturnTermModel {
            fulfillment_state: self.fulfillment_state.to_owned(),
            return_eligible: self.return_eligible,
            return_time: WSItemReturnTimeModel {
                duration: self.return_time.duration.to_owned(),
            },
            return_location: WSItemReturnLocationModel {
                address: self.return_location.address.to_owned(),
                gps: self.return_location.gps.to_owned(),
            },
            fulfillment_managed_by: self.fulfillment_managed_by.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum WSItemCancellationFee {
    Percent { percentage: f64 },
    Amount { amount: f64 },
}

impl WSItemCancellationFee {
    fn get_model(&self) -> WSItemCancellationFeeModel {
        match self {
            WSItemCancellationFee::Percent { percentage } => WSItemCancellationFeeModel::Percent {
                percentage: *percentage,
            },
            WSItemCancellationFee::Amount { amount } => {
                WSItemCancellationFeeModel::Amount { amount: *amount }
            }
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemCancellationTerm {
    pub fulfillment_state: FulfillmentStatusType,
    pub reason_required: bool,
    pub fee: WSItemCancellationFee,
}

impl WSItemCancellationTerm {
    fn get_model(&self) -> WSItemCancellationTermModel {
        WSItemCancellationTermModel {
            fulfillment_state: self.fulfillment_state.to_owned(),
            reason_required: self.reason_required,
            fee: self.fee.get_model(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemCancellation {
    pub is_cancellable: bool,
    pub terms: Vec<WSItemCancellationTerm>,
}

impl WSItemCancellation {
    pub fn get_model(&self) -> WSItemCancellationModel {
        let terms = self.terms.iter().map(|f| f.get_model()).collect();
        WSItemCancellationModel {
            is_cancellable: self.is_cancellable,
            terms,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItem {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub domain_category: CategoryDomain,
    pub price: WSSearchItemPrice,
    pub parent_item_id: Option<String>,
    pub recommended: bool,
    pub fullfillment_options: Vec<FulfillmentType>,
    pub location_ids: Vec<String>,
    pub payment_options: Vec<PaymentType>,
    pub creator: WSProductCreator,
    pub quantity: WSSearchItemQuantity,
    pub categories: Vec<WSProductCategory>,
    #[schema(value_type = f64)]
    pub tax_rate: BigDecimal,
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub price_slabs: Option<Vec<WSPriceSlab>>,
    pub attributes: Vec<WSSearchItemAttribute>,
    pub matched: bool,
    pub time_to_ship: String,
    pub country_of_origin: Option<String>,
    pub validity: Option<WSItemValidity>,
    pub replacement_terms: Vec<WSItemReplacementTerm>,
    pub return_terms: Vec<WSItemReturnTerm>,
    pub cancellation_terms: WSItemCancellation,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchCountry {
    pub code: CountryCode,
    pub name: Option<String>,
}
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchState {
    pub code: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchCity {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProviderLocation {
    pub id: String,
    pub gps: String,
    pub address: String,
    pub city: WSSearchCity,
    pub country: WSSearchCountry,
    pub state: WSSearchState,
    pub area_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchVariantAttribute {
    pub attribute_code: String,
    pub sequence: String,
}
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchVariant {
    pub name: String,
    pub attributes: Vec<WSSearchVariantAttribute>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProvider {
    pub items: Vec<WSSearchItem>,
    pub description: WSSearchProductProviderDescription,
    pub locations: HashMap<String, WSSearchProviderLocation>,
    pub servicability: HashMap<String, WSSearchServicability>,
    // pub payments: HashMap<String, WSPaymentTypes>,
    pub variants: Option<HashMap<String, WSSearchVariant>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSServicabilityData<D> {
    pub category_code: Option<String>,
    pub value: D,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchServicability {
    pub geo_json: Vec<WSServicabilityData<Value>>,
    pub hyperlocal: Vec<WSServicabilityData<f64>>,
    pub country: Vec<WSServicabilityData<CountryCode>>,
    pub intercity: Vec<WSServicabilityData<HashSet<String>>>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchBPP {
    pub name: String,
    pub code: Option<String>,
    pub subscriber_id: String,
    pub short_desc: String,
    pub long_desc: String,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchData {
    pub bpp: WSSearchBPP,
    pub providers: Vec<WSSearchProvider>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearch {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub message: WSSearchData,
}

pub struct BulkProviderCache<'a> {
    pub provider_ids: Vec<&'a str>,
    pub network_participant_cache_ids: Vec<Uuid>,
    pub names: Vec<&'a str>,
    pub codes: Vec<&'a str>,
    pub short_descs: Vec<&'a str>,
    pub long_descs: Vec<&'a str>,
    pub images: Vec<Value>,
    pub ratings: Vec<Option<f32>>,
    pub ttls: Vec<&'a str>,
    pub credentials: Vec<Value>,
    pub contacts: Vec<Value>,
    pub terms: Vec<Value>,
    pub identifications: Vec<Value>,
    pub ids: Vec<Uuid>,
    pub created_ons: Vec<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    License,
    FssaiLicenseNo,
}

pub struct BulkProviderLocationCache<'a> {
    pub ids: Vec<Uuid>,
    pub provider_ids: Vec<&'a Uuid>,
    pub location_ids: Vec<&'a str>,
    pub latitudes: Vec<BigDecimal>,
    pub longitudes: Vec<BigDecimal>,
    pub addresses: Vec<&'a str>,
    pub city_codes: Vec<&'a str>,
    pub city_names: Vec<&'a str>,
    pub state_codes: Vec<&'a str>,
    pub state_names: Vec<Option<&'a str>>,
    pub country_codes: Vec<&'a CountryCode>,
    pub country_names: Vec<Option<&'a str>>,
    pub area_codes: Vec<&'a str>,
    pub updated_ons: Vec<DateTime<Utc>>,
}

pub struct BulkGeoServicabilityCache<'a> {
    pub ids: Vec<Uuid>,
    pub location_cache_ids: Vec<&'a Uuid>,
    pub coordinates: Vec<&'a Value>,
    pub category_codes: Vec<&'a Option<String>>,
    pub created_ons: Vec<DateTime<Utc>>,
    pub domain_codes: Vec<&'a CategoryDomain>,
}

pub struct BulkHyperlocalServicabilityCache<'a> {
    pub ids: Vec<Uuid>,
    pub location_cache_ids: Vec<&'a Uuid>,
    pub radii: Vec<f64>,
    pub category_codes: Vec<&'a Option<String>>,
    pub created_ons: Vec<DateTime<Utc>>,
    pub domain_codes: Vec<&'a CategoryDomain>,
}

pub struct BulkCountryServicabilityCache<'a> {
    pub ids: Vec<Uuid>,
    pub location_cache_ids: Vec<&'a Uuid>,
    pub country_codes: Vec<&'a CountryCode>,
    pub category_codes: Vec<&'a Option<String>>,
    pub created_ons: Vec<DateTime<Utc>>,
    pub domain_codes: Vec<&'a CategoryDomain>,
}

pub struct BulkInterCityServicabilityCache<'a> {
    pub ids: Vec<Uuid>,
    pub location_cache_ids: Vec<&'a Uuid>,
    pub pincodes: Vec<&'a str>,
    pub category_codes: Vec<&'a Option<String>>,
    pub created_ons: Vec<DateTime<Utc>>,
    pub domain_codes: Vec<&'a CategoryDomain>,
}

pub struct BulkItemVariantCache<'a> {
    pub provider_ids: Vec<&'a Uuid>,
    pub ids: Vec<Uuid>,
    pub variant_ids: Vec<&'a str>,
    pub variant_names: Vec<&'a str>,
    pub created_ons: Vec<DateTime<Utc>>,
    pub attributes: Vec<Value>,
}

pub struct BulkItemCache<'a> {
    pub provider_ids: Vec<&'a Uuid>,
    pub ids: Vec<Uuid>,
    pub country_codes: Vec<&'a CountryCode>,
    pub domain_codes: Vec<&'a CategoryDomain>,
    pub category_codes: Vec<&'a str>,
    pub item_ids: Vec<&'a str>,
    pub item_codes: Vec<&'a str>,
    pub item_names: Vec<&'a str>,
    pub currencies: Vec<&'a CurrencyType>,
    pub price_with_taxes: Vec<&'a BigDecimal>,
    pub price_without_taxes: Vec<&'a BigDecimal>,
    pub offered_prices: Vec<&'a Option<BigDecimal>>,
    pub maximum_prices: Vec<&'a BigDecimal>,
    pub tax_rates: Vec<&'a BigDecimal>,
    pub variant_ids: Vec<Option<Uuid>>,
    pub recommends: Vec<bool>,
    pub matched: Vec<bool>,
    pub attributes: Vec<Value>,
    pub images: Vec<Value>,
    pub videos: Vec<Value>,
    pub price_slabs: Vec<Option<Value>>,
    pub fulfillment_options: Vec<Value>,
    pub payment_options: Vec<Value>,
    pub categories: Vec<Value>,
    pub qtys: Vec<Value>,
    pub creators: Vec<Value>,
    pub time_to_ships: Vec<&'a str>,
    pub country_of_origins: Vec<Option<&'a str>>,
    pub validities: Vec<Value>,
    pub replacement_terms: Vec<Value>,
    pub return_terms: Vec<Value>,
    pub cancellation_terms: Vec<Value>,
    pub created_ons: Vec<DateTime<Utc>>,
}

pub struct BulkItemLocationCache<'a> {
    pub item_cache_ids: Vec<&'a Uuid>,
    pub location_cache_ids: Vec<&'a Uuid>,
    pub ids: Vec<Uuid>,
    pub created_ons: Vec<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ProductList {
    pub query: String,
    pub domain_category_code: CategoryDomain,
    pub country_code: CountryCode,
    pub payment_type: Option<PaymentType>,
    pub fulfillment_type: Option<FulfillmentType>,
    pub search_type: ProductSearchType,
    pub fulfillment_locations: ProductFulFillmentLocation,
    pub city_code: String,
    pub offset: i16,
    pub limit: i16,
}

impl FromRequest for ProductList {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MinimalItem {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub domain_category: CategoryDomain,
    pub price: WSSearchItemPrice,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemMinimalProvider {
    pub items: Vec<MinimalItem>,
    pub description: ItemProviderMinimalDescription,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemProviderMinimalDescription {
    pub id: String,
    pub rating: Option<f32>,
    pub name: String,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemMinimalBPP {
    pub name: String,
    pub code: Option<String>,
    pub subscriber_id: String,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MinimalItemData {
    pub bpp: ItemMinimalBPP,
    pub providers: Vec<ItemMinimalProvider>,
}

pub struct ServicabilityIds {
    pub hyperlocal: Vec<Uuid>,
    pub country: Vec<Uuid>,
    pub inter_city: Vec<Uuid>,
    pub geo_json: Vec<Uuid>,
}

pub struct DBItemCacheData {
    pub servicability_ids: ServicabilityIds,
    pub location_ids: Vec<Uuid>,
    pub provider_ids: Vec<Uuid>,
    pub network_participant_ids: Vec<Uuid>,
}
