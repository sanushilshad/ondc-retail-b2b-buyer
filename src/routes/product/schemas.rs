use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::routes::ondc::schemas::{ONDCFulfillmentType, ONDCPaymentType};
use crate::routes::ondc::ONDCItemUOM;
use crate::schemas::{CurrencyType, ONDCNetworkType};
use crate::{errors::GenericError, schemas::CountryCode};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use bigdecimal::BigDecimal;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::Type, Clone)]
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
pub struct ProductFulFillmentLocations {
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
    pub fulfillment_locations: Option<Vec<ProductFulFillmentLocations>>,
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
#[sqlx(type_name = "domain_category")]
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

#[derive(Debug, sqlx::Type)]
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
    pub maximum_value: BigDecimal,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSCreatorContactData<'a> {
    pub name: &'a str,
    pub address: &'a str,
    pub phone: &'a str,
    pub email: &'a str,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSProductCreator<'a> {
    pub name: &'a str,
    pub contact: WSCreatorContactData<'a>,
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

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItemQtyMeasure {
    pub unit: ONDCItemUOM,
    #[schema(value_type = f64)]
    pub value: BigDecimal,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnitizedProductQty {
    pub unit: ONDCItemUOM,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItemQuantity {
    pub unitized: UnitizedProductQty,
    pub available: WSSearchItemQty,
    pub maximum: WSSearchItemQty,
    pub minimum: Option<WSSearchItemQty>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProductProvider<'a> {
    pub id: &'a str,
    pub rating: Option<&'a str>,
    pub name: &'a str,
    pub code: &'a str,
    pub short_desc: &'a str,
    pub long_desc: &'a str,
    pub videos: Vec<&'a str>,
    pub images: Vec<&'a str>,
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

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSItemPayment<'a> {
    pub r#type: PaymentType,
    pub collected_by: &'a ONDCNetworkType,
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

#[derive(Debug, Serialize, ToSchema)]
#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
pub struct WSSearchItem<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub code: Option<&'a str>,
    pub domain_category: CategoryDomain,
    pub price: WSSearchItemPrice,
    pub parent_item_id: Option<&'a str>,
    pub recommended: bool,
    pub payment_types: Vec<WSItemPayment<'a>>,
    pub fullfillment_type: Vec<FulfillmentType>,
    pub location_ids: Vec<&'a str>,
    pub creator: WSProductCreator<'a>,
    pub quantity: WSSearchItemQuantity,
    pub categories: Vec<WSProductCategory>,
    #[schema(value_type = f64)]
    pub tax_rate: BigDecimal,
    // pub country_of_origin: CountryCode,
    pub images: Vec<&'a str>,
    pub price_slabs: Option<Vec<WSPriceSlab>>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchCountry<'a> {
    pub code: &'a CountryCode,
    pub name: Option<&'a str>,
}
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchState<'a> {
    pub code: &'a str,
    pub name: Option<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchCity<'a> {
    pub code: &'a str,
    pub name: &'a str,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProviderLocation<'a> {
    pub id: &'a str,
    pub gps: &'a str,
    pub address: &'a str,
    pub city: WSSearchCity<'a>,
    pub country: WSSearchCountry<'a>,
    pub state: WSSearchState<'a>,
    pub area_code: &'a str,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchProvider<'a> {
    pub items: Vec<WSSearchItem<'a>>,
    pub provider_detail: WSSearchProductProvider<'a>,
    pub locations: HashMap<String, WSSearchProviderLocation<'a>>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchBPP<'a> {
    pub name: &'a str,
    pub code: Option<&'a str>,
    pub subscriber_id: &'a str,
    pub subscriber_uri: &'a str,
    pub short_desc: &'a str,
    pub long_desc: &'a str,
    pub images: Vec<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearchData<'a> {
    pub bpp: WSSearchBPP<'a>,
    pub providers: Vec<WSSearchProvider<'a>>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSearch<'a> {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub message: &'a WSSearchData<'a>,
}
