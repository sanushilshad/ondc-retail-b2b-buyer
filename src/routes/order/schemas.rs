use std::collections::HashSet;

use crate::errors::GenericError;
use crate::routes::ondc::schemas::{
    ONDCFulfillmentStateType, ONDCPaymentSettlementCounterparty, ONDCPaymentSettlementPhase,
    ONDCPaymentSettlementType, ONDCSettlementBasis,
};
use crate::routes::ondc::{ONDCOrderStatus, ONDCOrderUpdateTarget, ONDCPaymentCollectedBy};
use crate::routes::product::schemas::FulfillmentType;
use crate::routes::product::schemas::{CategoryDomain, PaymentType};
use crate::schemas::DataSource;
use crate::schemas::{CountryCode, CurrencyType, FeeType};
use crate::utils::pascal_to_snake_case;
// use crate::utils::deserialize_non_empty_vector;
use actix_http::Payload;
use actix_web::{web, FromRequest, HttpRequest};
use serde_json::Value;

use crate::domain::EmailObject;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuyerTerms {
    pub item_req: String,
    pub packaging_req: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderSelectItem {
    pub item_id: String,
    pub location_ids: Vec<String>,
    pub qty: i32,
    pub buyer_term: Option<BuyerTerms>,
    pub fulfillment_ids: Vec<String>,
}

#[derive(Deserialize, Debug, ToSchema, Serialize)]
pub struct Country {
    pub code: CountryCode,
    pub name: String,
}

#[derive(Deserialize, Debug, ToSchema, Serialize)]
pub struct City {
    pub code: String,
    pub name: String,
}

#[derive(Deserialize, Debug, ToSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectFulfillmentLocation {
    pub gps: String,
    pub area_code: String,
    pub address: String,
    pub city: City,
    pub country: Country,
    pub state: String,
    pub contact_mobile_no: String,
}

#[derive(Deserialize, Debug, ToSchema, sqlx::Type)]
#[sqlx(type_name = "inco_term_type", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum IncoTermType {
    Exw,
    Cif,
    Fob,
    Dap,
    Ddp,
}

// impl PgHasArrayType for &IncoTermType {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_inco_term_type")
//     }
// }

impl std::fmt::Display for IncoTermType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IncoTermType::Exw => "EXW",
            IncoTermType::Cif => "CIF",
            IncoTermType::Fob => "FOB",
            IncoTermType::Dap => "DAP",
            IncoTermType::Ddp => "DDP",
        };

        write!(f, "{}", s)
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderDeliveyTerm {
    pub inco_terms: IncoTermType,
    pub place_of_delivery: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderSelectFulfillment {
    pub id: String,
    pub r#type: FulfillmentType,
    // #[serde(deserialize_with = "deserialize_non_empty_vector")]
    pub location: SelectFulfillmentLocation,
    pub delivery_terms: Option<OrderDeliveyTerm>,
}

// #[derive(Deserialize, Debug, sqlx::Type)]
// #[sqlx(type_name = "commerce_data_type", rename_all = "snake_case")]
// #[serde(rename_all = "snake_case")]
// pub enum CommerceDataType {
//     Order,
//     PurchaseOrder,
// }

#[derive(Deserialize, Debug, ToSchema, PartialEq, sqlx::Type, Serialize)]
#[sqlx(type_name = "commerce_data_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    PurchaseOrder,
    SaleOrder,
}

impl OrderType {
    pub fn is_purchase_order(&self) -> bool {
        matches!(self, OrderType::PurchaseOrder)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(
    type_name = "fulfillment_servicability_status",
    rename_all = "snake_case"
)]
pub enum ServiceableType {
    #[serde(rename = "non_serviceable")]
    NonServiceable,
    #[serde(rename = "serviceable")]
    Serviceable,
}
// impl PgHasArrayType for ServiceableType {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_fulfillment_servicability_status")
//     }
// }

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "fulfillment_category_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentCategoryType {
    #[serde(rename = "standard_delivery")]
    StandardDelivery,
    #[serde(rename = "express_delivery")]
    ExpressDelivery,
    #[serde(rename = "self_pickup")]
    SelfPickup,
}
// impl PgHasArrayType for FulfillmentCategoryType {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_fulfillment_category_type")
//     }
// }

