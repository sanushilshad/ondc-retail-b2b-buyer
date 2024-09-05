use super::errors::ONDCBuyerError;
use crate::domain::EmailObject;
use crate::routes::ondc::schemas::{ONDCContext, ONDCResponseErrorBody};
use crate::routes::ondc::{ONDCItemUOM, ONDCSellerErrorCode};
use crate::routes::order::schemas::{
    FulfillmentCategoryType, IncoTermType, Payment, ServiceableType,
};
use crate::routes::product::schemas::{FulfillmentType, PaymentType};
use crate::schemas::{CountryCode, CurrencyType, FeeType, ONDCNetworkType, WSKeyTrait};
use crate::utils::pascal_to_snake_case;
use crate::websocket::WebSocketActionType;
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use bigdecimal::BigDecimal;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use utoipa::ToSchema;
use uuid::Uuid;
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
}

impl std::fmt::Display for ONDCTagItemCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

#[derive(Debug, Serialize, Deserialize)]

enum ONDCFulfillmentState {
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
    pub code: String,
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
    pub name: Option<String>,
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
struct ONDCFulfillmentDescriptor {
    code: ONDCFulfillmentState,
}

#[derive(Debug, Serialize, Deserialize)]
struct FulfillmentState {
    descriptor: ONDCFulfillmentDescriptor,
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
    fulfillment_state: FulfillmentState,
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
    fulfillment_state: FulfillmentState,
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
struct ONDCRange {
    start: String,
    end: String,
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
    creds: Vec<ONDCCredential>,
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCCustomer {
    person: ONDCPerson,
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
pub struct ONDCOrderFulfillmentEnd<T> {
    pub r#type: ONDCFulfillmentStopType,
    pub location: T,
    pub contact: ONDCContact,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCFulfillment<T> {
    pub id: String,
    pub r#type: ONDCFulfillmentType,
    pub stops: Option<Vec<ONDCOrderFulfillmentEnd<T>>>,
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

    pub tags: Option<Vec<ONDCTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct ONDCSelectOrder {
    pub provider: ONDCSelectProvider,
    pub items: Vec<ONDCSelectedItem>,
    pub add_ons: Option<Vec<ONDCItemAddOns>>,
    pub payments: Vec<ONDCSelectPayment>,
    pub fulfillments: Vec<ONDCFulfillment<ONDCSelectFulfillmentLocation>>,
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
    pub collected_by: ONDCNetworkType,
}

impl From<&ONDCOnSelectPayment> for Payment {
    fn from(ondc_payment_obj: &ONDCOnSelectPayment) -> Self {
        Payment {
            r#type: ondc_payment_obj.r#type.get_payment(),
            collected_by: Some(ondc_payment_obj.collected_by.clone()),
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
    pub stops: Option<Vec<ONDCOrderFulfillmentEnd<ONDCSelectFulfillmentLocation>>>,
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
    title: String,
    #[serde(rename = "@ondc/org/item_id")]
    pub item_id: Option<String>,
    #[serde(rename = "@ondc/org/title_type")]
    pub title_type: BreakupTitleType,
    pub price: ONDCAmount,
    #[serde(rename = "@ondc/org/item_quantity")]
    pub quantity: Option<ONDCOrderItemQuantity>,
    pub item: Option<ONDCBreakupItemInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCQuote {
    pub price: ONDCAmount,
    ttl: String,
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
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub action_type: WebSocketActionType,
    pub error_message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WSSelect<'a> {
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub action_type: WebSocketActionType,
    pub error: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ONDCOrderParams {
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub device_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub business_id: Option<Uuid>,
}

impl WSKeyTrait for ONDCOrderParams {
    fn get_key(&self) -> String {
        format!(
            "{}#{}#{}",
            self.user_id.map_or("NA".to_string(), |id| id.to_string()),
            self.business_id
                .map_or("NA".to_string(), |id| id.to_string()),
            self.device_id.clone().unwrap_or("NA".to_string())
        )
    }
}

pub struct BulkSellerProductInfo<'a> {
    pub seller_subscriber_ids: Vec<&'a str>,
    pub provider_ids: Vec<&'a str>,
    pub provider_names: Vec<&'a str>,
    pub item_codes: Vec<Option<&'a str>>,
    pub item_ids: Vec<&'a str>,
    pub item_names: Vec<&'a str>,
    pub tax_rates: Vec<BigDecimal>,
    pub mrps: Vec<BigDecimal>,
    pub unit_prices: Vec<BigDecimal>,
    pub image_objs: Vec<Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SellerProductInfo {
    pub item_name: String,
    pub item_code: Option<String>,
    pub item_id: String,
    pub seller_subscriber_id: String,
    pub provider_id: String,
    pub provider_name: Option<String>,
    pub tax_rate: BigDecimal,
    pub mrp: BigDecimal,
    pub unit_price: BigDecimal,
    pub images: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCRequestModel {
    pub transaction_id: Uuid,
    pub message_id: Uuid,
    pub user_id: Option<Uuid>,
    pub business_id: Option<Uuid>,
    pub device_id: Option<String>,
    pub request_payload: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnInitRequest {
    pub context: ONDCContext,
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
pub struct ONDCInitPayment {
    pub r#type: ONDCPaymentType,
    pub collected_by: ONDCNetworkType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCBilling {
    pub name: String,
    pub address: String,
    pub state: ONDCState,
    pub city: ONDCCity,
    pub tax_id: String,
    pub email: EmailObject,
    pub phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCInitProvider {
    pub id: String,
    pub locations: Vec<ONDCLocationId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCInitOrder {
    pub provider: ONDCInitProvider,
    pub items: Vec<ONDCSelectedItem>,
    pub add_ons: Option<Vec<ONDCItemAddOns>>,
    pub billing: ONDCBilling,
    pub payments: Vec<ONDCInitPayment>,
    pub fulfillments: Vec<ONDCFulfillment<ONDCSelectFulfillmentLocation>>,
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
