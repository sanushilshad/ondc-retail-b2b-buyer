use std::str::FromStr;

use crate::routes::product::schemas::{CategoryDomain, FulfillmentType, PaymentType};
use crate::schemas::DataSource;
use crate::schemas::{CountryCode, CurrencyType, FeeType};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::schemas::{
    CancellationFeeType, CommerceList, CommerceSeller, CommerceStatusType, DocumentType,
    FulfillmentCategoryType, FulfillmentStatusType, IncoTermType, MinimalCommerceData, OrderType,
    PaymentCollectedBy, PaymentSettlementCounterparty, PaymentSettlementPhase,
    PaymentSettlementType, PaymentStatus, ServiceableType, SettlementBasis, TradeType,
};
use crate::domain::EmailObject;
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderCancellationFeeModel {
    pub r#type: CancellationFeeType,
    pub val: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
// #[sqlx(type_name = "order_cancellation_term_model")]
pub struct OrderCancellationTermModel {
    pub fulfillment_state: FulfillmentStatusType,
    pub reason_required: bool,
    pub cancellation_fee: OrderCancellationFeeModel,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CommerceBppTermsModel {
    pub max_liability: String,
    pub max_liability_cap: String,
    pub mandatory_arbitration: bool,
    pub court_jurisdiction: String,
    pub delay_interest: String,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug, FromRow)]
pub struct CommerceDataModel {
    pub id: Uuid,
    pub urn: String,
    pub external_urn: Uuid,
    pub record_type: OrderType,
    pub record_status: CommerceStatusType,
    pub domain_category_code: CategoryDomain,
    pub buyer_id: Uuid,
    pub seller_id: String,
    pub buyer_name: Option<String>,
    pub seller_name: Option<String>,
    pub source: DataSource,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
    pub deleted_on: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub created_by: Uuid,
    pub grand_total: Option<BigDecimal>,
    pub bpp_id: String,
    pub bpp_uri: String,
    pub bap_id: String,
    pub bap_uri: String,
    pub quote_ttl: String,
    pub currency_code: Option<CurrencyType>,
    pub city_code: String,
    pub country_code: CountryCode,
    pub billing: Option<sqlx::types::Json<OrderBillingModel>>,
    pub cancellation_terms: Option<sqlx::types::Json<Vec<OrderCancellationTermModel>>>,
    pub bpp_terms: Option<sqlx::types::Json<CommerceBppTermsModel>>,
    pub documents: Option<sqlx::types::Json<Vec<CommerceDocumentModel>>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, FromRow)]
