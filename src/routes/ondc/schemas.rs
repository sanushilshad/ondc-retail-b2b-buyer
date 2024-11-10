use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;
use uuid::Uuid;

use super::utils::serialize_timestamp_without_nanos;
use crate::{
    routes::{order::schemas::OrderType, product::schemas::CategoryDomain},
    schemas::{CountryCode, ONDCNetworkType},
    utils::pascal_to_snake_case,
};

use super::errors::ONDCBuyerError;
use crate::domain::EmailObject;

use crate::routes::order::models::PaymentSettlementDetailModel;
use crate::routes::order::schemas::{
    CancellationFeeType, CommerceBPPTerms, CommerceStatusType, DocumentType,
    FulfillmentCategoryType, FulfillmentStatusType, IncoTermType, Payment, PaymentCollectedBy,
    PaymentSettlementCounterparty, PaymentSettlementPhase, PaymentSettlementType, PaymentStatus,
    ServiceableType, SettlementBasis,
};
use crate::routes::product::schemas::{FulfillmentType, PaymentType};
use crate::schemas::{CurrencyType, FeeType};

use crate::websocket_client::WebSocketActionType;
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use bigdecimal::BigDecimal;
use futures_util::future::LocalBoxFuture;

use serde_json::Value;

use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCVersion {
    #[serde(rename = "2.0.2")]
    V2point2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ONDCActionType {
    Search,
    OnSearch,
    Select,
    OnSelect, // Pending
    Init,
    OnInit, // Pending
    Confirm,
    OnConfirm, // Pending
    Cancel,
    OnCancel, // Pending
    Status,
    OnStatus, // Pending
    Track,
    OnTrack, // Pending
    Update,
    OnUpdate, // Pending
    Issue,
    OnIssue, // Pending
    IssueStatus,
    OnIssueStatus, // Pending
}

impl Display for ONDCActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
pub enum ONDCDomain {
    #[serde(rename = "ONDC:RET10")]
    #[sqlx(rename = "ONDC:RET10")]
    Grocery,
    #[serde(rename = "ONDC:RET12")]
    #[sqlx(rename = "ONDC:RET12")]
    Fashion,
    #[serde(rename = "ONDC:RET13")]
    #[sqlx(rename = "ONDC:RET13")]
    Bpc,
    #[serde(rename = "ONDC:RET14")]
    #[sqlx(rename = "ONDC:RET14")]
    Electronics,
    #[serde(rename = "ONDC:RET15")]
    #[sqlx(rename = "ONDC:RET15")]
    Appliances,
    #[serde(rename = "ONDC:RET16")]
    #[sqlx(rename = "ONDC:RET16")]
    HomeAndKitchen,
    #[serde(rename = "ONDC:RET1A")]
    #[sqlx(rename = "ONDC:RET1A")]
    AutoComponentsAndAccessories,
    #[serde(rename = "ONDC:RET1B")]
    #[sqlx(rename = "ONDC:RET1B")]
    HardwareAndIndustrialEquipments,
    #[serde(rename = "ONDC:RET1C")]
    #[sqlx(rename = "ONDC:RET1C")]
    BuildingAndConstructionSupplies,
}

impl ONDCDomain {
    pub fn get_category_domain(&self) -> CategoryDomain {
        match self {
            ONDCDomain::Grocery => CategoryDomain::Grocery,
            ONDCDomain::Fashion => CategoryDomain::Fashion,
            ONDCDomain::Bpc => CategoryDomain::Bpc,
            ONDCDomain::Electronics => CategoryDomain::Electronics,
            ONDCDomain::Appliances => CategoryDomain::Appliances,
            ONDCDomain::HomeAndKitchen => CategoryDomain::HomeAndKitchen,
            ONDCDomain::AutoComponentsAndAccessories => {
                CategoryDomain::AutoComponentsAndAccessories
            }
            ONDCDomain::HardwareAndIndustrialEquipments => {
                CategoryDomain::HardwareAndIndustrialEquipments
            }
            ONDCDomain::BuildingAndConstructionSupplies => {
                CategoryDomain::BuildingAndConstructionSupplies
            }
        }
    }
}

impl Display for ONDCDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ONDC:{}", self.get_category_domain())
    }
}