// impl std::fmt::Display for FulfillmentCategoryType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let s = match self {
//             FulfillmentCategoryType::StandardDelivery => "standard_delivery",
//             FulfillmentCategoryType::ExpressDelivery => "express_delivery",
//             FulfillmentCategoryType::SelfPickup => "self_pickup",
//         };

//         write!(f, "{}", s)
//     }
// }

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderSelectRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub domain_category_code: CategoryDomain,
    pub payment_types: Vec<PaymentType>,
    pub provider_id: String,
    pub items: Vec<OrderSelectItem>,
    pub ttl: String,
    pub fulfillments: Vec<OrderSelectFulfillment>,
    pub order_type: OrderType,
    pub bpp_id: String,
}

impl FromRequest for OrderSelectRequest {
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

#[derive(Deserialize, Debug, sqlx::Type, ToSchema)]
#[sqlx(type_name = "commerce_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CommerceStatusType {
    QuoteRequested,
    QuoteAccepted,
    QuoteRejected,
    Initialized,
    Created,
    Accepted,
    InProgress,
    Completed,
    Cancelled,
}

impl CommerceStatusType {
    pub fn get_ondc_order_status(&self) -> ONDCOrderStatus {
        match self {
            CommerceStatusType::QuoteRequested => ONDCOrderStatus::InProgress,
            CommerceStatusType::QuoteAccepted => ONDCOrderStatus::InProgress,
            CommerceStatusType::QuoteRejected => ONDCOrderStatus::Cancelled,
            CommerceStatusType::Initialized => ONDCOrderStatus::InProgress,
            CommerceStatusType::Created => ONDCOrderStatus::Created,
            CommerceStatusType::Accepted => ONDCOrderStatus::Accepted,
            CommerceStatusType::InProgress => ONDCOrderStatus::InProgress,
            CommerceStatusType::Completed => ONDCOrderStatus::Completed,
            CommerceStatusType::Cancelled => ONDCOrderStatus::Cancelled,
        }
    }
}

impl std::fmt::Display for CommerceStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

// #[derive(Deserialize, Debug)]
// pub struct OrderStatusHistory {
//     created_on: DateTime<Utc>,
//     status: CommerceStatusType,
// }

#[derive(Deserialize, Debug, Serialize, sqlx::Encode)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub collected_by: Option<PaymentCollectedBy>,
    pub r#type: PaymentType,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, ToSchema, PartialEq)]