pub struct CommerceItemModel {
    pub id: Uuid,
    pub item_id: String,
    pub commerce_data_id: Uuid,
    pub item_name: String,
    pub item_code: Option<String>,
    pub item_image: String,
    pub qty: BigDecimal,
    pub item_req: Option<String>,
    pub packaging_req: Option<String>,
    pub tax_rate: BigDecimal,
    pub tax_value: BigDecimal,
    pub unit_price: BigDecimal,
    pub gross_total: BigDecimal,
    pub available_qty: Option<BigDecimal>,
    pub discount_amount: BigDecimal,
    pub location_ids: Option<sqlx::types::Json<Vec<String>>>,
    pub fulfillment_ids: Option<sqlx::types::Json<Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaymentSettlementDetailModel {
    pub settlement_counterparty: PaymentSettlementCounterparty,
    pub settlement_phase: PaymentSettlementPhase,
    pub settlement_type: PaymentSettlementType,
    pub settlement_bank_account_no: String,
    pub settlement_ifsc_code: String,
    pub beneficiary_name: String,
    pub bank_name: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SellerPaymentDetailModel {
    pub uri: String,
    pub ttl: Option<String>,
    pub dsa: Option<String>,
    pub signature: Option<String>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CommercePaymentModel {
    pub id: Uuid,
    pub collected_by: Option<PaymentCollectedBy>,
    pub payment_type: PaymentType,
    pub commerce_data_id: Uuid,
    pub buyer_fee_type: Option<FeeType>,
    pub buyer_fee_amount: Option<BigDecimal>,
    pub settlement_basis: Option<SettlementBasis>,
    pub settlement_window: Option<String>,
    pub withholding_amount: Option<BigDecimal>,
    pub settlement_details: Option<sqlx::types::Json<Vec<PaymentSettlementDetailModel>>>,
    pub seller_payment_detail: Option<sqlx::types::Json<SellerPaymentDetailModel>>,
    pub payment_status: Option<PaymentStatus>,
    pub payment_order_id: Option<String>,
    pub payment_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct CommerceFulfillmentModel {
    pub id: String,
    pub commerce_data_id: Uuid,
    pub fulfillment_id: String,
    pub fulfillment_type: FulfillmentType,
    pub tat: Option<String>,
    pub fulfillment_status: FulfillmentStatusType,
    pub inco_terms: Option<IncoTermType>,
    pub place_of_delivery: Option<String>,
    pub provider_name: Option<String>,
    pub category: Option<FulfillmentCategoryType>,
    pub servicable_status: Option<ServiceableType>,
    pub drop_off_data: sqlx::types::Json<Option<DropOffDataModel>>,
    pub pickup_data: sqlx::types::Json<PickUpDataModel>,
    pub tracking: Option<bool>,
    pub packaging_charge: BigDecimal,
    pub delivery_charge: BigDecimal,
    pub convenience_fee: BigDecimal,
    pub trade_type: Option<TradeType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderBillingModel {
    pub name: String,
    pub address: String,
    pub state: String,
    pub city: String,
    pub tax_id: String,
    pub email: Option<EmailObject>,
    pub phone: String,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Clone)]
pub struct DropOffLocationModel {
    pub gps: String,
    pub area_code: String,
    pub address: Option<String>,
    pub city: String,
    pub country: CountryCode,
    pub state: String,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Clone)]
pub struct DropOffContactModel {
    pub mobile_no: String,
    pub email: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Clone)]
pub struct DropOffDataModel {
    pub location: DropOffLocationModel,
    pub contact: DropOffContactModel,
    pub time_range: Option<TimeRangeModel>,
    pub instruction: Option<FulfillmentInstruction>,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Clone)]
pub struct PickUpContactModel {
    pub mobile_no: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeRangeModel {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]

pub struct FulfillmentInstruction {
    pub short_desc: String,
    pub long_desc: String,
    pub name: String,
    pub images: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Clone)]
pub struct PickUpDataModel {
    pub location: PickUpLocationModel,
    pub contact: PickUpContactModel,
    pub time_range: Option<TimeRangeModel>,
    pub instruction: Option<FulfillmentInstruction>,
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Clone)]
pub struct PickUpLocationModel {
    pub gps: String,
    pub area_code: String,
    pub address: String,
    pub city: String,
    pub country: CountryCode,
    pub state: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CommerceDocumentModel {
    pub r#type: DocumentType,
    pub url: String,
}

#[derive(Deserialize, Debug, FromRow)]
// #[serde(rename_all = "snake_C")]
pub struct CommerceListModel {
    pub id: Uuid,
    pub external_urn: Uuid,
    pub urn: String,
    pub currency_code: CurrencyType,
    pub grand_total: Option<BigDecimal>,
    pub record_status: CommerceStatusType,
    pub created_on: DateTime<Utc>,
    pub seller_id: String,
    pub seller_name: Option<String>,
    pub created_by: Uuid,
    pub buyer_id: Uuid,
    pub record_type: OrderType,
}

impl CommerceListModel {
    pub fn schema(self) -> CommerceList {
        CommerceList {
            id: self.id,
            external_urn: self.external_urn,
            urn: self.urn,
            currency_code: self.currency_code,
            grand_total: self
                .grand_total
                .unwrap_or(BigDecimal::from_str("0.00").unwrap()),
            record_status: self.record_status,
            created_on: self.created_on,
            buyer_id: self.buyer_id,
            created_by: self.created_by,
            seller: CommerceSeller {
                name: self.seller_name,
                id: self.seller_id,
            },
            record_type: self.record_type,
        }
    }
}

#[derive(Deserialize, Debug, FromRow)]
// #[serde(rename_all = "snake_C")]
pub struct MinimalCommerceModel {
    pub id: Uuid,
    pub external_urn: Uuid,
    pub urn: String,
    pub currency_code: CurrencyType,
    pub grand_total: Option<BigDecimal>,
    pub record_status: CommerceStatusType,
    pub created_on: DateTime<Utc>,
    pub seller_id: String,
    pub seller_name: Option<String>,
    pub created_by: Uuid,
    pub buyer_id: Uuid,
    pub record_type: OrderType,
}

impl MinimalCommerceModel {
    pub fn schema(self) -> MinimalCommerceData {
        MinimalCommerceData {
            id: self.id,
            external_urn: self.external_urn,
            urn: self.urn,
            currency_code: self.currency_code,
            grand_total: self
                .grand_total
                .unwrap_or(BigDecimal::from_str("0.00").unwrap()),
            record_status: self.record_status,
            created_on: self.created_on,
            buyer_id: self.buyer_id,
            created_by: self.created_by,
            seller: CommerceSeller {
                name: self.seller_name,
                id: self.seller_id,
            },
            record_type: self.record_type,
        }
    }
}
