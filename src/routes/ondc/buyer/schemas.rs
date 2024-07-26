use super::errors::ONDCBuyerError;
use crate::routes::ondc::schemas::{ONDCContext, ONDCResponseErrorBody};
use crate::routes::ondc::{ONDCItemUOM, ONDCSellerErrorCode};
use crate::routes::order::schemas::IncoTermType;
use crate::routes::product::schemas::{CategoryDomain, FulfillmentType, PaymentType};
use crate::schemas::{CountryCode, CurrencyType, FeeType, ONDCNetworkType};
use crate::utils::pascal_to_snake_case;
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use bigdecimal::BigDecimal;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCTagDescriptor {
    pub code: ONDCTagType,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn get_tag_item(descriptor_code: ONDCTagItemCode, tag_item_code: &str) -> ONDCTagItem {
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
                ONDCTagItem::get_tag_item(
                    ONDCTagItemCode::FinderFeeType,
                    &finder_fee_type.to_string(),
                ),
                ONDCTagItem::get_tag_item(ONDCTagItemCode::FinderFeeAmount, finder_fee_amount),
            ],
        }
    }

    pub fn get_buyer_id_tag(id_code: &ONDCBuyerIdType, id_no: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BuyerId,
            },
            list: vec![
                ONDCTagItem::get_tag_item(ONDCTagItemCode::BuyerIdCode, &id_code.to_string()),
                ONDCTagItem::get_tag_item(ONDCTagItemCode::BuyerIdNo, id_no),
            ],
        }
    }

    pub fn get_delivery_terms(inco_term: &IncoTermType, place_of_delivery: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::DeliveyTerms,
            },
            list: vec![
                ONDCTagItem::get_tag_item(ONDCTagItemCode::IncoTerms, &inco_term.to_string()),
                ONDCTagItem::get_tag_item(ONDCTagItemCode::BuyerIdNo, place_of_delivery),
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
struct ONDCAmount {
    currency: CurrencyType,
    value: String,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchItemPrice {
    pub currency: CurrencyType,
    #[schema(value_type = f64)]
    pub value: BigDecimal,
    #[schema(value_type =Option<f64>)]
    pub offered_value: Option<BigDecimal>,
    #[schema(value_type = f64)]
    pub maximum_value: BigDecimal,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSCreatorContactData<'a> {
    pub name: &'a str,
    pub address: &'a str,
    pub phone: &'a str,
    pub email: &'a str,
}

#[derive(Debug, Serialize, ToSchema)]
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
pub struct WSSearchItemQty {
    pub measure: WSSearchItemQtyMeasure,
    pub count: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchItemQtyMeasure {
    pub unit: ONDCItemUOM,
    #[schema(value_type = f64)]
    pub value: BigDecimal,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnitizedProductQty {
    pub unit: ONDCItemUOM,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchItemQuantity {
    pub unitized: UnitizedProductQty,
    pub available: WSSearchItemQty,
    pub maximum: WSSearchItemQty,
    pub minimum: Option<WSSearchItemQty>,
}
#[derive(Debug, Serialize, ToSchema)]
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
pub struct WSSearchProductNpDeatils {
    pub name: String,
    pub code: Option<String>,
    pub short_desc: String,
    pub long_desc: String,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSProductCategory {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSItemPayment<'a> {
    pub r#type: PaymentType,
    pub collected_by: &'a ONDCNetworkType,
}

#[derive(Debug, Serialize, ToSchema)]
#[skip_serializing_none]
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
    pub tax_rate: BigDecimal,
    // pub country_of_origin: CountryCode,
    pub images: Vec<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchCountry<'a> {
    pub code: &'a str,
    pub name: Option<&'a str>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchState<'a> {
    pub code: &'a str,
    pub name: Option<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchCity<'a> {
    pub code: &'a str,
    pub name: Option<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
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
pub struct WSSearchProvider<'a> {
    pub items: Vec<WSSearchItem<'a>>,
    pub provider_detail: WSSearchProductProvider<'a>,
    pub locations: HashMap<String, WSSearchProviderLocation<'a>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchBPP<'a> {
    pub name: &'a str,
    pub code: Option<&'a str>,
    pub short_desc: &'a str,
    pub long_desc: &'a str,
    pub images: Vec<&'a str>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearchData<'a> {
    pub bpp: WSSearchBPP<'a>,
    pub providers: Vec<WSSearchProvider<'a>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSSearch<'a> {
    #[schema(value_type = String)]
    pub transaction_id: Uuid,
    #[schema(value_type = String)]
    pub message_id: Uuid,
    pub message: WSSearchData<'a>,
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

#[derive(Debug, Serialize)]
pub struct ONDCLocationIds {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct ONDCSelectProvider {
    pub id: String,
    pub locations: Vec<ONDCLocationIds>,
    pub ttl: String,
}

#[derive(Debug, Serialize)]
pub struct ONDCItemSelectQty {
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct ONDCItemSelect {
    pub selected: ONDCItemSelectQty,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ONDCItemAddOns {
    id: String,
}
#[derive(Debug, Serialize)]
pub struct ONDCSelectPaymentType {
    pub r#type: ONDCPaymentType,
}

#[derive(Debug, Serialize)]
pub struct ONDCCity {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ONDCState {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ONDCCountry {
    pub code: CountryCode,
}
#[derive(Debug, Serialize)]
pub struct ONDCSelectFulfillmentLocation {
    pub gps: String,
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

#[derive(Debug, Serialize)]
pub struct ONDCOrderFulfillmentEnd<T> {
    pub r#type: ONDCFulfillmentStopType,
    pub location: T,
    pub contact: ONDCContact,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ONDCFulfillment<T> {
    pub id: String,
    pub r#type: ONDCFulfillmentType,
    pub stops: Option<Vec<ONDCOrderFulfillmentEnd<T>>>,
    pub tags: Option<Vec<ONDCTag>>,
    pub customer: Option<ONDCCustomer>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ONDCSelectItem {
    pub id: String,
    pub location_ids: Vec<String>,
    pub fulfillment_ids: Vec<String>,
    pub quantity: ONDCItemSelect,

    pub tags: Option<Vec<ONDCTag>>,
}

#[derive(Debug, Serialize)]
#[skip_serializing_none]
pub struct ONDCSelectOrder {
    pub provider: ONDCSelectProvider,
    pub items: Vec<ONDCSelectItem>,
    pub add_ons: Option<Vec<ONDCItemAddOns>>,
    pub payments: Vec<ONDCSelectPaymentType>,
    pub fulfillments: Vec<ONDCFulfillment<ONDCSelectFulfillmentLocation>>,
    pub tags: Vec<ONDCTag>,
}

#[derive(Debug, Serialize)]
pub struct ONDCSelectMessage {
    pub order: ONDCSelectOrder,
}

#[derive(Debug, Serialize)]
pub struct ONDCSelectRequest {
    pub context: ONDCContext,
    pub message: ONDCSelectMessage,
}
