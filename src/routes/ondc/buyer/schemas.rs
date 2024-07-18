use super::errors::ONDCBuyerError;
use crate::routes::ondc::schemas::{ONDCContext, ONDCResponseErrorBody};
use crate::routes::ondc::{ONDCItemUOM, ONDCSellerErrorCode};
use crate::routes::product::schemas::{FulfillmentType, PaymentType};
use crate::routes::schemas::VectorType;
use crate::schemas::{CurrencyType, FeeType, ONDCNetworkType};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[derive(Debug, Serialize, Deserialize)]
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCTagDescriptor {
    code: ONDCTagType,
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
    pub fn get_buyer_fee_type(fee_type: ONDCFeeType) -> ONDCTagItem {
        ONDCTagItem {
            descriptor: ONDCTagItemDescriptor {
                code: ONDCTagItemCode::FinderFeeType,
            },
            value: fee_type.to_string(),
        }
    }
    pub fn get_buyer_fee_amount(fee_amount: &str) -> ONDCTagItem {
        ONDCTagItem {
            descriptor: ONDCTagItemDescriptor {
                code: ONDCTagItemCode::FinderFeeType,
            },
            value: fee_amount.to_owned(),
        }
    }

    pub fn get_buyer_id_code(id_code: &VectorType) -> ONDCTagItem {
        ONDCTagItem {
            descriptor: ONDCTagItemDescriptor {
                code: ONDCTagItemCode::BuyerIdCode,
            },
            value: id_code.to_string(),
        }
    }
    pub fn get_buyer_id_no(id_no: &str) -> ONDCTagItem {
        ONDCTagItem {
            descriptor: ONDCTagItemDescriptor {
                code: ONDCTagItemCode::BuyerIdNo,
            },
            value: id_no.to_string(),
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCTag {
    pub descriptor: ONDCTagDescriptor,
    pub list: Vec<ONDCTagItem>,
}

impl ONDCTag {
    pub fn get_buyer_fee_tag(finder_fee_type: ONDCFeeType, finder_fee_amount: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BapTerms,
            },
            list: vec![
                ONDCTagItem::get_buyer_fee_type(finder_fee_type),
                ONDCTagItem::get_buyer_fee_amount(finder_fee_amount),
            ],
        }
    }

    pub fn get_buyer_id_tag(id_code: &VectorType, id_no: &str) -> ONDCTag {
        ONDCTag {
            descriptor: ONDCTagDescriptor {
                code: ONDCTagType::BuyerId,
            },
            list: vec![
                ONDCTagItem::get_buyer_id_code(id_code),
                ONDCTagItem::get_buyer_id_no(id_no),
            ],
        }
    }
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
    pub fn get_ondc_fulfillment(payment_type: &FulfillmentType) -> ONDCFulfillmentType {
        match payment_type {
            FulfillmentType::Delivery => ONDCFulfillmentType::Delivery,
            FulfillmentType::SelfPickup => ONDCFulfillmentType::SelfPickup,
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
    pub fn get_ondc_payment(payment_type: &PaymentType) -> ONDCPaymentType {
        match payment_type {
            PaymentType::CashOnDelivery => ONDCPaymentType::OnFulfillment,
            PaymentType::PrePaid => ONDCPaymentType::PreFulfillment,
            PaymentType::Credit => ONDCPaymentType::PostFulfillment,
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

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchDescriptor {
    name: String,
    code: Option<String>,
    short_desc: String,
    long_desc: String,
    images: Vec<ONDCImage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchPayment {
    pub id: String,
    pub r#type: ONDCPaymentType,
    pub collected_by: Option<ONDCNetworkType>,
}

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
struct ONDCOnSearchProviderDescriptor {
    name: String,
    code: String,
    short_desc: String,
    long_desc: String,
    additional_desc: Option<ONDCOnSearchAdditionalDescriptor>,
    images: Vec<ONDCImage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCOnSearchCountry {
    code: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct ONDCOnSearchState {
    code: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCOnSearchCity {
    code: String,
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
    images: Vec<ONDCImage>,
    media: Option<Vec<ONDCMedia>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchCreatorAddress {
    full: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCItemCreatorContact {
    name: String,
    address: ONDCOnSearchCreatorAddress,
    phone: String,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCItemCreatorDescriptor {
    name: String,
    contact: ONDCItemCreatorContact,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCOnSearchItemCreator {
    descriptor: ONDCItemCreatorDescriptor,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchItemPrice {
    pub currency: CurrencyType,
    pub value: String,
    pub offered_value: Option<String>,
    pub maximum_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchQtyMeasure {
    unit: ONDCItemUOM,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchQtUnitized {
    measure: ONDCOnSearchQtyMeasure,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchQty {
    measure: ONDCOnSearchQtyMeasure,
    count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCOnSearchItemQuantity {
    unitized: ONDCOnSearchQtUnitized,
    available: ONDCOnSearchQty,
    maximum: ONDCOnSearchQty,
    minimum: Option<ONDCOnSearchQty>,
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
pub struct ONDCOnSearchItemTag {
    pub descriptor: ONDCTagDescriptor,
    pub list: Vec<ONDCOnSearchItemTagItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ONDCOnSearchItem {
    pub id: String,
    pub parent_item_id: Option<String>,
    matched: bool,
    pub recommended: bool,
    pub descriptor: ONDCOnSearchItemDescriptor,
    creator: ONDCOnSearchItemCreator,
    category_ids: Vec<String>,
    fulfillment_ids: Vec<String>,
    location_ids: Vec<String>,
    payment_ids: Vec<String>,
    pub price: ONDCOnSearchItemPrice,
    quantity: ONDCOnSearchItemQuantity,
    add_ons: Option<Vec<ONDCOnSearchItemAddOns>>,
    time: Option<ONDCTime>,
    replacement_terms: Vec<ONDCItemReplacementTerm>,
    return_terms: Vec<ONDCReturnTerm>,
    cancellation_terms: Vec<ONDCItemCancellationTerm>,
    tags: Vec<ONDCOnSearchItemTag>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ONDCOnSearchProviderLocation {
    id: String,
    gps: String,
    address: String,
    city: ONDCOnSearchCity,
    state: ONDCOnSearchState,
    country: ONDCOnSearchCountry,
    area_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ONDCContact {
    email: Option<String>,
    phone: String,
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
    id: String,
    descriptor: ONDCOnSearchProviderDescriptor,
    pub payments: Option<Vec<ONDCOnSearchPayment>>,
    rating: String,
    ttl: String,
    creds: Option<Vec<ONDCCredential>>,
    locations: Vec<ONDCOnSearchProviderLocation>,
    tags: Vec<ONDCTag>,
    fulfillments: Vec<ONDCOnSearchFulfillmentContact>,
    offers: Option<Vec<ONDCOnSearchOffer>>,
    categories: Option<Vec<ONDCOnSearchCategory>>,
    pub items: Vec<ONDCOnSearchItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchCatalog {
    descriptor: ONDCOnSearchDescriptor,
    pub payments: Vec<ONDCOnSearchPayment>,
    fulfillments: Vec<ONDCOnSearchFullFillment>,
    pub providers: Vec<ONDCOnSearchProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ONDCOnSearchMessage {
    pub catalog: Option<ONDCOnSearchCatalog>,
}

#[derive(Debug, Serialize, Deserialize)]
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