impl ONDCDomain {
    pub fn get_ondc_domain(domain_category_code: &CategoryDomain) -> Self {
        match domain_category_code {
            CategoryDomain::Grocery => ONDCDomain::Grocery,
            CategoryDomain::Fashion => ONDCDomain::Fashion,
            CategoryDomain::Bpc => ONDCDomain::Bpc,
            CategoryDomain::Electronics => ONDCDomain::Electronics,
            CategoryDomain::Appliances => ONDCDomain::Appliances,
            CategoryDomain::HomeAndKitchen => ONDCDomain::HomeAndKitchen,
            CategoryDomain::AutoComponentsAndAccessories => {
                ONDCDomain::AutoComponentsAndAccessories
            }
            CategoryDomain::HardwareAndIndustrialEquipments => {
                ONDCDomain::HardwareAndIndustrialEquipments
            }
            CategoryDomain::BuildingAndConstructionSupplies => {
                ONDCDomain::BuildingAndConstructionSupplies
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContextCountry {
    pub code: CountryCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContextCity {
    pub code: String,
}

impl ONDCContextCity {
    fn _default() -> Self {
        ONDCContextCity {
            code: "std:080".to_string(),
        }
    }
}

impl ONDCContextCountry {
    fn _default() -> Self {
        ONDCContextCountry {
            code: CountryCode::IND,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContextLocation {
    pub city: ONDCContextCity,
    pub country: ONDCContextCountry,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContext {
    pub domain: ONDCDomain,
    pub location: ONDCContextLocation,
    pub action: ONDCActionType,
    pub version: ONDCVersion,
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub timestamp: DateTime<Utc>,
    pub bap_id: String,
    pub bap_uri: String,
    pub bpp_id: Option<String>,
    pub bpp_uri: Option<String>,
    pub ttl: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ONDCResponseStatusType {
    Ack,
    Nack,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCGateWayErrorCode {
    #[serde(rename = "10000")]
    GateWayInvalidRequest,
    #[serde(rename = "10001")]
    GateWayInvalidSignature,
    #[serde(rename = "10002")]
    GateWayInvalidCityCode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCBuyerErrorCode {
    #[serde(rename = "23001")]
    InternalErrorCode,
    #[serde(rename = "20008")]
    ResponseSequenceCode,
    #[serde(rename = "20001")]
    InvalidSignatureCode,
    #[serde(rename = "20002")]
    StaleRequestCode,
    #[serde(rename = "20006")]
    InvalidResponseCode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCSellerErrorCode {
    #[serde(rename = "30016")]
    SellerInvalidSignatureCode,
    #[serde(rename = "30022")]
    SellerStaleRequestCode,
    #[serde(rename = "30000")]
    SellerInvalidRequestCode,
    #[serde(rename = "40000")]
    SellerBusinessErrorCode,
    #[serde(rename = "30001")]
    SellerProviderNotFoundError,
    #[serde(rename = "30009")]
    SellerServiceabilityError,
    #[serde(rename = "31004")]
    SellerPaymentFailure,
    #[serde(rename = "50001")]
    SellerCancelNotPossibleError,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
pub enum LookUpErrorCode {
    #[serde(rename = "151")]
    InvalidRequestCode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ONDCErrorCode {
    GatewayError(ONDCGateWayErrorCode),
    BuyerError(ONDCBuyerErrorCode),
    SellerError(ONDCSellerErrorCode),
    LookUpError(LookUpErrorCode),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ONDErrorType {
    #[serde(rename = "Gateway")]
    Gateway,
    ContextError,
    DomainError,
    PolicyError,
    JsonSchemaError,
    CoreError,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ONDCResponseAck {
    pub status: ONDCResponseStatusType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ONDCResponseMessage {
    pub ack: ONDCResponseAck,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCResponseErrorBody<D> {
    pub r#type: ONDErrorType,
    pub code: D,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ONDCResponse<D> {
    // pub context: Option<ONDCContext>,
    pub message: ONDCResponseMessage,
    pub error: Option<ONDCResponseErrorBody<D>>,
}

impl<D> ONDCResponse<D> {
    pub fn successful_response(_context: Option<ONDCContext>) -> Self {
        Self {
            message: ONDCResponseMessage {
                ack: ONDCResponseAck {
                    status: ONDCResponseStatusType::Ack,
                },
            },
            // context: context,
            error: None,
        }
    }

    pub fn error_response(_context: Option<ONDCContext>, error: ONDCResponseErrorBody<D>) -> Self {
        Self {
            message: ONDCResponseMessage {
                ack: ONDCResponseAck {
                    status: ONDCResponseStatusType::Nack,
                },
            },
            // context: context,
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum OndcUrl {
    #[serde(rename = "/lookup")]
    LookUp,
}

impl Display for OndcUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "/{}",
            match self {
                OndcUrl::LookUp => "lookup",
            }
        )
    }
}

#[derive(Debug, Serialize)]
pub struct LookupRequest<'a> {
    pub subscriber_id: &'a str,
    pub domain: &'a ONDCDomain,
    pub r#type: &'a ONDCNetworkType,
}

#[derive(Debug, Deserialize)]
pub struct LookupData {
    pub br_id: String,
    pub subscriber_id: String,
    pub signing_public_key: String,
    pub subscriber_url: String,
    pub encr_public_key: String,
    #[serde(rename = "ukId")]
    pub uk_id: String,
    pub domain: ONDCDomain,
    pub r#type: ONDCNetworkType,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ONDCItemUOM {
    Unit,
    Dozen,
    Gram,
    Kilogram,
    Tonne,
    Litre,
    Millilitre,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ONDCTagType {
    BuyerId,
    BapTerms,
    Serviceability,
    SellerTerms,
    SellerId,
    #[serde(rename = "FSSAI_LICENSE_NO")]
    FssaiLicenseNo,
    Type,
    Attr,
    Origin,
    Image,
    VegNonveg,
    G2,
    G3,
    PriceSlab,
    Attribute,
    BackImage,
    #[serde(rename = "DELIVERY_TERMS")]
    DeliveyTerms,
    #[serde(rename = "BUYER_TERMS")]
    BuyerTerms,
    #[serde(rename = "BPP_payment")]
    BPPPayment,
    BppTerms,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCTagDescriptor {
    pub code: ONDCTagType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ONDCTagItemCode {
    BuyerIdCode,
    BuyerIdNo,
    FinderFeeType,
    FinderFeeAmount,
    Location,
    Category,
    Type,
    Val,
    Unit,
    GstCreditInvoice,
    SellerIdCode,
    SellerIdNo,
    #[serde(rename = "BRAND_OWNER")]
    BrandOwner,
    #[serde(rename = "OTHER")]
    Other,
    #[serde(rename = "IMPORTER")]
    Importer,
    Name,
    Seq,
    Country,
    Url,
    Veg,
    Nonveg,
    TimeToShip,
    TaxRate,
    Cancellable,
    Brand,
    PackSize,
    NumPriceSlabs,
    MinPackSize,
    MaxPackSize,
    UnitSalePrice,
    #[serde(rename = "INCOTERMS")]
    IncoTerms,
    #[serde(rename = "NAMED_PLACE_OF_DELIVERY")]
    NamedPlaceOfDelivery,
    #[serde(rename = "ITEM_REQ")]
    ItemReq,
    #[serde(rename = "PACKAGING_REQ")]
    PackagingsReq,
    Ttl,
    Signature,
    Dsa,
    MaxLiability,
    MaxLiabilityCap,
    MandatoryArbitration,
    CourtJurisdiction,
    DelayInterest,
    AcceptBppTerms,
}

impl std::fmt::Display for ONDCTagItemCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

#[derive(Debug, Serialize, Deserialize)]

pub enum ONDCFulfillmentStateType {
    #[serde(rename = "Agent-assigned")]
    AgentAssigned,
    #[serde(rename = "Packed")]
    Packed,
    #[serde(rename = "Out-for-delivery")]
    OutForDelivery,
    #[serde(rename = "Order-picked-up")]
    OrderPickedUp,
    #[serde(rename = "Searching-for-agent")]
    SearchingForAgent,
    #[serde(rename = "Pending")]
    Pending,
    #[serde(rename = "Order-delivered")]
    OrderDelivered,
    #[serde(rename = "Cancelled")]
    Cancelled,
}

impl ONDCFulfillmentStateType {
    pub fn get_fulfillment_state(&self) -> FulfillmentStatusType {
        match self {
            ONDCFulfillmentStateType::AgentAssigned => FulfillmentStatusType::AgentAssigned,
            ONDCFulfillmentStateType::Packed => FulfillmentStatusType::Packed,
            ONDCFulfillmentStateType::OutForDelivery => FulfillmentStatusType::OutForDelivery,
            ONDCFulfillmentStateType::OrderPickedUp => FulfillmentStatusType::OrderPickedUp,
            ONDCFulfillmentStateType::SearchingForAgent => FulfillmentStatusType::SearchingForAgent,
            ONDCFulfillmentStateType::Pending => FulfillmentStatusType::Pending,
            ONDCFulfillmentStateType::OrderDelivered => FulfillmentStatusType::OrderDelivered,
            ONDCFulfillmentStateType::Cancelled => FulfillmentStatusType::Cancelled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ONDCFulfillmentStopType {
    Start,
    End,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCTagItemDescriptor {
    pub code: ONDCTagItemCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCTagItem {
    pub descriptor: ONDCTagItemDescriptor,
    pub value: String,
}

impl ONDCTagItem {
    pub fn set_tag_item(descriptor_code: ONDCTagItemCode, tag_item_code: &str) -> ONDCTagItem {
        ONDCTagItem {
            descriptor: ONDCTagItemDescriptor {
                code: descriptor_code,
            },
            value: tag_item_code.to_string(),
        }
    }
    // pub fn get_buyer_fee_type(fee_type: ONDCFeeType) -> ONDCTagItem {
    //     ONDCTagItem {
    //         descriptor: ONDCTagItemDescriptor {
    //             code: ONDCTagItemCode::FinderFeeType,
    //         },
    //         value: fee_type.to_string(),
    //     }
    // }
    // pub fn get_buyer_fee_amount(fee_amount: &str) -> ONDCTagItem {
    //     ONDCTagItem {
    //         descriptor: ONDCTagItemDescriptor {
    //             code: ONDCTagItemCode::FinderFeeAmount,
    //         },
    //         value: fee_amount.to_string(),
    //     }
    // }

    // pub fn get_buyer_id_code(id_code: &ONDCBuyerIdType) -> ONDCTagItem {
    //     ONDCTagItem {
    //         descriptor: ONDCTagItemDescriptor {
    //             code: ONDCTagItemCode::BuyerIdCode,
    //         },
    //         value: id_code.to_string(),
    //     }
    // }
    // pub fn get_buyer_id_no(id_no: &str) -> ONDCTagItem {
    //     ONDCTagItem {
    //         descriptor: ONDCTagItemDescriptor {
    //             code: ONDCTagItemCode::BuyerIdNo,
    //         },
    //         value: id_no.to_string(),
    //     }
    // }

    // pub fn get_delivery_incoterm(inco_term_type: IncoTermType) -> ONDCTagItem {
    //     ONDCTagItem {
    //         descriptor: ONDCTagItemDescriptor {
    //             code: ONDCTagItemCode::FinderFeeType,
    //         },
    //         value: inco_term_type.to_string(),
    //     }
    // }
    // pub fn get_buyer_fee_amount(fee_amount: &str) -> ONDCTagItem {
    //     ONDCTagItem {
    //         descriptor: ONDCTagItemDescriptor {
    //             code: ONDCTagItemCode::FinderFeeAmount,
    //         },
    //         value: fee_amount.to_owned(),
    //     }
    // }
}

// #[derive(Debug, Serialize)]
// #[serde(rename_all = "lowercase")]
// pub enum ONDCTagType{
//     BuyerId,
//     BapTerms
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ONDCFeeType {
    Percent,
    Amount,
}

impl std::fmt::Display for ONDCFeeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ONDCFeeType::Percent => "percent",
            ONDCFeeType::Amount => "amount",
        };

        write!(f, "{}", s)
    }
}

impl ONDCFeeType {
    pub fn get_fee_type(fee_type: &FeeType) -> Self {
        match fee_type {
            FeeType::Percent => ONDCFeeType::Percent,
            FeeType::Amount => ONDCFeeType::Amount,
        }
    }
}

pub trait TagTrait {
    fn get_tag_value(&self, item_code: &str) -> Option<&str>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCTag {
    pub descriptor: ONDCTagDescriptor,
    pub list: Vec<ONDCTagItem>,
}

impl TagTrait for ONDCTag {
    fn get_tag_value(&self, tag_item_code: &str) -> Option<&str> {
        self.list
            .iter()
            .find(|item| item.descriptor.code.to_string() == tag_item_code)
            .map(|item| &item.value)
            .map(|x| x.as_str())
    }
}

impl ONDCTag {
    pub fn get_buyer_fee_tag(finder_fee_type: ONDCFeeType, finder_fee_amount: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BapTerms,
            },
            list: vec![
                ONDCTagItem::set_tag_item(
                    ONDCTagItemCode::FinderFeeType,
                    &finder_fee_type.to_string(),
                ),
                ONDCTagItem::set_tag_item(ONDCTagItemCode::FinderFeeAmount, finder_fee_amount),
            ],
        }
    }

    pub fn get_buyer_id_tag(id_code: &ONDCBuyerIdType, id_no: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BuyerId,
            },
            list: vec![
                ONDCTagItem::set_tag_item(ONDCTagItemCode::BuyerIdCode, &id_code.to_string()),
                ONDCTagItem::set_tag_item(ONDCTagItemCode::BuyerIdNo, id_no),
            ],
        }
    }

    pub fn get_delivery_terms(inco_term: &IncoTermType, place_of_delivery: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::DeliveyTerms,
            },
            list: vec![
                ONDCTagItem::set_tag_item(ONDCTagItemCode::IncoTerms, &inco_term.to_string()),
                ONDCTagItem::set_tag_item(ONDCTagItemCode::NamedPlaceOfDelivery, place_of_delivery),
            ],
        }
    }

    pub fn get_item_tags(item_terms: &str, packaging_req: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BuyerTerms,
            },
            list: vec![
                ONDCTagItem::set_tag_item(ONDCTagItemCode::PackagingsReq, item_terms),
                ONDCTagItem::set_tag_item(ONDCTagItemCode::ItemReq, packaging_req),
            ],
        }
    }

    pub fn get_bpp_terms_tag(commerce_bpp_term: &CommerceBPPTerms) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BppTerms,
            },
            list: vec![
                ONDCTagItem::set_tag_item(
                    ONDCTagItemCode::MaxLiability,
                    &commerce_bpp_term.max_liability,
                ),
                ONDCTagItem::set_tag_item(
                    ONDCTagItemCode::MaxLiabilityCap,
                    &commerce_bpp_term.max_liability_cap,
                ),
                ONDCTagItem::set_tag_item(
                    ONDCTagItemCode::MandatoryArbitration,
                    &commerce_bpp_term.mandatory_arbitration.to_string(),
                ),
                ONDCTagItem::set_tag_item(
                    ONDCTagItemCode::CourtJurisdiction,
                    &commerce_bpp_term.court_jurisdiction,
                ),
                ONDCTagItem::set_tag_item(
                    ONDCTagItemCode::DelayInterest,
                    &commerce_bpp_term.delay_interest,
                ),
            ],
        }
    }
    pub fn get_bap_agreement_to_bpp_terms_tag(agree: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BapTerms,
            },
            list: vec![ONDCTagItem::set_tag_item(
                ONDCTagItemCode::AcceptBppTerms,
                agree,
            )],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchItemTag {
    pub descriptor: ONDCTagDescriptor,
    pub list: Vec<ONDCOnSearchItemTagItem>,
}

impl TagTrait for ONDCOnSearchItemTag {
    fn get_tag_value(&self, tag_item_code: &str) -> Option<&str> {
        self.list
            .iter()
            .find(|item| item.descriptor.code == tag_item_code)
            .map(|item| &item.value)
            .map(|x| x.as_str())
        // Some("".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchItemTagItemDescriptor {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchItemTagItem {
    pub descriptor: ONDCOnSearchItemTagItemDescriptor,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchLocation {
    pub gps: String,
    pub area_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchStop {
    pub r#type: ONDCFulfillmentStopType,
    pub location: ONDCSearchLocation,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ONDCFulfillmentType {
    #[serde(rename = "Delivery")]
    Delivery,
    #[serde(rename = "Self-Pickup")]
    SelfPickup,
}

impl ONDCFulfillmentType {
    // pub fn get_ondc_fulfillment(payment_type: &FulfillmentType) -> ONDCFulfillmentType {
    //     match payment_type {
    //         FulfillmentType::Delivery => ONDCFulfillmentType::Delivery,
    //         FulfillmentType::SelfPickup => ONDCFulfillmentType::SelfPickup,
    //     }
    // }

    pub fn get_fulfillment_from_ondc(&self) -> FulfillmentType {
        match &self {
            ONDCFulfillmentType::Delivery => FulfillmentType::Delivery,
            ONDCFulfillmentType::SelfPickup => FulfillmentType::SelfPickup,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchFulfillment {
    pub r#type: ONDCFulfillmentType,
    pub stops: Option<Vec<ONDCSearchStop>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchDescriptor {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchCategory {
    pub id: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchItem {
    pub descriptor: Option<ONDCSearchDescriptor>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize, PartialEq)]

pub enum ONDCPaymentType {
    #[serde(rename = "PRE-FULFILLMENT")]
    PreFulfillment,
    #[serde(rename = "ON-FULFILLMENT")]
    OnFulfillment,
    #[serde(rename = "POST-FULFILLMENT")]
    PostFulfillment,
}

impl ONDCPaymentType {
    pub fn get_payment(&self) -> PaymentType {
        match self {
            ONDCPaymentType::OnFulfillment => PaymentType::CashOnDelivery,
            ONDCPaymentType::PreFulfillment => PaymentType::PrePaid,
            ONDCPaymentType::PostFulfillment => PaymentType::Credit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchPayment {
    pub r#type: ONDCPaymentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchProvider {
    id: String,
}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchIntent {
    pub item: Option<ONDCSearchItem>,
    pub fulfillment: Option<ONDCSearchFulfillment>,
    pub tags: Vec<ONDCTag>,
    pub payment: Option<ONDCSearchPayment>,
    pub provider: Option<ONDCSearchProvider>,
    pub category: Option<ONDCSearchCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchMessage {
    pub intent: ONDCSearchIntent,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSearchRequest {
    pub context: ONDCContext,
    pub message: ONDCSearchMessage,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum OnSearchContentType {
    #[serde(rename = "text/html")]
    Html,
    #[serde(rename = "video/mp4")]
    Mp4,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchAdditionalDescriptor {
    pub url: String,
    pub content_type: OnSearchContentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCImage {
    url: String,
}

impl ONDCImage {
    pub fn get_value(&self) -> &str {
        &self.url
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchDescriptor {
    pub name: String,
    pub code: Option<String>,
    pub short_desc: String,
    pub long_desc: String,
    pub images: Vec<ONDCImage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchPayment {
    pub id: String,
    pub r#type: ONDCPaymentType,
    pub collected_by: Option<ONDCNetworkType>,
}

impl ONDCOnSearchPayment {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchFullFillment {
    pub id: String,
    pub r#type: ONDCFulfillmentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCredential {
    pub id: String,
    pub r#type: String,
    pub desc: String,
    pub icon: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchProviderDescriptor {
    pub name: String,
    pub code: String,
    pub short_desc: String,
    pub long_desc: String,
    pub additional_desc: Option<ONDCOnSearchAdditionalDescriptor>,
    pub images: Vec<ONDCImage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchCountry {
    pub code: CountryCode,
    pub name: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchState {
    pub code: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchCity {
    pub code: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCMedia {
    mimetype: OnSearchContentType,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchItemDescriptor {
    pub name: String,
    pub code: Option<String>,
    short_desc: String,
    long_desc: String,
    pub images: Vec<ONDCImage>,
    media: Option<Vec<ONDCMedia>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchCreatorAddress {
    pub full: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCItemCreatorContact {
    pub name: String,
    pub address: ONDCOnSearchCreatorAddress,
    pub phone: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCItemCreatorDescriptor {
    pub name: String,
    pub contact: ONDCItemCreatorContact,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchItemCreator {
    pub descriptor: ONDCItemCreatorDescriptor,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchItemPrice {
    pub currency: CurrencyType,
    pub value: String,
    pub offered_value: Option<String>,
    pub maximum_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchQtyMeasure {
    pub unit: ONDCItemUOM,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchQtUnitized {
    pub measure: ONDCOnSearchQtyMeasure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchQty {
    pub measure: ONDCOnSearchQtyMeasure,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchItemQuantity {
    pub unitized: ONDCOnSearchQtUnitized,
    pub available: ONDCOnSearchQty,
    pub maximum: ONDCOnSearchQty,
    pub minimum: Option<ONDCOnSearchQty>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchItemAddOns {
    id: String,
    descriptor: ONDCOnSearchItemDescriptor,
    price: ONDCOnSearchItemPrice,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCItemReplacementTerm {
    replace_within: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCFulfillmentDescriptor {
    pub code: ONDCFulfillmentStateType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCFulfillmentState {
    pub descriptor: ONDCFulfillmentDescriptor,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCItemReturnTime {
    duration: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCItemReturnLocation {
    address: String,
    gps: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCReturnTerm {
    fulfillment_state: ONDCFulfillmentState,
    return_eligible: bool,
    return_time: ONDCItemReturnTime,
    return_location: ONDCItemReturnLocation,
    fulfillment_managed_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCAmount {
    pub currency: CurrencyType,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ONDCItemCancellationFee {
    Percentage { percentage: String },
    Amount { amount: ONDCAmount },
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCItemCancellationTerm {
    fulfillment_state: ONDCFulfillmentState,
    reason_required: bool,
    cancellation_fee: ONDCItemCancellationFee,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchItem {
    pub id: String,
    pub parent_item_id: Option<String>,
    matched: bool,
    pub recommended: bool,
    pub descriptor: ONDCOnSearchItemDescriptor,
    pub creator: ONDCOnSearchItemCreator,
    pub category_ids: Vec<String>,
    pub fulfillment_ids: Vec<String>,
    pub location_ids: Vec<String>,
    pub payment_ids: Vec<String>,
    pub price: ONDCOnSearchItemPrice,
    pub quantity: ONDCOnSearchItemQuantity,
    add_ons: Option<Vec<ONDCOnSearchItemAddOns>>,
    time: Option<ONDCTime>,
    replacement_terms: Vec<ONDCItemReplacementTerm>,
    return_terms: Vec<ONDCReturnTerm>,
    cancellation_terms: Vec<ONDCItemCancellationTerm>,
    pub tags: Vec<ONDCOnSearchItemTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchProviderLocation {
    pub id: String,
    pub gps: String,
    pub address: String,
    pub city: ONDCOnSearchCity,
    pub state: ONDCOnSearchState,
    pub country: ONDCOnSearchCountry,
    pub area_code: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCContact {
    pub email: Option<String>,
    pub phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchFulfillmentContact {
    contact: ONDCContact,
}

#[derive(Debug, Serialize, Deserialize)]
enum OfferType {
    #[serde(rename = "Disc_Pct")]
    DiscPct,
    #[serde(rename = "Disc_Amt")]
    DiscAmt,
    #[serde(rename = "BuyXGetY")]
    BuyXGetY,
    #[serde(rename = "Freebie")]
    Freebie,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOfferDescriptor {
    name: Option<String>,
    code: OfferType,
    short_desc: Option<String>,
    long_desc: Option<String>,
    images: Option<Vec<ONDCImage>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCRange {
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub start: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCTime {
    label: String,
    range: ONDCRange,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchOffer {
    id: String,
    descriptor: ONDCOfferDescriptor,
    location_ids: Vec<String>,
    category_ids: Vec<String>,
    item_ids: Vec<String>,
    time: ONDCTime,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchCategoryDescriptor {
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchCategory {
    id: String,
    descriptor: ONDCOnSearchCategoryDescriptor,
    tags: Vec<ONDCTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchProvider {
    pub id: String,
    pub descriptor: ONDCOnSearchProviderDescriptor,
    pub payments: Option<Vec<ONDCOnSearchPayment>>,
    pub rating: Option<String>,
    ttl: String,
    creds: Option<Vec<ONDCCredential>>,
    pub locations: Vec<ONDCOnSearchProviderLocation>,
    tags: Vec<ONDCTag>,
    fulfillments: Vec<ONDCOnSearchFulfillmentContact>,
    offers: Option<Vec<ONDCOnSearchOffer>>,
    categories: Option<Vec<ONDCOnSearchCategory>>,
    pub items: Vec<ONDCOnSearchItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchCatalog {
    pub descriptor: ONDCOnSearchDescriptor,
    pub payments: Vec<ONDCOnSearchPayment>,
    pub fulfillments: Vec<ONDCOnSearchFullFillment>,
    pub providers: Vec<ONDCOnSearchProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchMessage {
    pub catalog: Option<ONDCOnSearchCatalog>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ONDCOnSearchRequest {
    pub context: ONDCContext,
    pub message: ONDCOnSearchMessage,
    #[allow(dead_code)]
    pub error: Option<ONDCResponseErrorBody<ONDCSellerErrorCode>>,
}

impl FromRequest for ONDCOnSearchRequest {
    type Error = ONDCBuyerError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: e.to_string(),
                }),
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ONDCBuyerIdType {
    Gst,
    Pan,
    Tin,
    Aadhaar,
    Mobile,
    Email,
}

impl std::fmt::Display for ONDCBuyerIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

#[derive(Debug, Serialize, Deserialize)]

pub struct ONDCPerson {
    pub creds: Option<Vec<ONDCCredential>>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCustomer {
    pub person: ONDCPerson,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCLocationId {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSelectProvider {
    pub id: String,
    pub locations: Vec<ONDCLocationId>,
    pub ttl: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCQuantityCountInt {
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCQuantitySelect {
    pub selected: ONDCQuantityCountInt,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCItemAddOns {
    id: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSelectPayment {
    pub r#type: ONDCPaymentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCity {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCState {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCountry {
    pub code: CountryCode,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSelectFulfillmentLocation {
    pub gps: String,
    pub address: Option<String>,
    pub area_code: String,
    pub city: ONDCCity,
    pub country: ONDCCountry,
    pub state: ONDCState,
    // pub contact: ONDCContact,
}

#[derive(Debug, Serialize)]
pub struct ONDCInitFulfillmentLocation {
    gps: String,
    area_code: String,
    address: String,
    city: ONDCCity,
    country: ONDCCountry,
    state: ONDCState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOrderFulfillmentEnd {
    pub r#type: ONDCFulfillmentStopType,
    pub location: ONDCSelectFulfillmentLocation,
    pub contact: ONDCContact,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCFulfillment {
    pub id: String,
    pub r#type: ONDCFulfillmentType,
    pub stops: Option<Vec<ONDCOrderFulfillmentEnd>>,
    pub tags: Option<Vec<ONDCTag>>,
    pub customer: Option<ONDCCustomer>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSelectedItem {
    pub id: String,
    pub location_ids: Vec<String>,
    pub fulfillment_ids: Vec<String>,
    pub quantity: ONDCQuantitySelect,
    pub payment_ids: Option<Vec<String>>,
    pub tags: Option<Vec<ONDCTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct ONDCSelectOrder {
    pub provider: ONDCSelectProvider,
    pub items: Vec<ONDCSelectedItem>,
    pub add_ons: Option<Vec<ONDCItemAddOns>>,
    pub payments: Vec<ONDCSelectPayment>,
    pub fulfillments: Vec<ONDCFulfillment>,
    pub tags: Vec<ONDCTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSelectMessage {
    pub order: ONDCSelectOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCSelectRequest {
    pub context: ONDCContext,
    pub message: ONDCSelectMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectProvider {
    pub id: String,
    pub locations: Vec<ONDCLocationId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectMessage {
    pub order: ONDCOnSelectOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectPayment {
    pub r#type: ONDCPaymentType,
    pub collected_by: ONDCPaymentCollectedBy,
}

impl From<&ONDCOnSelectPayment> for Payment {
    fn from(ondc_payment_obj: &ONDCOnSelectPayment) -> Self {
        Payment {
            r#type: ondc_payment_obj.r#type.get_payment(),
            collected_by: Some(ondc_payment_obj.collected_by.get_type()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ONDCFulfillmentCategoryType {
    #[serde(rename = "Standard Delivery")]
    StandardDelivery,
    #[serde(rename = "Express Delivery")]
    ExpressDelivery,
    #[serde(rename = "Self-Pickup")]
    SelfPickup,
}

impl ONDCFulfillmentCategoryType {
    pub fn get_category_type(&self) -> FulfillmentCategoryType {
        match self {
            ONDCFulfillmentCategoryType::StandardDelivery => {
                FulfillmentCategoryType::StandardDelivery
            }
            ONDCFulfillmentCategoryType::ExpressDelivery => {
                FulfillmentCategoryType::ExpressDelivery
            }
            ONDCFulfillmentCategoryType::SelfPickup => FulfillmentCategoryType::SelfPickup,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCServiceableType {
    #[serde(rename = "Non-serviceable")]
    NonServiceable,
    #[serde(rename = "Serviceable")]
    Serviceable,
}

impl ONDCServiceableType {
    pub fn get_servicable_type(&self) -> ServiceableType {
        match self {
            ONDCServiceableType::NonServiceable => ServiceableType::NonServiceable,
            ONDCServiceableType::Serviceable => ServiceableType::Serviceable,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectFulfillmentDescriptor {
    pub code: ONDCServiceableType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectFulfillmentState {
    pub descriptor: ONDCOnSelectFulfillmentDescriptor,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectFulfillment {
    pub id: String,
    #[serde(rename = "@ondc/org/provider_name")]
    pub provider_name: Option<String>,
    #[serde(rename = "@ondc/org/category")]
    pub category: ONDCFulfillmentCategoryType,
    #[serde(rename = "@ondc/org/TAT")]
    pub tat: String,
    pub tracking: bool,
    pub state: ONDCOnSelectFulfillmentState,
    pub stops: Option<Vec<ONDCOrderFulfillmentEnd>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BreakupTitleType {
    Item,
    Delivery,
    Tax,
    Discount,
    Packing,
    Misc,
    Refund,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ONDCTitleName {
    #[serde(rename = "Convenience Fee")]
    ConvenienceFee,
    #[serde(rename = "Delivery Charge")]
    DeliveryCharge,
    #[serde(rename = "Packing")]
    Packing,
    #[serde(rename = "Discount")]
    Discount,
    #[serde(rename = "Tax")]
    Tax,
}

impl std::fmt::Display for ONDCTitleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ONDCTitleName::ConvenienceFee => "Convenience Fee",
            ONDCTitleName::DeliveryCharge => "Delivery Charge",
            ONDCTitleName::Packing => "Packing",
            ONDCTitleName::Discount => "Discount",
            ONDCTitleName::Tax => "Tax",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOrderItemQuantity {
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCBreakupItemInfo {
    pub price: ONDCAmount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCBreakUp {
    pub title: String,
    #[serde(rename = "@ondc/org/item_id")]
    pub item_id: String,
    #[serde(rename = "@ondc/org/title_type")]
    pub title_type: BreakupTitleType,
    pub price: ONDCAmount,
    #[serde(rename = "@ondc/org/item_quantity")]
    pub quantity: Option<ONDCOrderItemQuantity>,
    pub item: Option<ONDCBreakupItemInfo>,
}

impl ONDCBreakUp {
    pub fn create(
        title: String,
        item_id: String,
        title_type: BreakupTitleType,
        price: ONDCAmount,
        quantity: Option<ONDCOrderItemQuantity>,
        item: Option<ONDCBreakupItemInfo>,
    ) -> Self {
        Self {
            title,
            item_id,
            title_type,
            price,
            quantity,
            item,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCQuote {
    pub price: ONDCAmount,
    pub ttl: String,
    pub breakup: Vec<ONDCBreakUp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectOrder {
    pub provider: ONDCOnSelectProvider,
    pub payments: Vec<ONDCOnSelectPayment>,
    pub items: Vec<ONDCSelectedItem>,
    pub fulfillments: Vec<ONDCOnSelectFulfillment>,
    pub quote: ONDCQuote,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSelectRequest {
    pub context: ONDCContext,
    pub message: ONDCOnSelectMessage,
    pub error: Option<ONDCResponseErrorBody<ONDCSellerErrorCode>>,
}

impl FromRequest for ONDCOnSelectRequest {
    type Error = ONDCBuyerError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: e.to_string(),
                }),
            }
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSError {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub action_type: WebSocketActionType,
    pub error_message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSelect<'a> {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub action_type: WebSocketActionType,
    pub error: Option<&'a str>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ONDCOrderParams {
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub device_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub business_id: Option<Uuid>,
}

pub struct BulkSellerProductInfo<'a> {
    pub seller_subscriber_ids: Vec<&'a str>,
    pub provider_ids: Vec<&'a str>,
    pub item_codes: Vec<Option<&'a str>>,
    pub item_ids: Vec<&'a str>,
    pub item_names: Vec<&'a str>,
    pub tax_rates: Vec<BigDecimal>,
    pub mrps: Vec<BigDecimal>,
    pub unit_prices: Vec<BigDecimal>,
    pub image_objs: Vec<Value>,
    pub currency_codes: Vec<&'a CurrencyType>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ONDCSellerProductInfo {
    pub item_name: String,
    pub item_code: Option<String>,
    pub item_id: String,
    pub seller_subscriber_id: String,
    pub provider_id: String,
    #[schema(value_type = f64)]
    pub tax_rate: BigDecimal,
    #[schema(value_type = f64)]
    pub mrp: BigDecimal,
    #[schema(value_type = f64)]
    pub unit_price: BigDecimal,
    pub images: Value,
    pub currency_code: CurrencyType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCRequestModel {
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub device_id: Option<String>,
    pub request_payload: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRequestParamsModel {
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnInitProvider {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ONDCSettlementBasis {
    ReturnWindowExpiry,
    Shipment,
    Delivery,
}

impl ONDCSettlementBasis {
    pub fn get_settlement_basis_from_ondc_type(&self) -> SettlementBasis {
        match self {
            ONDCSettlementBasis::ReturnWindowExpiry => SettlementBasis::ReturnWindowExpiry,
            ONDCSettlementBasis::Shipment => SettlementBasis::Shipment,
            ONDCSettlementBasis::Delivery => SettlementBasis::Delivery,
        }
    }
}

// impl PgHasArrayType for &SettlementBasis {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_settlement_basis_type")
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCPaymentSettlementCounterparty {
    #[serde(rename = "buyer-app")]
    BuyerApp,
    #[serde(rename = "seller-app")]
    SellerApp,
}

impl ONDCPaymentSettlementCounterparty {
    pub fn get_settlement_counterparty(&self) -> PaymentSettlementCounterparty {
        match self {
            ONDCPaymentSettlementCounterparty::BuyerApp => PaymentSettlementCounterparty::BuyerApp,
            ONDCPaymentSettlementCounterparty::SellerApp => {
                PaymentSettlementCounterparty::SellerApp
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCPaymentSettlementPhase {
    #[serde(rename = "sale-amount")]
    SaleAmount,
}
impl ONDCPaymentSettlementPhase {
    pub fn get_settlement_phase(&self) -> PaymentSettlementPhase {
        match self {
            ONDCPaymentSettlementPhase::SaleAmount => PaymentSettlementPhase::SaleAmount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ONDCPaymentSettlementType {
    Neft,
}

impl ONDCPaymentSettlementType {
    pub fn get_settlement_type(&self) -> PaymentSettlementType {
        match self {
            ONDCPaymentSettlementType::Neft => PaymentSettlementType::Neft,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCPaymentSettlementDetail {
    pub settlement_counterparty: ONDCPaymentSettlementCounterparty,
    pub settlement_phase: ONDCPaymentSettlementPhase,
    pub settlement_type: ONDCPaymentSettlementType,
    pub settlement_bank_account_no: String,
    pub settlement_ifsc_code: String,
    pub beneficiary_name: String,
    pub bank_name: String,
}
impl ONDCPaymentSettlementDetail {
    pub fn to_payment_settlement_detail(&self) -> PaymentSettlementDetailModel {
        PaymentSettlementDetailModel {
            settlement_counterparty: self.settlement_counterparty.get_settlement_counterparty(),
            settlement_phase: self.settlement_phase.get_settlement_phase(),
            settlement_type: self.settlement_type.get_settlement_type(),
            settlement_bank_account_no: self.settlement_bank_account_no.clone(),
            settlement_ifsc_code: self.settlement_ifsc_code.clone(),
            beneficiary_name: self.beneficiary_name.clone(),
            bank_name: self.bank_name.clone(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnInitPayment {
    pub r#type: ONDCPaymentType,
    pub collected_by: ONDCPaymentCollectedBy,
    #[serde(rename = "@ondc/org/buyer_app_finder_fee_type")]
    pub buyer_app_finder_fee_type: FeeType,
    #[serde(rename = "@ondc/org/buyer_app_finder_fee_amount")]
    pub buyer_app_finder_fee_amount: String,
    #[serde(rename = "@ondc/org/settlement_basis")]
    pub settlement_basis: Option<ONDCSettlementBasis>,
    #[serde(rename = "@ondc/org/settlement_window")]
    pub settlement_window: Option<String>,
    #[serde(rename = "@ondc/org/withholding_amount")]
    pub withholding_amount: Option<String>,
    #[serde(rename = "@ondc/org/settlement_details")]
    pub settlement_details: Option<Vec<ONDCPaymentSettlementDetail>>,
    pub uri: Option<String>,
    pub tags: Option<Vec<ONDCTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnInitFulfillment {
    pub id: String,
    pub tracking: bool,
    pub r#type: ONDCFulfillmentType,
    pub stops: Vec<ONDCOrderFulfillmentEnd>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderLocation {
    id: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, ToSchema, PartialEq)]
pub enum ONDCPaymentCollectedBy {
    #[serde(rename = "BAP")]
    Bap,
    #[serde(rename = "BPP")]
    Bpp,
    #[serde(rename = "buyer")]
    Buyer,
}

impl ONDCPaymentCollectedBy {
    pub fn get_type(&self) -> PaymentCollectedBy {
        match self {
            ONDCPaymentCollectedBy::Bap => PaymentCollectedBy::Bap,
            ONDCPaymentCollectedBy::Bpp => PaymentCollectedBy::Bpp,
            ONDCPaymentCollectedBy::Buyer => PaymentCollectedBy::Buyer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCInitPayment {
    pub r#type: ONDCPaymentType,
    pub collected_by: ONDCPaymentCollectedBy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCBilling {
    pub name: String,
    pub address: String,
    pub state: ONDCState,
    pub city: ONDCCity,
    pub tax_id: String,
    pub email: Option<EmailObject>,
    pub phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCInitProvider {
    pub id: String,
    pub locations: Vec<ONDCLocationId>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ONDCOrderCancellationFee {
    Percent { percentage: String },
    Amount { amount: ONDCAmount },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOrderCancellationTerm {
    pub fulfillment_state: ONDCFulfillmentState,
    pub reason_required: bool,
    pub cancellation_fee: ONDCOrderCancellationFee,
}
impl ONDCOrderCancellationFee {
    pub fn get_type(&self) -> CancellationFeeType {
        match self {
            ONDCOrderCancellationFee::Percent { .. } => CancellationFeeType::Percent,
            ONDCOrderCancellationFee::Amount { .. } => CancellationFeeType::Amount,
        }
    }

    pub fn get_amount(&self) -> BigDecimal {
        match self {
            ONDCOrderCancellationFee::Percent { percentage } => {
                BigDecimal::from_str(percentage).unwrap()
            }
            ONDCOrderCancellationFee::Amount { amount } => {
                BigDecimal::from_str(&amount.value).unwrap()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCInitOrder {
    pub provider: ONDCInitProvider,
    pub items: Vec<ONDCSelectedItem>,
    pub add_ons: Option<Vec<ONDCItemAddOns>>,
    pub billing: ONDCBilling,
    pub payments: Vec<ONDCInitPayment>,
    pub fulfillments: Vec<ONDCFulfillment>,
    pub tags: Vec<ONDCTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCInitMessage {
    pub order: ONDCInitOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCInitRequest {
    pub context: ONDCContext,
    pub message: ONDCInitMessage,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSInitData<'a> {
    pub payment_links: Vec<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSInit<'a> {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub action_type: WebSocketActionType,
    pub error: Option<&'a str>,
    pub data: Option<WSInitData<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnInitOrder {
    pub provider: ONDCOnInitProvider,
    pub provider_location: ProviderLocation,
    pub items: Vec<ONDCSelectedItem>,
    pub payments: Vec<ONDCOnInitPayment>,
    pub tags: Vec<ONDCTag>,
    pub quote: ONDCQuote,
    pub billing: ONDCBilling,
    pub cancellation_terms: Vec<ONDCOrderCancellationTerm>,
    pub fulfillments: Vec<ONDCOnInitFulfillment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnInitMessage {
    pub order: ONDCOnInitOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnInitRequest {
    pub context: ONDCContext,
    pub message: ONDCOnInitMessage,
    pub error: Option<ONDCResponseErrorBody<ONDCSellerErrorCode>>,
}

impl FromRequest for ONDCOnInitRequest {
    type Error = ONDCBuyerError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: e.to_string(),
                }),
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ONDCOrderStatus {
    Created,
    Accepted,
    Completed,
    Cancelled,
    #[serde(rename = "In-progress")]
    InProgress,
}

impl ONDCOrderStatus {
    pub fn get_commerce_status(
        &self,
        record_type: &OrderType,
        proforma_present: Option<bool>,
    ) -> CommerceStatusType {
        match (self, record_type, proforma_present) {
            (ONDCOrderStatus::Accepted, OrderType::PurchaseOrder, Some(false)) => {
                CommerceStatusType::Created
            }
            (ONDCOrderStatus::Accepted, _, _) => CommerceStatusType::Accepted,
            (ONDCOrderStatus::Created, _, _) => CommerceStatusType::Created,
            (ONDCOrderStatus::Completed, _, _) => CommerceStatusType::Completed,
            (ONDCOrderStatus::Cancelled, _, _) => CommerceStatusType::Cancelled,
            (ONDCOrderStatus::InProgress, _, _) => CommerceStatusType::InProgress,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCConfirmOrder {
    pub id: String,
    pub state: ONDCOrderStatus,
    pub provider: ONDCConfirmProvider,
    pub payments: Vec<ONDCOnConfirmPayment>,
    pub quote: ONDCQuote,
    pub items: Vec<ONDCSelectedItem>,
    pub billing: ONDCBilling,
    pub tags: Vec<ONDCTag>,
    pub cancellation_terms: Vec<ONDCOrderCancellationTerm>,
    pub fulfillments: Vec<ONDCFulfillment>,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCConfirmMessage {
    pub order: ONDCConfirmOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDConfirmRequest {
    pub context: ONDCContext,
    pub message: ONDCConfirmMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ONDCPaymentStatus {
    #[serde(rename = "PAID")]
    Paid,
    #[serde(rename = "NOT-PAID")]
    NotPaid,
    #[serde(rename = "PENDING")]
    Pending,
}

impl ONDCPaymentStatus {
    pub fn get_payment_status(&self) -> PaymentStatus {
        match self {
            ONDCPaymentStatus::Paid => PaymentStatus::Paid,
            ONDCPaymentStatus::NotPaid => PaymentStatus::NotPaid,
            ONDCPaymentStatus::Pending => PaymentStatus::NotPaid,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCPaymentParams {
    pub amount: String,
    pub currency: CurrencyType,
    pub transaction_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnConfirmPayment {
    pub id: Option<String>,
    pub r#type: ONDCPaymentType,
    pub collected_by: ONDCPaymentCollectedBy,
    #[serde(rename = "@ondc/org/buyer_app_finder_fee_type")]
    pub buyer_app_finder_fee_type: FeeType,
    #[serde(rename = "@ondc/org/buyer_app_finder_fee_amount")]
    pub buyer_app_finder_fee_amount: String,
    #[serde(rename = "@ondc/org/settlement_basis")]
    pub settlement_basis: ONDCSettlementBasis,
    #[serde(rename = "@ondc/org/settlement_window")]
    pub settlement_window: String,
    #[serde(rename = "@ondc/org/withholding_amount")]
    pub withholding_amount: String,
    #[serde(rename = "@ondc/org/settlement_details")]
    pub settlement_details: Option<Vec<ONDCPaymentSettlementDetail>>,
    pub uri: Option<String>,
    pub tags: Option<Vec<ONDCTag>>,
    pub status: ONDCPaymentStatus,
    pub params: ONDCPaymentParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCConfirmProvider {
    pub id: String,
    pub locations: Vec<ONDCLocationId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCConfirmFulfillmentEndLocation {
    pub gps: String,
    pub address: Option<String>,
    pub area_code: String,
    pub city: ONDCCity,
    pub country: ONDCCountry,
    pub state: ONDCState,
    // pub contact: ONDCContact,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationDescriptor {
    name: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCConfirmFulfillmentStartLocation {
    pub id: String,
    pub gps: String,
    pub state: Option<String>,
    pub area_code: Option<String>,
    pub descriptor: LocationDescriptor,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ONDConfirmFulfillmentLocationType {
    End(ONDCConfirmFulfillmentEndLocation),
    Start(ONDCConfirmFulfillmentStartLocation),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCFulfillmentTime {
    pub range: ONDCRange,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCFulfillmentInstruction {
    pub name: String,
    pub short_desc: String,
    pub images: Option<Vec<String>>,
    pub long_desc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDConfirmFulfillmentEnd {
    pub r#type: ONDCFulfillmentStopType,
    pub location: ONDConfirmFulfillmentLocationType,
    pub contact: ONDCContact,
    pub time: ONDCFulfillmentTime,
    pub instructions: Option<ONDCFulfillmentInstruction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnConfirmFulfillment {
    pub id: String,
    pub r#type: ONDCFulfillmentType,
    pub state: ONDCFulfillmentState,
    #[serde(rename = "@ondc/org/provider_name")]
    pub provider_name: String,
    pub tracking: bool,
    pub rateable: bool,
    pub stops: Vec<ONDConfirmFulfillmentEnd>,
}

impl ONDCOnConfirmFulfillment {
    pub fn get_fulfillment_start(&self) -> Option<&ONDCConfirmFulfillmentStartLocation> {
        self.stops.iter().find_map(|stop| {
            if let ONDConfirmFulfillmentLocationType::Start(ref start_location) = stop.location {
                Some(start_location)
            } else {
                None
            }
        })
    }
    pub fn get_fulfillment_end(&self) -> Option<&ONDCConfirmFulfillmentEndLocation> {
        self.stops.iter().find_map(|stop| {
            if let ONDConfirmFulfillmentLocationType::End(ref start_location) = stop.location {
                Some(start_location)
            } else {
                None
            }
        })
    }
    pub fn get_fulfillment_contact(
        &self,
        fulfillment_type: ONDCFulfillmentStopType,
    ) -> Option<&ONDCContact> {
        self.stops.iter().find_map(|stop| {
            if stop.r#type == fulfillment_type {
                Some(&stop.contact)
            } else {
                None
            }
        })
    }
    pub fn get_fulfillment_time(
        &self,
        fulfillment_type: ONDCFulfillmentStopType,
    ) -> Option<&ONDCFulfillmentTime> {
        self.stops.iter().find_map(|stop| {
            if stop.r#type == fulfillment_type {
                Some(&stop.time)
            } else {
                None
            }
        })
    }

    pub fn get_fulfillment_instruction(
        &self,
        fulfillment_type: ONDCFulfillmentStopType,
    ) -> Option<&ONDCFulfillmentInstruction> {
        self.stops.iter().find_map(|stop| {
            if stop.r#type == fulfillment_type {
                stop.instructions.as_ref()
            } else {
                None
            }
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCConfirmedItem {
    pub id: String,
    pub location_ids: Option<Vec<String>>,
    pub fulfillment_ids: Vec<String>,
    pub quantity: ONDCQuantitySelect,

    pub tags: Option<Vec<ONDCTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnConfirmOrder {
    pub provider: ONDCConfirmProvider,
    pub state: ONDCOrderStatus,
    pub id: String,
    pub payments: Vec<ONDCOnConfirmPayment>,
    pub quote: ONDCQuote,
    pub items: Vec<ONDCSelectedItem>,
    pub billing: ONDCBilling,
    pub tags: Vec<ONDCTag>,
    pub cancellation_terms: Vec<ONDCOrderCancellationTerm>,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub updated_at: DateTime<Utc>,
    pub fulfillments: Vec<ONDCOnConfirmFulfillment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnConfirmMessage {
    pub order: ONDCOnConfirmOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnConfirmRequest {
    pub context: ONDCContext,
    pub message: ONDCOnConfirmMessage,
    pub error: Option<ONDCResponseErrorBody<ONDCSellerErrorCode>>,
}

impl FromRequest for ONDCOnConfirmRequest {
    type Error = ONDCBuyerError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: e.to_string(),
                }),
            }
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSConfirmData<'a> {
    pub payment_links: Vec<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSConfirm<'a> {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub error: Option<&'a str>,
    pub data: Option<WSConfirmData<'a>>,
}

pub struct BulkSellerLocationInfo<'a> {
    pub seller_subscriber_ids: Vec<&'a str>,
    pub provider_ids: Vec<&'a str>,
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
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ONDCSellerLocationInfo {
    pub location_id: String,
    pub seller_subscriber_id: String,
    pub provider_id: String,
    #[schema(value_type = f64)]
    pub latitude: BigDecimal,
    #[schema(value_type = f64)]
    pub longitude: BigDecimal,
    pub address: String,
    pub city_code: String,
    pub city_name: String,
    pub state_code: String,
    pub state_name: Option<String>,
    pub country_code: CountryCode,
    pub country_name: Option<String>,
    pub area_code: String,
}

pub struct BulkSellerInfo<'a> {
    pub seller_subscriber_ids: Vec<&'a str>,
    pub provider_ids: Vec<&'a str>,
    pub provider_names: Vec<&'a str>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct ONDCSellerInfo {
    pub seller_subscriber_id: String,
    pub provider_id: String,
    pub provider_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCStatusMessage {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCStatusRequest {
    pub context: ONDCContext,
    pub message: ONDCStatusMessage,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSStatus {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]

pub enum ONDCDocumentType {
    #[serde(rename = "PROFORMA_INVOICE")]
    ProformaInvoice,
    #[serde(rename = "Invoice")]
    Invoice,
}
impl ONDCDocumentType {
    pub fn get_document_type(&self) -> DocumentType {
        match self {
            ONDCDocumentType::ProformaInvoice => DocumentType::ProformaInvoice,
            ONDCDocumentType::Invoice => DocumentType::Invoice,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCDocument {
    pub label: ONDCDocumentType,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnStatusOrder {
    pub provider: ONDCConfirmProvider,
    pub state: ONDCOrderStatus,
    pub id: String,
    pub payments: Vec<ONDCOnConfirmPayment>,
    pub quote: ONDCQuote,
    pub items: Vec<ONDCSelectedItem>,
    pub billing: ONDCBilling,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub updated_at: DateTime<Utc>,
    pub fulfillments: Vec<ONDCOnConfirmFulfillment>,
    pub documents: Option<Vec<ONDCDocument>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnStatusMessage {
    pub order: ONDCOnStatusOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnStatusRequest {
    pub context: ONDCContext,
    pub message: ONDCOnStatusMessage,
    pub error: Option<ONDCResponseErrorBody<ONDCSellerErrorCode>>,
}

impl FromRequest for ONDCOnStatusRequest {
    type Error = ONDCBuyerError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: e.to_string(),
                }),
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCancelMessage {
    pub order_id: String,
    pub cancellation_reason_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCancelRequest {
    pub context: ONDCContext,
    pub message: ONDCCancelMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCReason {
    pub id: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCCancellation {
    pub cancelled_by: String,
    pub reason: ONDCReason,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnCancelOrder {
    pub provider: ONDCConfirmProvider,
    pub state: ONDCOrderStatus,
    pub id: String,
    pub payments: Vec<ONDCOnConfirmPayment>,
    pub quote: ONDCQuote,
    pub items: Vec<ONDCSelectedItem>,
    pub billing: ONDCBilling,
    pub cancellation: ONDCCancellation,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub updated_at: DateTime<Utc>,
    pub fulfillments: Vec<ONDCOnConfirmFulfillment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnCancelMessage {
    pub order: ONDCOnCancelOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnCancelRequest {
    pub context: ONDCContext,
    pub message: ONDCOnCancelMessage,
    pub error: Option<ONDCResponseErrorBody<ONDCSellerErrorCode>>,
}

impl FromRequest for ONDCOnCancelRequest {
    type Error = ONDCBuyerError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(ONDCBuyerError::InvalidResponseError {
                    path: None,
                    message: e.to_string(),
                }),
            }
        })
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSCancel {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub error: Option<String>,
}
