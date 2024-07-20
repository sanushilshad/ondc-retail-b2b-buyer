use std::fmt::{Display, Formatter};

use crate::routes::ondc::buyer::schemas::{ONDCFulfillmentType, ONDCPaymentType};
use crate::{errors::GenericError, schemas::CountryCode};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "payment_type", rename_all = "snake_case")]
pub enum PaymentType {
    PrePaid,
    CashOnDelivery,
    Credit,
}

impl PaymentType {
    pub fn get_ondc_payment(&self) -> ONDCPaymentType {
        match self {
            PaymentType::CashOnDelivery => ONDCPaymentType::OnFulfillment,
            PaymentType::PrePaid => ONDCPaymentType::PreFulfillment,
            PaymentType::Credit => ONDCPaymentType::PostFulfillment,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "fulfillment_type", rename_all = "snake_case")]

pub enum FulfillmentType {
    Delivery,
    SelfPickup,
}

impl FulfillmentType {
    pub fn get_ondc_fulfillment(&self) -> ONDCFulfillmentType {
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
pub enum CategoryDomain {
    #[serde(rename = "RET10")]
    Grocery,
    #[serde(rename = "RET12")]
    Fashion,
    #[serde(rename = "RET13")]
    Bpc,
    #[serde(rename = "RET14")]
    Electronics,
    #[serde(rename = "RET15")]
    Appliances,
    #[serde(rename = "RET16")]
    HomeAndKitchen,
    #[serde(rename = "RET1A")]
    AutoComponentsAndAccessories,
    #[serde(rename = "RET1B")]
    HardwareAndIndustrialEquipments,
    #[serde(rename = "RET1C")]
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
