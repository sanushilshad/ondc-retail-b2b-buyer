use crate::routes::product::schemas::{CategoryDomain, FulfillmentType, PaymentType};
use crate::routes::user::schemas::DataSource;
use crate::schemas::{CountryCode, CurrencyType, FeeType, ONDCNetworkType};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::schemas::{
    CancellationFeeType, CommerceFulfillmentStatusType, CommerceStatusType, DropOffDataModel,
    FulfillmentCategoryType, IncoTermType, OrderBillingModel, OrderType,
    PaymentSettlementCounterparty, PaymentSettlementPhase, PaymentSettlementType, PickUpDataModel,
    ServiceableType, SettlementBasis,
};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderCancellationFeeModel {
    pub r#type: CancellationFeeType,
    pub val: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
// #[sqlx(type_name = "order_cancellation_term_model")]
pub struct OrderCancellationTermModel {
    pub fulfillment_state: CommerceFulfillmentStatusType,
    pub reason_required: bool,
    pub cancellation_fee: OrderCancellationFeeModel,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BuyerCommerceBppTermsModel {
    pub max_liability: String,
    pub max_liability_cap: String,
    pub mandatory_arbitration: bool,
    pub court_jurisdiction: String,
    pub delay_interest: String,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug, FromRow)]
pub struct BuyerCommerceDataModel {
    pub id: Uuid,
    pub urn: Option<String>,
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
    pub is_import: bool,
    pub quote_ttl: String,
    pub currency_code: Option<CurrencyType>,
    pub city_code: String,
    pub country_code: CountryCode,
    pub billing: Option<sqlx::types::Json<OrderBillingModel>>,
    pub cancellation_terms: Option<sqlx::types::Json<Vec<OrderCancellationTermModel>>>,
    pub bpp_terms: Option<sqlx::types::Json<BuyerCommerceBppTermsModel>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, FromRow)]
pub struct BuyerCommerceItemModel {
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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BuyerCommercePaymentModel {
    pub id: Uuid,
    pub collected_by: Option<ONDCNetworkType>,
    pub payment_type: PaymentType,
    pub commerce_data_id: Uuid,
    pub seller_payment_uri: Option<String>,
    pub buyer_fee_type: Option<FeeType>,
    pub buyer_fee_amount: Option<BigDecimal>,
    pub settlement_basis: Option<SettlementBasis>,
    pub settlement_window: Option<String>,
    pub withholding_amount: Option<BigDecimal>,
    pub settlement_details: Option<sqlx::types::Json<Vec<PaymentSettlementDetailModel>>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BuyerCommerceFulfillmentModel {
    pub id: String,
    pub commerce_data_id: Uuid,
    pub fulfillment_id: String,
    pub fulfillment_type: FulfillmentType,
    pub tat: Option<String>,
    pub fulfillment_status: CommerceFulfillmentStatusType,
    pub inco_terms: Option<IncoTermType>,
    pub place_of_delivery: Option<String>,
    pub provider_name: Option<String>,
    pub category: Option<FulfillmentCategoryType>,
    pub servicable_status: Option<ServiceableType>,
    pub drop_off_data: sqlx::types::Json<Option<DropOffDataModel>>,
    pub pickup_data: sqlx::types::Json<Option<PickUpDataModel>>,
    pub tracking: Option<bool>,
    pub packaging_charge: BigDecimal,
    pub delivery_charge: BigDecimal,
    pub convenience_fee: BigDecimal,
}