#[sqlx(type_name = "payment_collected_by_type")]
pub enum PaymentCollectedBy {
    #[serde(rename = "BAP")]
    #[sqlx(rename = "BAP")]
    Bap,
    #[serde(rename = "BPP")]
    #[sqlx(rename = "BPP")]
    Bpp,
    #[serde(rename = "buyer")]
    #[sqlx(rename = "buyer")]
    Buyer,
}
impl PaymentCollectedBy {
    pub fn get_ondc_type(&self) -> ONDCPaymentCollectedBy {
        match self {
            PaymentCollectedBy::Bap => ONDCPaymentCollectedBy::Bap,
            PaymentCollectedBy::Bpp => ONDCPaymentCollectedBy::Bpp,
            PaymentCollectedBy::Buyer => ONDCPaymentCollectedBy::Buyer,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderInitBilling {
    pub name: String,
    pub address: String,
    pub tax_id: String,
    pub mobile_no: String,
    pub email: String,
    pub city: City,
    pub state: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderInitRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub billing: OrderInitBilling,
}

impl FromRequest for OrderInitRequest {
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

#[derive(Deserialize, Debug, ToSchema)]
pub struct CommerceSeller {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct BasicNetworkData {
    pub id: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaymentSettlementDetail {
    pub settlement_counterparty: PaymentSettlementCounterparty,
    pub settlement_phase: PaymentSettlementPhase,
    pub settlement_type: PaymentSettlementType,
    pub settlement_bank_account_no: String,
    pub settlement_ifsc_code: String,
    pub beneficiary_name: String,
    pub bank_name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "settlement_basis_type", rename_all = "snake_case")]
pub enum SettlementBasis {
    ReturnWindowExpiry,
    Shipment,
    Delivery,
}

impl SettlementBasis {
    pub fn get_ondc_settlement_basis(self) -> ONDCSettlementBasis {
        match self {
            SettlementBasis::ReturnWindowExpiry => ONDCSettlementBasis::ReturnWindowExpiry,
            SettlementBasis::Shipment => ONDCSettlementBasis::Shipment,
            SettlementBasis::Delivery => ONDCSettlementBasis::Delivery,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct SellerPaymentDetail {
    pub uri: String,
    pub ttl: Option<String>,
    pub dsa: Option<String>,
    pub signature: Option<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CommercePayment {
    #[schema(value_type = String)]
    pub id: Uuid,
    pub collected_by: Option<PaymentCollectedBy>,
    pub payment_type: PaymentType,
    pub buyer_fee_type: Option<FeeType>,
    pub buyer_fee_amount: Option<String>,
    pub settlement_basis: Option<SettlementBasis>,
    pub settlement_window: Option<String>,
    pub withholding_amount: Option<String>,
    pub seller_payment_detail: Option<SellerPaymentDetail>,
    pub settlement_details: Option<Vec<PaymentSettlementDetail>>,
}

#[derive(Deserialize, Debug, ToSchema, sqlx::Type, Serialize, Clone)]
#[sqlx(type_name = "commerce_fulfillment_status_type")]
#[sqlx(rename_all = "snake_case")]
pub enum FulfillmentStatusType {
    AgentAssigned,
    Packed,
    OutForDelivery,
    OrderPickedUp,
    SearchingForAgent,
    Pending,
    OrderDelivered,
    Cancelled,
}

impl FulfillmentStatusType {
    pub fn get_ondc_fulfillment_state(&self) -> ONDCFulfillmentStateType {
        match self {
            FulfillmentStatusType::AgentAssigned => ONDCFulfillmentStateType::AgentAssigned,
            FulfillmentStatusType::Packed => ONDCFulfillmentStateType::Packed,
            FulfillmentStatusType::OutForDelivery => ONDCFulfillmentStateType::OutForDelivery,
            FulfillmentStatusType::OrderPickedUp => ONDCFulfillmentStateType::OrderPickedUp,
            FulfillmentStatusType::SearchingForAgent => ONDCFulfillmentStateType::SearchingForAgent,
            FulfillmentStatusType::Pending => ONDCFulfillmentStateType::Pending,
            FulfillmentStatusType::OrderDelivered => ONDCFulfillmentStateType::OrderDelivered,
            FulfillmentStatusType::Cancelled => ONDCFulfillmentStateType::Cancelled,
        }
    }
}

impl std::fmt::Display for FulfillmentStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}
#[derive(Deserialize, Debug, ToSchema)]
pub struct DeliveryTerm {
    pub inco_terms: IncoTermType,
    pub place_of_delivery: String,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct FulfillmentLocation {
    pub gps: String,
    pub area_code: String,
    pub address: Option<String>,
    pub city: String,
    pub country: CountryCode,
    pub state: String,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct FulfillmentContact {
    pub mobile_no: String,
    pub email: Option<String>,
}
#[derive(Deserialize, Debug, Serialize, sqlx::FromRow)]
pub struct ExtOffContact {
    pub mobile_no: String,
    pub email: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct DropOffData {
    pub location: FulfillmentLocation,
    pub contact: FulfillmentContact,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct PickUpFulfillmentLocation {
    pub name: Option<String>,
    pub gps: String,
    pub area_code: String,
    pub address: String,
    pub city: String,
    pub country: CountryCode,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct PickUpData {
    pub location: PickUpFulfillmentLocation,
    pub time_range: Option<TimeRange>,
    pub contact: FulfillmentContact,
}
#[derive(Deserialize, Debug, ToSchema)]
pub struct CommerceFulfillment {
    pub id: String,
    pub fulfillment_id: String,
    pub fulfillment_type: FulfillmentType,
    pub tat: Option<String>,
    pub fulfillment_status: FulfillmentStatusType,
    pub delivery_term: Option<DeliveryTerm>,
    pub provider_name: Option<String>,
    pub category: Option<FulfillmentCategoryType>,
    pub servicable_status: Option<ServiceableType>,
    pub drop_off: Option<DropOffData>,
    pub pickup: PickUpData,
    pub tracking: Option<bool>,
    #[schema(value_type = f64)]
    pub packaging_charge: BigDecimal,
    #[schema(value_type = f64)]
    pub delivery_charge: BigDecimal,
    #[schema(value_type = f64)]
    pub convenience_fee: BigDecimal,
    pub trade_type: Option<TradeType>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct BuyerTerm {
    pub item_req: String,
    pub packaging_req: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommerceBilling {
    pub name: String,
    pub address: String,
    pub state: String,
    pub city: String,
    pub tax_id: String,
    pub email: Option<EmailObject>,
    pub phone: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CommerceItem {
    #[schema(value_type = String)]
    pub id: Uuid,
    pub item_id: String,
    pub item_name: String,
    pub item_code: Option<String>,
    pub item_image: String,
    #[schema(value_type = f64)]
    pub qty: BigDecimal,
    pub buyer_terms: Option<BuyerTerm>,
    #[schema(value_type = f64)]
    pub tax_rate: BigDecimal,
    #[schema(value_type = f64)]
    pub tax_value: BigDecimal,
    #[schema(value_type = f64)]
    pub unit_price: BigDecimal,
    #[schema(value_type = f64)]
    pub gross_total: BigDecimal,
    #[schema(value_type = Option<f64>)]
    pub available_qty: Option<BigDecimal>,
    #[schema(value_type = f64)]
    pub discount_amount: BigDecimal,
    pub location_ids: Vec<String>,
    pub fulfillment_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommerceCancellationFee {
    pub r#type: CancellationFeeType,
    #[schema(value_type = f64)]
    pub val: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommerceCancellationTerm {
    pub fulfillment_state: FulfillmentStatusType,
    pub reason_required: bool,
    pub cancellation_fee: CommerceCancellationFee,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CommerceBPPTerms {
    pub max_liability: String,
    pub max_liability_cap: String,
    pub mandatory_arbitration: bool,
    pub court_jurisdiction: String,
    pub delay_interest: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct Commerce {
    #[schema(value_type = String)]
    pub id: Uuid,
    pub urn: String,
    #[schema(value_type = String)]
    pub external_urn: Uuid,
    pub record_type: OrderType,
    pub record_status: CommerceStatusType,
    pub domain_category_code: CategoryDomain,
    pub seller: CommerceSeller,
    pub source: DataSource,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
    #[schema(value_type = String)]
    pub created_by: Uuid,
    #[schema(value_type = Option<f64>)]
    pub grand_total: Option<BigDecimal>,
    pub bap: BasicNetworkData,
    pub bpp: BasicNetworkData,
    pub quote_ttl: String,
    pub city_code: String,
    pub country_code: CountryCode,
    pub items: Vec<CommerceItem>,
    pub payments: Vec<CommercePayment>,
    pub fulfillments: Vec<CommerceFulfillment>,
    pub billing: Option<CommerceBilling>,
    pub cancellation_terms: Option<Vec<CommerceCancellationTerm>>,
    pub currency_type: Option<CurrencyType>,
    pub bpp_terms: Option<CommerceBPPTerms>,
    pub documents: Option<Vec<CommerceDocument>>,
    #[schema(value_type = String)]
    pub buyer_id: Uuid,
}

impl Commerce {
    pub fn get_ondc_location_ids(&self) -> Vec<&str> {
        let mut unique_ids = HashSet::new();

        self.items
            .iter()
            .flat_map(|item| item.location_ids.iter().map(|id| id.as_str()))
            .filter(|id| unique_ids.insert(*id))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSettlementCounterparty {
    BuyerApp,
    SellerApp,
}

impl PaymentSettlementCounterparty {
    pub fn get_ondc_settlement_counterparty(&self) -> ONDCPaymentSettlementCounterparty {
        match self {
            PaymentSettlementCounterparty::BuyerApp => ONDCPaymentSettlementCounterparty::BuyerApp,
            PaymentSettlementCounterparty::SellerApp => {
                ONDCPaymentSettlementCounterparty::SellerApp
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "payment_settlement_type", rename_all = "snake_case")]
pub enum PaymentSettlementPhase {
    SaleAmount,
}

impl PaymentSettlementPhase {
    pub fn get_ondc_settlement_phase(&self) -> ONDCPaymentSettlementPhase {
        match self {
            PaymentSettlementPhase::SaleAmount => ONDCPaymentSettlementPhase::SaleAmount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "payment_settlement_phase", rename_all = "snake_case")]
pub enum PaymentSettlementType {
    Neft,
}

impl PaymentSettlementType {
    pub fn get_ondc_settlement_type(&self) -> ONDCPaymentSettlementType {
        match self {
            PaymentSettlementType::Neft => ONDCPaymentSettlementType::Neft,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CancellationFeeType {
    Percent,
    Amount,
}

// #[derive(Deserialize, Debug, ToSchema)]
// #[serde(rename_all = "camelCase")]
// pub struct OrderConfirmPayment {
//     pub id: String,
//     #[schema(value_type = f64)]
//     pub amount: BigDecimal,
// }

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderConfirmRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    // pub payment: OrderConfirmPayment,
}

impl FromRequest for OrderConfirmRequest {
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

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "payment_status", rename_all = "snake_case")]
pub enum PaymentStatus {
    Paid,
    NotPaid,
    Pending,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
}
impl FromRequest for OrderStatusRequest {
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

#[derive(Deserialize, Debug, Serialize, ToSchema, PartialEq, Eq, Hash, Clone, sqlx::Type)]
pub enum DocumentType {
    Invoice,
    ProformaInvoice,
}

#[derive(Deserialize, Debug, Serialize, ToSchema, Clone, sqlx::Type)]
pub struct CommerceDocument {
    pub r#type: DocumentType,
    pub url: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancelRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub reason_id: String,
}
impl FromRequest for OrderCancelRequest {
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

#[derive(Deserialize, Debug, ToSchema, sqlx::Type, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "trade_type", rename_all = "snake_case")]
pub enum TradeType {
    Import,
    Domestic,
}

#[derive(Debug)]
pub struct BulkCancelFulfillmentData {
    pub commerce_ids: Vec<Uuid>,
    pub refunded_convenience_fees: Vec<BigDecimal>,
    pub fulfillment_ids: Vec<String>,
    pub refunded_delivery_charges: Vec<BigDecimal>,
    pub refunded_packaging_charges: Vec<BigDecimal>,
}

#[derive(Debug)]
pub struct BulkCancelItemData {
    pub commerce_ids: Vec<Uuid>,
    pub item_ids: Vec<String>,
    pub refunded_tax_values: Vec<BigDecimal>,
    pub refunded_discount_amounts: Vec<BigDecimal>,
    pub refunded_gross_totals: Vec<BigDecimal>,
}

#[derive(Debug)]
pub struct BulkConfirmFulfillmentData {
    pub fulfillment_statuses: Vec<FulfillmentStatusType>,
    pub pickup_datas: Vec<Value>,
    pub commerce_data_ids: Vec<Uuid>,
    pub fulfillment_ids: Vec<String>,
}

#[derive(Debug)]
pub struct BulkStatusFulfillmentData {
    pub fulfillment_statuses: Vec<FulfillmentStatusType>,
    pub pickup_datas: Vec<Value>,
    pub commerce_data_ids: Vec<Uuid>,
    pub fulfillment_ids: Vec<String>,
    pub drop_off_datas: Vec<Value>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderUpdateType {
    Payment,
    Item,
    Fulfillment,
}

impl OrderUpdateType {
    pub fn get_ondc_type(&self) -> ONDCOrderUpdateTarget {
        match self {
            OrderUpdateType::Payment => ONDCOrderUpdateTarget::Payment,
            OrderUpdateType::Item => ONDCOrderUpdateTarget::Item,
            OrderUpdateType::Fulfillment => ONDCOrderUpdateTarget::Fulfillment,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderPaymentRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub target_type: OrderUpdateType,
}
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderItemRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub target_type: OrderUpdateType,
    pub items: Vec<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderFulfillmentRequest {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub target_type: OrderUpdateType,
    pub fulfillments: Vec<OrderSelectFulfillment>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(untagged)]
#[allow(clippy::enum_variant_names)]
pub enum OrderUpdateRequest {
    UpdatePayment(UpdateOrderPaymentRequest),
    UpdateItem(UpdateOrderItemRequest),
    UpdateFulfillment(UpdateOrderFulfillmentRequest),
}

impl OrderUpdateRequest {
    pub fn transaction_id(&self) -> Uuid {
        match self {
            OrderUpdateRequest::UpdatePayment(request) => request.transaction_id,
            OrderUpdateRequest::UpdateItem(request) => request.transaction_id,
            OrderUpdateRequest::UpdateFulfillment(request) => request.transaction_id,
        }
    }
    pub fn message_id(&self) -> Uuid {
        match self {
            OrderUpdateRequest::UpdatePayment(request) => request.message_id,
            OrderUpdateRequest::UpdateItem(request) => request.message_id,
            OrderUpdateRequest::UpdateFulfillment(request) => request.message_id,
        }
    }
}

impl FromRequest for OrderUpdateRequest {
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
