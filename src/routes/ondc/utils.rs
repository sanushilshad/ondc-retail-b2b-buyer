use super::{
    BreakupTitleType, LookupData, LookupRequest, ONDCActionType, ONDCBreakUp, ONDCCancelMessage,
    ONDCCancelRequest, ONDCConfirmMessage, ONDCConfirmOrder, ONDCConfirmProvider, ONDCContext,
    ONDCContextCity, ONDCContextCountry, ONDCContextLocation, ONDCCredential, ONDCCredentialType,
    ONDCDomain, ONDCFeeType, ONDCItemCancellationFee, ONDCOnSearchCategory,
    ONDCOnSearchFulfillmentContact, ONDCOnSearchItem, ONDCSearchStop, ONDCSellePriceSlab,
    ONDCServicabilityCoordinate, ONDCStatusMessage, ONDCStatusRequest, ONDCTag, ONDCUpdateItem,
    ONDCUpdateMessage, ONDCUpdateOrder, ONDCUpdateProvider, ONDCUpdateRequest, ONDCVersion,
    OndcUrl,
};

use crate::chat_client::ChatData;
use crate::elastic_search_client::ElasticSearchClient;
use crate::routes::product::utils::{save_cache_to_db, save_cache_to_elastic_search};
use crate::user_client::{get_vector_val_from_list, BusinessAccount, UserAccount, VectorType};
use crate::websocket_client::{NotificationProcessType, WebSocketActionType, WebSocketClient};
use crate::{constants::ONDC_TTL, routes::product::ProductSearchError};
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use geojson::{GeoJson, Geometry};
use reqwest::Client;
use serde::Serializer;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::vec;

use bigdecimal::{BigDecimal, ToPrimitive};

use super::schemas::{
    BulkSellerInfo, BulkSellerLocationInfo, BulkSellerProductInfo, ONDCAmount, ONDCBilling,
    ONDCBreakupItemInfo, ONDCContact, ONDCCountry, ONDCCustomer, ONDCFulfillment,
    ONDCFulfillmentDescriptor, ONDCFulfillmentState, ONDCFulfillmentStopType, ONDCFulfillmentType,
    ONDCImage, ONDCInitMessage, ONDCInitOrder, ONDCInitPayment, ONDCInitProvider, ONDCInitRequest,
    ONDCLocationId, ONDCOnConfirmPayment, ONDCOnSearchItemPrice, ONDCOnSearchItemQuantity,
    ONDCOnSearchItemTag, ONDCOnSearchPayment, ONDCOnSearchProviderDescriptor,
    ONDCOnSearchProviderLocation, ONDCOnSearchRequest, ONDCOrderCancellationFee,
    ONDCOrderCancellationTerm, ONDCOrderFulfillmentEnd, ONDCOrderItemQuantity, ONDCOrderStatus,
    ONDCPaymentParams, ONDCPaymentSettlementCounterparty, ONDCPaymentSettlementDetail,
    ONDCPaymentStatus, ONDCQuantityCountInt, ONDCQuantitySelect, ONDCQuote, ONDCRequestModel,
    ONDCSearchCategory, ONDCSearchDescriptor, ONDCSearchFulfillment, ONDCSearchIntent,
    ONDCSearchItem, ONDCSearchLocation, ONDCSearchMessage, ONDCSearchPayment, ONDCSearchRequest,
    ONDCSelectFulfillmentLocation, ONDCSelectMessage, ONDCSelectOrder, ONDCSelectPayment,
    ONDCSelectProvider, ONDCSelectRequest, ONDCSelectedItem, ONDCSellerLocationInfo,
    ONDCSellerProductInfo, ONDCState, ONDCTagItemCode, ONDCTagType, ONDConfirmRequest,
    OnSearchContentType, TagTrait,
};
use crate::domain::EmailObject;
use crate::routes::ondc::schemas::{ONDCCity, ONDCPerson, ONDCSellerInfo};
use crate::routes::ondc::{ONDCErrorCode, ONDCResponse};
use crate::routes::order::errors::{
    ConfirmOrderError, InitOrderError, OrderCancelError, OrderStatusError, OrderUpdateError,
    SelectOrderError,
};
use crate::routes::order::schemas::{
    BuyerTerms, CancellationFeeType, Commerce, CommerceBilling, CommerceCancellationFee,
    CommerceCancellationTerm, CommerceFulfillment, CommerceItem, CommercePayment, DropOffData,
    OrderCancelRequest, OrderConfirmRequest, OrderDeliveyTerm, OrderInitBilling, OrderInitRequest,
    OrderSelectFulfillment, OrderSelectItem, OrderSelectRequest, OrderStatusRequest, OrderType,
    OrderUpdateRequest, PaymentCollectedBy, PickUpData, SelectFulfillmentLocation, SettlementBasis,
    TradeType, UpdateOrderPaymentRequest,
};
use crate::routes::product::schemas::{
    CategoryDomain, CredentialType, FulfillmentType, PaymentType, ProductFulFillmentLocation,
    ProductSearchRequest, ProductSearchType, SearchRequestModel, WSCreatorContactData,
    WSItemCancellation, WSItemCancellationFee, WSItemCancellationTerm, WSItemReplacementTerm,
    WSItemReturnLocation, WSItemReturnTerm, WSItemReturnTime, WSItemValidity, WSPaymentTypes,
    WSPriceSlab, WSProductCategory, WSProductCreator, WSSearch, WSSearchBPP, WSSearchCity,
    WSSearchCountry, WSSearchData, WSSearchItem, WSSearchItemAttribute, WSSearchItemPrice,
    WSSearchItemQty, WSSearchItemQtyMeasure, WSSearchItemQuantity,
    WSSearchProductProviderDescription, WSSearchProvider, WSSearchProviderContact,
    WSSearchProviderCredential, WSSearchProviderID, WSSearchProviderLocation,
    WSSearchProviderTerms, WSSearchServicability, WSSearchState, WSSearchVariant,
    WSSearchVariantAttribute, WSServicabilityData,
};
use serde_json::Value;
use sqlx::types::Json;

use crate::schemas::{
    CountryCode, CurrencyType, FeeType, NetworkCall, ONDCNetworkType, RegisteredNetworkParticipant,
    WebSocketParam,
};
use crate::utils::get_gps_string;

#[tracing::instrument(name = "Call lookup API", skip())]
pub async fn call_lookup_api(
    payload: &str,
    lookup_url: &str,
) -> Result<Option<LookupData>, anyhow::Error> {
    let client = Client::new();
    let network_call = NetworkCall { client };
    let result = network_call
        .async_post_call_with_retry(lookup_url, Some(payload), None)
        .await?;
    match result {
        serde_json::Value::Array(data) => {
            if data.is_empty() {
                return Ok(None);
            }
            let lookup_data_value = data.first().expect("Expected non-empty array");
            let lookup_data: LookupData = serde_json::from_value(lookup_data_value.clone())?;
            Ok(Some(lookup_data))
        }
        _ => {
            return Err(anyhow::format_err!("Error while parsing looup"));
        }
    }
}
#[tracing::instrument(name = "Get lookup for subscriber", skip())]
pub async fn get_lookup_for_subscriber_by_api(
    subscriber_id: &str,
    np_type: &ONDCNetworkType,
    domain: &ONDCDomain,
    lookup_uri: &str,
) -> Result<Option<LookupData>, anyhow::Error> {
    let look_up_request = LookupRequest {
        subscriber_id,
        domain,
        r#type: np_type,
    };
    let request_str = serde_json::to_string(&look_up_request).unwrap();
    let url = format!("{}{}", lookup_uri, OndcUrl::LookUp);
    let look_up_data = call_lookup_api(&request_str, &url).await?;
    Ok(look_up_data)
}

#[tracing::instrument(name = "Get lookup data from db", skip(pool))]
pub async fn get_lookup_data_from_db(
    pool: &PgPool,
    subscriber_id: &str,
    np_type: &ONDCNetworkType,
    domain: &ONDCDomain,
) -> Result<Option<LookupData>, anyhow::Error> {
    let row = sqlx::query_as!(
        LookupData,
        r#"SELECT br_id, subscriber_id, signing_public_key, subscriber_url, encr_public_key, uk_id, domain as "domain: ONDCDomain", type as "type: ONDCNetworkType"  FROM network_participant
        WHERE subscriber_id = $1 AND type = $2 AND domain = $3
        "#,
        subscriber_id,
        np_type as &ONDCNetworkType,
        domain.to_string()
    )
    .fetch_optional(pool)
    .await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("failed to fetch network lookup data from database")
    })?;
    Ok(row)
}

#[tracing::instrument(name = "Save lookup data to db", skip(pool))]
pub async fn save_lookup_data_to_db(pool: &PgPool, data: &LookupData) -> Result<(), anyhow::Error> {
    // let uuid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO network_participant (subscriber_id, br_id, subscriber_url, signing_public_key, domain, encr_public_key, type, uk_id, created_on)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (subscriber_id, type) DO NOTHING;
        "#,
        &data.subscriber_id,
        &data.br_id,
        &data.subscriber_url,
        &data.signing_public_key,
        &data.domain.to_string(),
        &data.encr_public_key,
        &data.r#type as &ONDCNetworkType,
        &data.uk_id,
        Utc::now(),
    )
    .execute(pool).await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving look up data")
    })?;
    Ok(())
}

#[tracing::instrument(name = "Fetch lookup data", skip(pool))]
pub async fn fetch_lookup_data(
    pool: &PgPool,
    subscriber_id: &str,
    np_type: &ONDCNetworkType,
    domain: &ONDCDomain,
    lookup_uri: &str,
) -> Result<Option<LookupData>, anyhow::Error> {
    let look_up_data = get_lookup_data_from_db(pool, subscriber_id, np_type, domain).await?;
    if look_up_data.is_some() {
        return Ok(look_up_data);
    }

    let look_up_data_from_api =
        get_lookup_for_subscriber_by_api(subscriber_id, np_type, domain, lookup_uri).await?;

    if let Some(ref data) = look_up_data_from_api {
        save_lookup_data_to_db(pool, data).await?;
    }

    Ok(look_up_data_from_api)
}

pub fn serialize_timestamp_without_nanos<S>(
    date: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted_date = date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    serializer.serialize_str(&formatted_date)
}

#[allow(clippy::too_many_arguments)]
pub fn get_common_context(
    transaction_id: Uuid,
    message_id: Uuid,
    domain_category_code: &CategoryDomain,
    action: ONDCActionType,
    bap_id: &str,
    bap_uri: &str,
    bpp_id: Option<&str>,
    bpp_uri: Option<&str>,
    country_code: &CountryCode,
    city_code: &str,
    ttl: Option<&str>,
) -> Result<ONDCContext, anyhow::Error> {
    let ondc_domain = ONDCDomain::get_ondc_domain(domain_category_code);
    Ok(ONDCContext {
        domain: ondc_domain,
        location: ONDCContextLocation {
            city: ONDCContextCity {
                code: city_code.to_owned(),
            },
            country: ONDCContextCountry {
                code: country_code.clone(),
            },
        },
        action,
        version: ONDCVersion::V2point2,
        transaction_id: transaction_id.to_owned(),
        message_id: message_id.to_owned(),
        bap_id: bap_id.to_string(),
        bap_uri: bap_uri.to_string(),
        timestamp: Utc::now(),
        bpp_id: bpp_id.map(|s| s.to_string()),
        bpp_uri: bpp_uri.map(|s| s.to_string()),
        ttl: ttl.map_or_else(|| ONDC_TTL.to_owned(), |s| s.to_string()),
    })
}

fn get_buyer_id_tag(business_account: &BusinessAccount) -> Result<ONDCTag, anyhow::Error> {
    let vector_obj = get_vector_val_from_list(
        &business_account.default_vector_type,
        &business_account.vectors,
    );
    let ondc_buyer_id_type = &business_account
        .default_vector_type
        .get_ondc_vector_type()?;
    match vector_obj {
        Some(vector) => Ok(ONDCTag::get_buyer_id_tag(ondc_buyer_id_type, &vector.value)),
        None => Err(anyhow!(
            "Failed to get buyer ID tag: {}",
            &business_account.default_vector_type.to_string()
        )),
    }
}

#[tracing::instrument(name = "get search tag", skip())]
fn get_search_tags(
    business_account: &BusinessAccount,
    np_detail: &RegisteredNetworkParticipant,
) -> Result<Vec<ONDCTag>, ProductSearchError> {
    match get_buyer_id_tag(business_account) {
        Ok(id_tag) => Ok(vec![
            ONDCTag::get_buyer_fee_tag(
                ONDCFeeType::get_fee_type(&np_detail.fee_type),
                &np_detail.fee_value.to_string(),
            ),
            id_tag,
        ]),
        Err(e) => {
            return Err(ProductSearchError::ValidationError(format!(
                "Failed to get buyer ID tag: {}",
                e
            )));
        }
    }
}

#[tracing::instrument(name = "get search fulfillment stops", skip())]
pub fn get_search_fulfillment_stops(
    fulfillment_locations: Option<&Vec<ProductFulFillmentLocation>>,
) -> Option<Vec<ONDCSearchStop>> {
    let mut ondc_fulfillment_stops = Vec::new();
    match fulfillment_locations {
        Some(locations) => {
            for location in locations {
                let search_fulfillment_end_obj = ONDCSearchStop {
                    r#type: ONDCFulfillmentStopType::End,
                    location: ONDCSearchLocation {
                        gps: get_gps_string(location.latitude, location.longitude),
                        area_code: location.area_code.to_string(),
                    },
                };
                ondc_fulfillment_stops.push(search_fulfillment_end_obj);
            }
            Some(ondc_fulfillment_stops)
        }
        None => None,
    }
}

fn get_search_by_item(search_request: &ProductSearchRequest) -> Option<ONDCSearchItem> {
    if search_request.search_type == ProductSearchType::Item {
        return Some(ONDCSearchItem {
            descriptor: Some(ONDCSearchDescriptor {
                name: search_request.query.to_owned(),
            }),
        });
    }

    None
}

fn get_search_by_category(search_request: &ProductSearchRequest) -> Option<ONDCSearchCategory> {
    if search_request.search_type == ProductSearchType::Category {
        return Some(ONDCSearchCategory {
            id: search_request.query.to_owned(),
        });
    }

    None
}

pub fn get_ondc_search_payment_obj(payment_obj: &Option<PaymentType>) -> Option<ONDCSearchPayment> {
    match payment_obj {
        Some(_) => payment_obj.as_ref().map(|obj| ONDCSearchPayment {
            r#type: PaymentType::get_ondc_payment(obj),
        }),
        None => None,
    }
}

#[tracing::instrument(name = "get search fulfillment obj", skip())]
pub fn get_search_fulfillment_obj(
    fulfillment_type: &Option<FulfillmentType>,
    locations: Option<&Vec<ProductFulFillmentLocation>>,
) -> Option<ONDCSearchFulfillment> {
    if let Some(fulfillment_type) = fulfillment_type {
        return Some(ONDCSearchFulfillment {
            r#type: fulfillment_type.get_ondc_fulfillment_type(),
            stops: get_search_fulfillment_stops(locations),
        });
    }

    None
}

#[tracing::instrument(name = "get ondc search message obj", skip())]
pub fn get_ondc_search_message_obj(
    _user_account: &UserAccount,
    business_account: &BusinessAccount,
    search_request: &ProductSearchRequest,
    np_detail: &RegisteredNetworkParticipant,
) -> Result<ONDCSearchMessage, ProductSearchError> {
    let mut fulfillment_obj = None;
    let mut payment_obj = None;
    if search_request.search_type != ProductSearchType::City {
        fulfillment_obj = get_search_fulfillment_obj(
            &search_request.fulfillment_type,
            search_request.fulfillment_locations.as_ref(),
        );
        payment_obj = get_ondc_search_payment_obj(&search_request.payment_type);
    }

    Ok(ONDCSearchMessage {
        intent: ONDCSearchIntent {
            fulfillment: fulfillment_obj,
            tags: get_search_tags(business_account, np_detail)?,
            payment: payment_obj,
            item: get_search_by_item(search_request),

            provider: None,
            category: get_search_by_category(search_request),
        },
    })
}

#[tracing::instrument(name = "get ondc search payload", skip())]
pub fn get_ondc_search_payload(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    search_request: &ProductSearchRequest,
    np_detail: &RegisteredNetworkParticipant,
) -> Result<ONDCSearchRequest, anyhow::Error> {
    let ondc_context = get_common_context(
        search_request.transaction_id,
        search_request.message_id,
        &search_request.domain_category_code,
        ONDCActionType::Search,
        &np_detail.subscriber_id,
        &np_detail.subscriber_uri,
        None,
        None,
        &search_request.country_code,
        &search_request.city_code,
        None,
    )?;
    let ondc_seach_message =
        get_ondc_search_message_obj(user_account, business_account, search_request, np_detail)?;
    Ok(ONDCSearchRequest {
        context: ondc_context,
        message: ondc_seach_message,
    })
}

#[tracing::instrument(name = "Send ONDC Payload")]
pub async fn send_ondc_payload(
    url: &str,
    payload: &str,
    header: &str,
    action: ONDCActionType,
) -> Result<ONDCResponse<ONDCErrorCode>, anyhow::Error> {
    let final_url = format!("{}/{}", url, action);
    let client = Client::new();
    let mut header_map = HashMap::new();
    header_map.insert("Authorization", header);
    let network_call = NetworkCall { client };
    let result = network_call
        .async_post_call_with_retry(&final_url, Some(payload), Some(header_map))
        .await;

    match result {
        Ok(response) => {
            // println!("{:?}", &response);
            let response_obj: ONDCResponse<ONDCErrorCode> = serde_json::from_value(response)?;
            if let Some(error) = response_obj.error {
                Err(anyhow!(
                    "{} {}",
                    error.message,
                    error.path.unwrap_or("".to_string())
                ))
            } else {
                Ok(response_obj)
            }
        }
        Err(err) => {
            println!("{}", err);
            Err(anyhow::Error::from(err))
        }
    }
}

#[tracing::instrument(name = "Fetch Search WebSocket Params", skip())]
pub fn get_websocket_params_from_search_req(search_model: SearchRequestModel) -> WebSocketParam {
    WebSocketParam {
        user_id: Some(search_model.user_id),
        business_id: search_model.business_id,
        device_id: Some(search_model.device_id),
    }
}

#[tracing::instrument(name = "Fetch Product Search Params", skip(pool))]
pub async fn get_product_search_params(
    pool: &PgPool,
    transaction_id: Uuid,
    message_id: Uuid,
) -> Result<Option<SearchRequestModel>, anyhow::Error> {
    let row = sqlx::query_as!(
        SearchRequestModel,
        r#"SELECT transaction_id, user_id, business_id, device_id, update_cache
        FROM search_request
        WHERE transaction_id = $1 AND message_id = $2 ORDER BY created_on DESC
        "#,
        transaction_id,
        message_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub fn get_ondc_order_param_from_req(ondc_req: &ONDCRequestModel) -> WebSocketParam {
    WebSocketParam {
        device_id: None,
        user_id: None,
        business_id: ondc_req.business_id,
    }
}

// pub fn _get_order_param_from_param_req(ondc_req: &OrderRequestParamsModel) -> WebSocketParam {
//     WebSocketParam {
//         device_id: ondc_req.device_id.clone(),
//         user_id: Some(ondc_req.user_id),
//         business_id: ondc_req.business_id,
//     }
// }

pub fn get_ondc_order_param_from_commerce(ondc_req: &Commerce) -> WebSocketParam {
    WebSocketParam {
        device_id: None,
        user_id: None,
        business_id: ondc_req.buyer_id,
    }
}

#[tracing::instrument(name = "get price obj from ondc price obj", skip())]
pub fn get_price_obj_from_ondc_price_obj(
    price: &ONDCOnSearchItemPrice,
    tax: &BigDecimal,
) -> Result<WSSearchItemPrice, anyhow::Error> {
    return Ok(WSSearchItemPrice {
        currency: price.currency.to_owned(),
        price_with_tax: BigDecimal::from_str(&price.value).unwrap_or(BigDecimal::from(0)),
        price_without_tax: price.get_price_without_tax(tax),
        offered_value: price
            .offered_value
            .as_ref()
            .map(|v| BigDecimal::from_str(v).unwrap_or_else(|_| BigDecimal::from(0))),
        maximum_price: BigDecimal::from_str(&price.maximum_value).unwrap(),
    });
}

#[tracing::instrument(name = "get ws location mapping", skip())]
fn get_ws_location_mapping(
    ondc_location: &ONDCOnSearchProviderLocation,
) -> WSSearchProviderLocation {
    WSSearchProviderLocation {
        id: ondc_location.id.clone(),
        gps: ondc_location.gps.clone(),
        address: ondc_location.address.clone(),
        city: WSSearchCity {
            code: ondc_location.city.code.clone(),
            name: ondc_location.city.name.clone(),
        },
        country: WSSearchCountry {
            code: ondc_location.country.code.clone(),
            name: ondc_location.country.name.clone(),
        },
        state: WSSearchState {
            code: ondc_location.state.code.clone(),
            name: ondc_location.state.name.clone(),
        },
        area_code: ondc_location.area_code.clone(),
    }
}

#[tracing::instrument(name = "ws search provider from ondc provider", skip())]
pub fn ws_search_provider_from_ondc_provider(
    id: &str,
    rating: &Option<String>,
    descriptor: &ONDCOnSearchProviderDescriptor,
    ttl: &str,
    fulfillments: &Vec<ONDCOnSearchFulfillmentContact>,
    tags: &Vec<ONDCTag>,
    ondc_credentials: &Option<Vec<ONDCCredential>>,
) -> WSSearchProductProviderDescription {
    let images: Vec<String> = descriptor
        .images
        .iter()
        .map(|image| image.get_value().to_owned())
        .collect();
    let contact = fulfillments.first();
    let gst_credit_invoice = get_tag_value_from_list(
        tags,
        ONDCTagType::SellerTerms,
        &ONDCTagItemCode::GstCreditInvoice.to_string(),
    )
    .unwrap_or("N")
    .to_uppercase()
        == "Y";
    let seller_id_code = get_tag_value_from_list(
        tags,
        ONDCTagType::SellerId,
        &ONDCTagItemCode::SellerIdCode.to_string(),
    )
    .unwrap_or("NA");
    let seller_id_value = get_tag_value_from_list(
        tags,
        ONDCTagType::SellerId,
        &ONDCTagItemCode::SellerIdNo.to_string(),
    )
    .unwrap_or("NA");
    // let fssai_numbers = ;
    // .collect();

    let rating: Option<f32> = rating.as_ref().and_then(|f| f.parse().ok());
    let mut credentials = vec![];
    if let Some(ondc_credentials) = ondc_credentials {
        for cred in ondc_credentials.iter() {
            credentials.push(WSSearchProviderCredential {
                id: cred.id.to_owned(),
                r#type: cred.r#type.get_credential_type(),
                desc: cred.desc.to_owned(),
                url: Some(cred.url.to_owned()),
            })
        }
    }
    if let Some(fssai_num) = tags
        .iter()
        .find(|tag| tag.descriptor.code == ONDCTagType::FssaiLicenseNo)
    {
        for data in fssai_num.list.iter() {
            if data.value.len() > 0 {
                let desc = format!(
                    "{} {}",
                    "FSSAI_LICENSE_NO",
                    data.descriptor.code.to_string().to_uppercase()
                );
                credentials.push(WSSearchProviderCredential {
                    id: data.value.to_owned(),
                    r#type: CredentialType::FssaiLicenseNo,
                    desc,
                    url: None,
                })
            }
        }
    }

    WSSearchProductProviderDescription {
        id: id.to_string(),
        rating,
        name: descriptor.name.clone(),
        code: descriptor.code.clone(),
        short_desc: descriptor.short_desc.clone(),
        long_desc: descriptor.long_desc.clone(),
        images,
        ttl: ttl.to_owned(),
        contact: WSSearchProviderContact {
            mobile_no: contact
                .as_ref()
                .map_or("NA".to_string(), |f| f.contact.phone.to_owned()),
            email: contact.as_ref().and_then(|f| f.contact.email.to_owned()),
        },
        terms: WSSearchProviderTerms { gst_credit_invoice },
        identifications: vec![WSSearchProviderID {
            r#type: seller_id_code.to_owned(),
            value: seller_id_value.to_owned(),
        }],
        credentials,
    }
}

// #[tracing::instrument(name = "get ws quantity from ondc quantity", skip())]
fn get_ws_quantity_from_ondc_quantity(
    ondc_quantity: &ONDCOnSearchItemQuantity,
) -> WSSearchItemQuantity {
    WSSearchItemQuantity {
        unitized: WSSearchItemQtyMeasure {
            unit: ondc_quantity.unitized.measure.unit.clone(),
            value: BigDecimal::from_str(&ondc_quantity.unitized.measure.value)
                .unwrap_or_else(|_| BigDecimal::from(0)),
        },
        available: WSSearchItemQty {
            measure: WSSearchItemQtyMeasure {
                unit: ondc_quantity.available.measure.unit.clone(),
                value: BigDecimal::from_str(&ondc_quantity.available.measure.value)
                    .unwrap_or_else(|_| BigDecimal::from(0)),
            },
            count: ondc_quantity.available.count,
        },
        maximum: WSSearchItemQty {
            measure: WSSearchItemQtyMeasure {
                unit: ondc_quantity.maximum.measure.unit.clone(),
                value: BigDecimal::from_str(&ondc_quantity.maximum.measure.value)
                    .unwrap_or_else(|_| BigDecimal::from(0)),
            },
            count: ondc_quantity.maximum.count,
        },
        minimum: ondc_quantity
            .minimum
            .as_ref()
            .map(|min_qty| WSSearchItemQty {
                measure: WSSearchItemQtyMeasure {
                    unit: min_qty.measure.unit.clone(),
                    value: BigDecimal::from_str(&min_qty.measure.value)
                        .unwrap_or_else(|_| BigDecimal::from(0)),
                },
                count: min_qty.count,
            }),
    }
}

fn get_ws_price_slab_from_ondc_slab(
    ondc_tags: &[ONDCOnSearchItemTag],
    tax_rate: &BigDecimal,
) -> Option<Vec<WSPriceSlab>> {
    let mut price_slabs = vec![];
    for tag in ondc_tags
        .iter()
        .filter(|t| matches!(t.descriptor.code, ONDCTagType::PriceSlab))
    {
        let min = tag
            .get_tag_value(&ONDCTagItemCode::MinPackSize.to_string())
            .and_then(|value| BigDecimal::from_str(value).ok())
            .unwrap_or_else(|| BigDecimal::from(0));

        let max = tag
            .get_tag_value(&ONDCTagItemCode::MaxPackSize.to_string())
            .and_then(|value| {
                if value.is_empty() {
                    None
                } else {
                    BigDecimal::from_str(value).ok()
                }
            });

        let price_with_tax = tag
            .get_tag_value(&ONDCTagItemCode::UnitSalePrice.to_string())
            .and_then(|value| BigDecimal::from_str(value).ok())
            .unwrap_or_else(|| BigDecimal::from(0));

        let price_without_tax = price_with_tax.clone() / (BigDecimal::from(1) + tax_rate);

        price_slabs.push(WSPriceSlab {
            min,
            max,
            price_with_tax,
            price_without_tax,
        });
    }
    if price_slabs.is_empty() {
        None
    } else {
        Some(price_slabs)
    }
}

fn get_ws_attributes_from_ondc(ondc_tags: &[ONDCOnSearchItemTag]) -> Vec<WSSearchItemAttribute> {
    println!("{:?}", ondc_tags);
    let mut data: Vec<WSSearchItemAttribute> = vec![];
    let attribute_tag = ondc_tags
        .iter()
        .find(|t| matches!(t.descriptor.code, ONDCTagType::Attribute));
    if let Some(attribute_tag) = attribute_tag {
        for attribute in attribute_tag.list.iter() {
            data.push(WSSearchItemAttribute {
                label: "".to_owned(),
                key: attribute.descriptor.code.to_owned(),
                value: attribute.value.to_owned(),
            })
        }
    }

    data
}

fn map_ws_item_categories(category_ids: &[String]) -> Vec<WSProductCategory> {
    category_ids
        .iter()
        .map(|f| WSProductCategory {
            code: f.to_string(),
            name: "".to_owned(),
        })
        .collect()
}

fn map_item_images(images: &[ONDCImage], tags: &[ONDCOnSearchItemTag]) -> Vec<String> {
    images
        .iter()
        .map(|image| image.get_value().to_owned())
        .chain(
            get_search_tag_item_value(tags, &ONDCTagType::Image, &ONDCTagItemCode::Url.to_string())
                .into_iter()
                .map(|s| s.to_owned()),
        )
        .collect()
}

fn get_payment_mapping(payment_objs: &[ONDCOnSearchPayment]) -> HashMap<String, WSPaymentTypes> {
    payment_objs
        .iter()
        .map(|f| (f.id.to_owned(), get_ws_search_item_payment_objs(f)))
        .collect()
}

// #[tracing::instrument(name = "get ws search item payment objs", skip())]
fn get_ws_search_item_payment_objs(ondc_payment_obj: &ONDCOnSearchPayment) -> WSPaymentTypes {
    WSPaymentTypes {
        r#type: ondc_payment_obj.r#type.get_payment(),
        collected_by: ondc_payment_obj
            .collected_by
            .clone()
            .unwrap_or(ONDCNetworkType::Bap),
    }
}

fn handle_geojson_servicability_in_ondc_search(
    servicability_val: &str,
    category_code: &Option<String>,
) -> Result<Vec<WSServicabilityData<Value>>, anyhow::Error> {
    let mut geo_json_duplicate = vec![];
    let mut final_data = vec![];
    match servicability_val.parse::<GeoJson>() {
        Ok(GeoJson::FeatureCollection(ref ctn)) => {
            for feature in &ctn.features {
                if let Some(ref geom) = feature.geometry {
                    if geo_json_duplicate.contains(geom) {
                        continue;
                    }
                    geo_json_duplicate.push(geom.clone());
                    let json_value: Value = serde_json::to_value(geom)?;

                    let data = WSServicabilityData {
                        category_code: category_code.clone(),
                        value: json_value,
                    };
                    final_data.push(data)
                }
            }
        }
        Ok(_) => (),
        Err(e) => return Err(anyhow!(e)),
    }
    Ok(final_data)
}

fn handle_coordinates_servicability_in_ondc_search(
    servicability_val: &str,
    category_code: &Option<String>,
) -> Result<WSServicabilityData<Value>, anyhow::Error> {
    let coordinates: Vec<ONDCServicabilityCoordinate> = serde_json::from_str(servicability_val)?;
    let polygon_coordinates: Vec<Vec<f64>> = coordinates
        .into_iter()
        .map(|coord| vec![coord.lng, coord.lat])
        .collect();
    let geometry = Geometry::new(geojson::Value::Polygon(vec![polygon_coordinates]));
    let json_value: Value = serde_json::to_value(geometry)?;
    Ok(WSServicabilityData {
        category_code: category_code.clone(),
        value: json_value,
    })
}

fn handle_hyperlocal_servicability_in_ondc_search(
    servicability_val: &str,
    category_code: &Option<String>,
) -> WSServicabilityData<f64> {
    let num: f64 = servicability_val.parse().unwrap_or(0.001);
    WSServicabilityData {
        category_code: category_code.clone(),
        value: num * 1000.0,
    }
}

fn handle_country_servicability_in_ondc_search(
    servicability_val: &str,
    category_code: &Option<String>,
) -> Result<WSServicabilityData<CountryCode>, anyhow::Error> {
    let country_code = CountryCode::from_str(servicability_val).map_err(|e| {
        tracing::error!(e);
        anyhow!(e)
    })?;
    Ok(WSServicabilityData {
        category_code: category_code.clone(),
        value: country_code,
    })
}

fn expand_pincodes(pincode_str: &str) -> HashSet<String> {
    let mut pincodes = HashSet::new(); // HashSet to store unique pincodes

    for part in pincode_str.split(',').map(|p| p.trim()) {
        if let Some((start, end)) = part.split_once('-') {
            if let (Ok(start), Ok(end)) = (start.parse::<u32>(), end.parse::<u32>()) {
                for code in start..=end {
                    pincodes.insert(code.to_string());
                }
            }
        } else {
            pincodes.insert(part.to_string()); // Insert single pincode
        }
    }

    pincodes
}

fn handle_intercity_servicability_in_ondc_search(
    servicability_val: &str,
    category_code: &Option<String>,
) -> Result<WSServicabilityData<HashSet<String>>, anyhow::Error> {
    Ok(WSServicabilityData {
        category_code: category_code.clone(),
        value: expand_pincodes(servicability_val),
    })
}

#[tracing::instrument(name = "get product from on search request", skip())]
pub fn get_servicability_from_on_search_request(
    tags: &Vec<ONDCTag>,
) -> Result<HashMap<String, WSSearchServicability>, anyhow::Error> {
    let mut location_mapping: HashMap<String, WSSearchServicability> = HashMap::new();
    for tag in tags
        .iter()
        .filter(|t| matches!(t.descriptor.code, ONDCTagType::Serviceability))
    {
        if let (
            Some(location_id),
            Some(category_id),
            Some(servicability_type),
            Some(servicability_val),
        ) = (
            tag.get_tag_value(&ONDCTagItemCode::Location.to_string()),
            tag.get_tag_value(&ONDCTagItemCode::Category.to_string()),
            tag.get_tag_value(&ONDCTagItemCode::Type.to_string()),
            tag.get_tag_value(&ONDCTagItemCode::Val.to_string()),
        ) {
            let category_code = if category_id.ends_with("-*") {
                None
            } else {
                Some(category_id.to_string())
            };

            if servicability_type == "13" {
                let data =
                    handle_geojson_servicability_in_ondc_search(servicability_val, &category_code)?;
                location_mapping
                    .entry(location_id.to_string())
                    .or_insert(WSSearchServicability {
                        geo_json: vec![],
                        hyperlocal: vec![],
                        country: vec![],
                        intercity: vec![],
                    })
                    .geo_json
                    .extend(data);
            } else if servicability_type == "14" {
                let data = handle_coordinates_servicability_in_ondc_search(
                    servicability_val,
                    &category_code,
                )?;
                location_mapping
                    .entry(location_id.to_string())
                    .or_insert(WSSearchServicability {
                        geo_json: vec![],
                        hyperlocal: vec![],
                        country: vec![],
                        intercity: vec![],
                    })
                    .geo_json
                    .push(data);
            } else if servicability_type == "10" {
                let data = handle_hyperlocal_servicability_in_ondc_search(
                    servicability_val,
                    &category_code,
                );
                location_mapping
                    .entry(location_id.to_string())
                    .or_insert(WSSearchServicability {
                        geo_json: vec![],
                        hyperlocal: vec![],
                        country: vec![],
                        intercity: vec![],
                    })
                    .hyperlocal
                    .push(data);
            } else if servicability_type == "12" {
                if let Ok(data) =
                    handle_country_servicability_in_ondc_search(servicability_val, &category_code)
                {
                    location_mapping
                        .entry(location_id.to_string())
                        .or_insert(WSSearchServicability {
                            geo_json: vec![],
                            hyperlocal: vec![],
                            country: vec![],
                            intercity: vec![],
                        })
                        .country
                        .push(data);
                }
            } else if servicability_type == "11" {
                if let Ok(data) =
                    handle_intercity_servicability_in_ondc_search(servicability_val, &category_code)
                {
                    location_mapping
                        .entry(location_id.to_string())
                        .or_insert(WSSearchServicability {
                            geo_json: vec![],
                            hyperlocal: vec![],
                            country: vec![],
                            intercity: vec![],
                        })
                        .intercity
                        .push(data);
                }
            }
        }
    }

    Ok(location_mapping)
}

fn get_variant_mapping(variants: &Vec<ONDCOnSearchCategory>) -> HashMap<String, WSSearchVariant> {
    let mut map = HashMap::new();
    for variant in variants {
        let mut attributes = vec![];
        for tag in variant
            .tags
            .iter()
            .filter(|t| matches!(t.descriptor.code, ONDCTagType::Attr))
        {
            if let (Some(name), Some(sequence)) = (
                tag.get_tag_value(&ONDCTagItemCode::Name.to_string()),
                tag.get_tag_value(&ONDCTagItemCode::Seq.to_string()),
            ) {
                let new_name = if name.contains("attribute") {
                    name.replace("tags.", "")
                } else {
                    name.to_owned()
                };

                attributes.push(WSSearchVariantAttribute {
                    attribute_code: new_name,
                    sequence: sequence.to_owned(),
                })
            }
        }
        if !attributes.is_empty() {
            map.insert(
                variant.id.clone(),
                WSSearchVariant {
                    name: variant.descriptor.name.clone(),
                    attributes,
                },
            );
        }
    }
    map
}
fn get_cancelletion_terms(item: &ONDCOnSearchItem) -> WSItemCancellation {
    let mut terms = vec![];
    let is_cancellable = get_search_tag_item_value(
        &item.tags,
        &ONDCTagType::G2,
        &ONDCTagItemCode::Cancellable.to_string(),
    )
    .unwrap_or("false")
        == "true";

    for term in item.cancellation_terms.iter() {
        let fee = match &term.cancellation_fee {
            ONDCItemCancellationFee::Percentage { percentage } => {
                let percentage_value = percentage.parse::<f64>().unwrap_or(0.0);
                WSItemCancellationFee::Percent {
                    percentage: percentage_value,
                }
            }
            ONDCItemCancellationFee::Amount { amount } => {
                let amount_value = amount.value.parse::<f64>().unwrap_or(0.0);
                WSItemCancellationFee::Amount {
                    amount: amount_value,
                }
            }
        };
        terms.push(WSItemCancellationTerm {
            fulfillment_state: term
                .fulfillment_state
                .descriptor
                .code
                .get_fulfillment_state(),
            reason_required: term.reason_required,
            fee,
        })
    }
    WSItemCancellation {
        is_cancellable,
        terms,
    }
}

#[tracing::instrument(name = "get product from on search request", skip())]
pub fn get_product_from_on_search_request(
    on_search_obj: &ONDCOnSearchRequest,
) -> Result<Option<WSSearchData>, anyhow::Error> {
    let subscriber_id = on_search_obj.context.bpp_id.as_deref().unwrap_or("");
    // let subscriber_uri = on_search_obj.context.bpp_uri.as_deref().unwrap_or("");
    if let Some(catalog) = &on_search_obj.message.catalog {
        let mut payment_mapping = get_payment_mapping(&catalog.payments);
        let mut provider_list: Vec<WSSearchProvider> = vec![];
        let descriptor_image = &catalog.descriptor.images;
        let fullfllments = &catalog.fulfillments;
        let fulfillment_map: HashMap<&String, &ONDCFulfillmentType> =
            fullfllments.iter().map(|f| (&f.id, &f.r#type)).collect();
        let urls = descriptor_image
            .iter()
            .map(|image| image.url.clone())
            .collect();
        for provider_obj in &catalog.providers {
            if let Some(provider_payment_objs) = &provider_obj.payments {
                payment_mapping = get_payment_mapping(provider_payment_objs);
            }
            let location_obj: HashMap<String, WSSearchProviderLocation> = provider_obj
                .locations
                .iter()
                .map(|f| (f.id.clone(), get_ws_location_mapping(f)))
                .collect();

            // let provider_payment_obj = &provider.payments;
            let mut product_list: Vec<WSSearchItem> = vec![];
            for item in &provider_obj.items {
                let tax_rate = get_search_tag_item_value(
                    &item.tags,
                    &ONDCTagType::Origin,
                    &ONDCTagItemCode::Country.to_string(),
                )
                .unwrap_or("0.00");
                let tax = BigDecimal::from_str(tax_rate).unwrap_or_else(|_| BigDecimal::from(0));
                let country_of_origin = get_search_tag_item_value(
                    &item.tags,
                    &ONDCTagType::Origin,
                    &ONDCTagItemCode::Country.to_string(),
                )
                .map(|e| e.to_owned());
                let time_to_ship = get_search_tag_item_value(
                    &item.tags,
                    &ONDCTagType::G2,
                    &ONDCTagItemCode::TimeToShip.to_string(),
                )
                .unwrap_or("NA");

                let fulfillment_type_list: Vec<FulfillmentType> = item
                    .fulfillment_ids
                    .iter()
                    .filter_map(|key| {
                        fulfillment_map
                            .get(key)
                            .map(|f| f.get_fulfillment_from_ondc())
                    })
                    .collect();
                let images = map_item_images(&item.descriptor.images, &item.tags);

                let price_slabs = get_ws_price_slab_from_ondc_slab(&item.tags, &tax);
                let categories: Vec<WSProductCategory> = map_ws_item_categories(&item.category_ids);
                let videos: Vec<String> = item.descriptor.media.as_ref().map_or(vec![], |media| {
                    media
                        .iter()
                        .filter_map(|desc| {
                            if desc.mimetype == OnSearchContentType::Mp4 {
                                Some(desc.url.to_owned())
                            } else {
                                None
                            }
                        })
                        .collect()
                });
                let validity = item.time.as_ref().map(|e| WSItemValidity {
                    start: e.range.start,
                    end: e.range.end,
                });
                let replacement_terms = item
                    .replacement_terms
                    .iter()
                    .map(|a| WSItemReplacementTerm {
                        replace_within: a.replace_within.to_owned(),
                    })
                    .collect();
                let return_terms = item
                    .return_terms
                    .iter()
                    .map(|f| WSItemReturnTerm {
                        fulfillment_state: f
                            .fulfillment_state
                            .descriptor
                            .code
                            .get_fulfillment_state(),
                        return_eligible: f.return_eligible,
                        return_time: WSItemReturnTime {
                            duration: f.return_time.duration.to_owned(),
                        },
                        return_location: WSItemReturnLocation {
                            address: f.return_location.address.to_owned(),
                            gps: f.return_location.gps.to_owned(),
                        },
                        fulfillment_managed_by: f.fulfillment_managed_by.to_owned(),
                    })
                    .collect();
                let cancellation_terms = get_cancelletion_terms(item);
                let attributes = get_ws_attributes_from_ondc(&item.tags);
                let payment_options = item
                    .payment_ids
                    .iter()
                    .filter_map(|a| payment_mapping.get(a).map(|a| a.r#type.clone()))
                    .collect();
                let prod_obj = WSSearchItem {
                    id: item.id.clone(),
                    name: item.descriptor.name.clone(),
                    long_desc: item.descriptor.long_desc.clone(),
                    short_desc: item.descriptor.short_desc.clone(),
                    code: item.descriptor.code.clone(),
                    domain_category: on_search_obj.context.domain.get_category_domain(),
                    price: get_price_obj_from_ondc_price_obj(&item.price, &tax)?,
                    parent_item_id: item.parent_item_id.clone(),
                    recommended: item.recommended,
                    creator: WSProductCreator {
                        name: item.creator.descriptor.name.clone(),
                        contact: WSCreatorContactData {
                            name: item.creator.descriptor.contact.name.clone(),
                            address: item.creator.descriptor.contact.address.full.clone(),
                            phone: item.creator.descriptor.contact.phone.clone(),
                            email: item.creator.descriptor.contact.email.clone(),
                        },
                    },
                    fullfillment_options: fulfillment_type_list,
                    images,
                    videos,
                    location_ids: item.location_ids.to_owned(),
                    payment_options,
                    categories,
                    tax_rate: tax,
                    quantity: get_ws_quantity_from_ondc_quantity(&item.quantity),
                    price_slabs,
                    attributes,
                    matched: item.matched,
                    country_of_origin,
                    time_to_ship: time_to_ship.to_owned(),
                    validity,
                    replacement_terms,
                    return_terms,
                    cancellation_terms,
                };
                product_list.push(prod_obj)
            }
            let servicability = get_servicability_from_on_search_request(&provider_obj.tags)?;

            let provider = WSSearchProvider {
                items: product_list,
                locations: location_obj,
                description: ws_search_provider_from_ondc_provider(
                    &provider_obj.id,
                    &provider_obj.rating,
                    &provider_obj.descriptor,
                    &provider_obj.ttl,
                    &provider_obj.fulfillments,
                    &provider_obj.tags,
                    &provider_obj.creds,
                ),
                servicability,
                // payments: payment_mapping.clone(),
                variants: provider_obj.categories.as_ref().map(get_variant_mapping),
            };
            provider_list.push(provider)
        }
        return Ok(Some(WSSearchData {
            providers: provider_list,
            bpp: WSSearchBPP {
                name: catalog.descriptor.name.clone(),
                subscriber_id: subscriber_id.to_owned(),
                code: catalog.descriptor.code.clone(),
                short_desc: catalog.descriptor.short_desc.clone(),
                long_desc: catalog.descriptor.long_desc.clone(),
                images: urls,
            },
        }));
    }

    Ok(None)
}

#[tracing::instrument(name = "get search ws body", skip())]
pub fn get_search_ws_body(
    message_id: Uuid,
    transaction_id: Uuid,
    search_data: WSSearchData,
) -> WSSearch {
    WSSearch {
        message_id,
        transaction_id,
        message: search_data,
    }
}

// #[tracing::instrument(name = "get search tag item  list from tag", skip())]
fn search_tag_item_list_from_tag<'a>(
    tag: &'a [ONDCOnSearchItemTag],
    tag_descriptor_code: &ONDCTagType,
) -> Vec<&'a ONDCOnSearchItemTag> {
    tag.iter()
        .filter(|t| &t.descriptor.code == tag_descriptor_code)
        .collect()
}

#[tracing::instrument(name = "get search tag item value", skip())]
pub fn get_search_tag_item_value<'a>(
    tag: &'a [ONDCOnSearchItemTag],
    tag_descriptor_code: &ONDCTagType,
    search_item_tag_type: &str,
) -> Option<&'a str> {
    let item_tag_list = search_tag_item_list_from_tag(tag, tag_descriptor_code);
    if !item_tag_list.is_empty() {
        item_tag_list[0].get_tag_value(search_item_tag_type)
    } else {
        None
    }
}

#[tracing::instrument(name = "get select context", skip())]
fn get_ondc_select_context(
    select_request: &OrderSelectRequest,
    bap_detail: &RegisteredNetworkParticipant,
    bpp_detail: &LookupData,
) -> Result<ONDCContext, anyhow::Error> {
    get_common_context(
        select_request.transaction_id,
        select_request.message_id,
        &select_request.domain_category_code,
        ONDCActionType::Select,
        &bap_detail.subscriber_id,
        &bap_detail.subscriber_uri,
        Some(&bpp_detail.subscriber_id),
        Some(&bpp_detail.subscriber_url),
        &select_request.fulfillments[0].location.country.code,
        &select_request.fulfillments[0].location.city.code,
        Some(&select_request.ttl),
    )
}

#[tracing::instrument(name = "get ondc select order provider", skip())]
fn get_ondc_select_order_provider(
    location_ids: &HashSet<&str>,
    provider_id: &str,
    ttl: &str,
) -> ONDCSelectProvider {
    let location_objs = location_ids
        .iter()
        .map(|id| ONDCLocationId { id: id.to_string() })
        .collect();
    ONDCSelectProvider {
        id: provider_id.to_owned(),
        locations: location_objs,
        ttl: ttl.to_owned(),
    }
}

fn get_ondc_select_payment_obs(payment_types: &[PaymentType]) -> Vec<ONDCSelectPayment> {
    payment_types
        .iter()
        .map(|payment| ONDCSelectPayment {
            r#type: payment.get_ondc_payment(),
        })
        .collect()
}

fn get_ondc_select_tags(
    business_account: &BusinessAccount,
    chat_data: &Option<ChatData>,
) -> Result<Vec<ONDCTag>, anyhow::Error> {
    let mut tags = match get_buyer_id_tag(business_account) {
        Ok(tag_option) => Ok(vec![tag_option]),
        Err(e) => Err(anyhow!("Failed to get buyer ID tag: {}", e)),
    }?;
    if let Some(chat_data) = chat_data {
        tags.push(ONDCTag::get_chat_tag(chat_data))
    }
    Ok(tags)
}

#[tracing::instrument(name = "get ondc select order item", skip())]
fn get_ondc_select_item_tags(
    order_type: &OrderType,
    buyer_terms: &Option<BuyerTerms>,
) -> Option<Vec<ONDCTag>> {
    if order_type == &OrderType::PurchaseOrder {
        if let Some(terms) = buyer_terms {
            return Some(vec![ONDCTag::get_item_tags(
                &terms.item_req,
                &terms.packaging_req,
            )]);
        }
    }
    None
}

#[tracing::instrument(name = "get ondc select order item", skip())]
fn get_ondc_select_order_item(
    order_type: &OrderType,
    items: &Vec<OrderSelectItem>,
) -> Vec<ONDCSelectedItem> {
    let mut ondc_item_objs: Vec<ONDCSelectedItem> = vec![];

    for item in items {
        ondc_item_objs.push(ONDCSelectedItem {
            id: item.item_id.clone(),
            location_ids: item.location_ids.clone(),
            fulfillment_ids: item.fulfillment_ids.clone(),
            quantity: ONDCQuantitySelect {
                selected: ONDCQuantityCountInt { count: item.qty },
            },
            tags: get_ondc_select_item_tags(order_type, &item.buyer_term),
            payment_ids: None,
        })
    }
    return ondc_item_objs;
}

fn get_fulfillment_tags(delivery_terms: &Option<OrderDeliveyTerm>) -> Option<Vec<ONDCTag>> {
    delivery_terms.as_ref().map(|terms| {
        vec![ONDCTag::get_delivery_terms(
            &terms.inco_terms,
            &terms.place_of_delivery,
        )]
    })
}

#[tracing::instrument(name = "getondc select fulfillment end", skip())]
fn get_ondc_select_fulfillment_end(
    location: &SelectFulfillmentLocation,
) -> Vec<ONDCOrderFulfillmentEnd> {
    // let mut fulfillment_end: Vec<ONDCOrderFulfillmentEnd<ONDCSelectFulfillmentLocation>> = vec![];
    // for location in locations {
    vec![ONDCOrderFulfillmentEnd {
        r#type: ONDCFulfillmentStopType::End,
        location: ONDCSelectFulfillmentLocation {
            gps: location.gps.clone(),
            address: Some(location.address.to_string()),
            area_code: location.area_code.clone(),
            city: ONDCCity {
                name: location.city.name.clone(),
            },
            country: ONDCCountry {
                code: location.country.code.clone(),
            },
            state: ONDCState {
                name: location.state.clone(),
            },
        },
        contact: ONDCContact {
            email: None,
            phone: location.contact_mobile_no.clone(),
        },
    }]

    // fulfillment_end
}

fn get_ondc_customer_detail(
    business_account: &BusinessAccount,
    trade_type: Option<&TradeType>,
) -> ONDCCustomer {
    let mut creds: Option<Vec<ONDCCredential>> = None;

    if trade_type == Some(&TradeType::Import) {
        creds = get_vector_val_from_list(&VectorType::ImportLicenseNo, &business_account.proofs)
            .and_then(|proof| {
                proof.value.first().map(|first_value| {
                    vec![ONDCCredential {
                        r#type: ONDCCredentialType::License,
                        desc: ONDCCredentialType::License.get_description(&proof.kyc_id),
                        id: proof.kyc_id.clone(),
                        icon: None,
                        url: first_value.to_owned(),
                    }]
                })
            });
    };

    ONDCCustomer {
        person: ONDCPerson {
            creds,
            name: business_account.company_name.clone(),
        },
    }
}

#[tracing::instrument(name = "get ondc select message body", skip())]
fn get_ondc_select_fulfillments(
    seller_location_mapping: &HashMap<String, ONDCSellerLocationInfo>,
    fulfillments: &Vec<OrderSelectFulfillment>,
    business_account: &BusinessAccount,
) -> Vec<ONDCFulfillment> {
    let mut fulfillment_objs: Vec<ONDCFulfillment> = vec![];
    let location_obj = seller_location_mapping.iter().next().unwrap();

    for fulfillment in fulfillments {
        let mut customer = None;
        let mut tags = None;
        let mut stops = None;
        let trade_type = if location_obj.1.country_code != fulfillment.location.country.code {
            TradeType::Import
        } else {
            TradeType::Domestic
        };

        if fulfillment.r#type == FulfillmentType::Delivery {
            stops = Some(get_ondc_select_fulfillment_end(&fulfillment.location));
            if trade_type == TradeType::Import {
                tags = get_fulfillment_tags(&fulfillment.delivery_terms);
                customer = Some(get_ondc_customer_detail(
                    business_account,
                    Some(&trade_type),
                ));
            };
        }

        fulfillment_objs.push(ONDCFulfillment {
            id: fulfillment.id.clone(),
            r#type: fulfillment.r#type.get_ondc_fulfillment_type(),
            tags,
            stops,
            customer,
        })
    }

    fulfillment_objs
}

#[tracing::instrument(name = "get ondc select message body", skip())]
fn get_ondc_select_message(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    order_request: &OrderSelectRequest,
    seller_location_mapping: &HashMap<String, ONDCSellerLocationInfo>,
    chat_data: &Option<ChatData>,
) -> Result<ONDCSelectMessage, SelectOrderError> {
    let location_ids: HashSet<&str> = order_request
        .items
        .iter()
        .flat_map(|item| item.location_ids.iter().map(|s| s.as_str()))
        .collect();
    let provider = get_ondc_select_order_provider(
        &location_ids,
        &order_request.provider_id,
        &order_request.ttl,
    );
    let select_tag = get_ondc_select_tags(business_account, chat_data)
        .map_err(|e| SelectOrderError::InvalidDataError(e.to_string()))?;
    Ok(ONDCSelectMessage {
        order: ONDCSelectOrder {
            provider,
            items: get_ondc_select_order_item(&order_request.order_type, &order_request.items),
            add_ons: None,
            tags: select_tag,
            payments: get_ondc_select_payment_obs(&order_request.payment_types),

            fulfillments: get_ondc_select_fulfillments(
                seller_location_mapping,
                &order_request.fulfillments,
                business_account,
            ),
        },
    })
}

#[tracing::instrument(name = "get ondc select payload", skip())]
pub fn get_ondc_select_payload(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    order_request: &OrderSelectRequest,
    bap_detail: &RegisteredNetworkParticipant,
    bpp_detail: &LookupData,
    seller_location_mapping: &HashMap<String, ONDCSellerLocationInfo>,
    chat_data: &Option<ChatData>,
) -> Result<ONDCSelectRequest, SelectOrderError> {
    let context = get_ondc_select_context(order_request, bap_detail, bpp_detail)?;
    let message = get_ondc_select_message(
        user_account,
        business_account,
        order_request,
        seller_location_mapping,
        chat_data,
    )?;
    Ok(ONDCSelectRequest { context, message })
}

fn get_ondc_seller_slab_from_ws_slab(ws_slabs: &Vec<WSPriceSlab>) -> Vec<ONDCSellePriceSlab> {
    let mut price_slabs = vec![];
    for ws_slab in ws_slabs {
        price_slabs.push(ONDCSellePriceSlab {
            min: ws_slab.min.clone(),
            max: ws_slab.max.clone(),
            price_with_tax: ws_slab.price_with_tax.clone(),
            price_without_tax: ws_slab.price_without_tax.clone(),
        })
    }
    price_slabs
}

#[tracing::instrument(name = "save ondc seller product info", skip())]
pub fn create_bulk_seller_product_info_objs<'a>(
    body: &'a WSSearchData,
    code: &'a CountryCode,
    updated_on: DateTime<Utc>,
) -> BulkSellerProductInfo<'a> {
    let mut seller_subscriber_ids: Vec<&str> = vec![];
    let mut provider_ids: Vec<&str> = vec![];
    let mut item_codes: Vec<Option<&str>> = vec![];
    let mut item_names: Vec<&str> = vec![];
    let mut item_ids: Vec<&str> = vec![];
    let mut tax_rates: Vec<BigDecimal> = vec![];
    let mut image_objs: Vec<Value> = vec![];
    let mut mrps: Vec<BigDecimal> = vec![];
    let mut unit_price_with_taxes: Vec<BigDecimal> = vec![];
    let mut unit_price_without_taxes: Vec<BigDecimal> = vec![];
    let mut currency_codes = vec![];
    let mut price_slabs = vec![];
    let mut country_codes = vec![];
    let mut updated_ons = vec![];
    let mut ids = vec![];
    for provider in &body.providers {
        for item in &provider.items {
            seller_subscriber_ids.push(&body.bpp.subscriber_id);
            provider_ids.push(&provider.description.id);
            item_ids.push(&item.id);
            item_codes.push(item.code.as_deref());
            item_names.push(&item.name);
            tax_rates.push(item.tax_rate.clone());
            mrps.push(item.price.maximum_price.clone());
            unit_price_with_taxes.push(item.price.price_with_tax.clone());
            unit_price_without_taxes.push(item.price.price_without_tax.clone());
            country_codes.push(code);
            updated_ons.push(updated_on);
            ids.push(Uuid::new_v4());
            // for image_url in item.images.iter() {
            image_objs.push(serde_json::to_value(&item.images).unwrap());
            currency_codes.push(&item.price.currency);
            if let Some(price_slab_obj) = item
                .price_slabs
                .as_ref()
                .map(get_ondc_seller_slab_from_ws_slab)
            {
                price_slabs.push(Some(serde_json::to_value(price_slab_obj).unwrap()));
            } else {
                price_slabs.push(None);
            }
        }
    }

    return BulkSellerProductInfo {
        ids,
        seller_subscriber_ids,
        provider_ids,
        item_codes,
        item_ids,
        item_names,
        tax_rates,
        image_objs,
        mrps,
        unit_price_with_taxes,
        unit_price_without_taxes,
        currency_codes,
        price_slabs,
        country_codes,
        updated_ons,
    };
}

#[tracing::instrument(name = "save ondc seller product info", skip(transaction, data))]
pub async fn save_ondc_seller_product_info<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    data: &WSSearchData,
    code: &CountryCode,
    updated_on: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let product_data = create_bulk_seller_product_info_objs(data, code, updated_on);
    let query = sqlx::query!(
        r#"
        INSERT INTO ondc_provider_product_info (
            seller_subscriber_id,
            provider_id,
            item_id,
            item_code,
            item_name,
            tax_rate,
            images,
            unit_price_with_tax,
            unit_price_without_tax,
            mrp,
            currency_code,
            price_slab,
            country_code,
            created_on,
            updated_on,
            id
        )
        SELECT *
        FROM UNNEST(
            $1::text[], 
            $2::text[], 
            $3::text[], 
            $4::text[], 
            $5::text[], 
            $6::decimal[],
            $7::jsonb[],
            $8::decimal[],
            $9::decimal[],
            $10::decimal[],
            $11::currency_code_type[],
            $12::jsonb[],
            $13::country_code_type[],
            $14::timestamptz[],
            $15::timestamptz[],
            $16::uuid[]
        )
        ON CONFLICT (seller_subscriber_id, country_code, provider_id, item_id) 
        DO UPDATE SET 
            item_name = EXCLUDED.item_name,
            tax_rate = EXCLUDED.tax_rate,
            images = EXCLUDED.images,
            unit_price_with_tax = EXCLUDED.unit_price_with_tax,
            unit_price_without_tax = EXCLUDED.unit_price_with_tax,
            mrp =  EXCLUDED.mrp,
            price_slab = EXCLUDED.price_slab,
            updated_on = EXCLUDED.updated_on;
            
        "#,
        &product_data.seller_subscriber_ids[..] as &[&str],
        &product_data.provider_ids[..] as &[&str],
        &product_data.item_ids[..] as &[&str],
        &product_data.item_codes[..] as &[Option<&str>],
        &product_data.item_names[..] as &[&str],
        &product_data.tax_rates[..] as &[BigDecimal],
        &product_data.image_objs[..],
        &product_data.unit_price_with_taxes[..] as &[BigDecimal],
        &product_data.unit_price_without_taxes[..] as &[BigDecimal],
        &product_data.mrps[..] as &[BigDecimal],
        &product_data.currency_codes[..] as &[&CurrencyType],
        &product_data.price_slabs[..] as &[Option<Value>],
        &product_data.country_codes[..] as &[&CountryCode],
        &product_data.updated_ons[..] as &[DateTime<Utc>],
        &product_data.updated_ons[..] as &[DateTime<Utc>],
        &product_data.ids[..] as &[Uuid],
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving ONDC seller product info")
    })?;

    Ok(())
}

pub async fn fetch_ondc_seller_product_info(
    pool: &PgPool,
    bpp_id: &str,
    provider_id: &str,
    item_id_list: &Vec<&str>,
    country_code: &CountryCode,
) -> Result<Vec<ONDCSellerProductInfo>, anyhow::Error> {
    let row: Vec<ONDCSellerProductInfo> = sqlx::query_as!(
        ONDCSellerProductInfo,
        r#"SELECT item_name, currency_code  as "currency_code: CurrencyType", item_id, item_code, seller_subscriber_id,
        price_slab as "price_slab?: Json<Vec<ONDCSellePriceSlab>>", provider_id, tax_rate, 
        unit_price_with_tax,unit_price_without_tax, mrp, images from ondc_provider_product_info where 
        provider_id  = $1 AND seller_subscriber_id=$2 AND item_id::text = ANY($3) AND country_code =$4"#,
        provider_id,
        bpp_id,
        item_id_list as &Vec<&str>,
        country_code as &CountryCode,
    )
    .fetch_all(pool)
    .await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while fetching ondc seller product info")
    })?;
    Ok(row)
}
/// Key for for the seller mapping key
pub fn get_ondc_seller_product_mapping_key(
    bpp_id: &str,
    provider_id: &str,
    item_code: &str,
) -> String {
    format!("{}_{}_{}", bpp_id, provider_id, item_code)
}

#[tracing::instrument(name = "fetch ondc seller product info mapping", skip(pool))]
pub async fn get_ondc_seller_product_info_mapping(
    pool: &PgPool,
    bpp_id: &str,
    provider_id: &str,
    item_id_list: &Vec<&str>,
    country_code: &CountryCode,
) -> Result<HashMap<String, ONDCSellerProductInfo>, anyhow::Error> {
    let seller_product_info =
        fetch_ondc_seller_product_info(pool, bpp_id, provider_id, item_id_list, country_code)
            .await?;
    let seller_product_map: HashMap<String, ONDCSellerProductInfo> = seller_product_info
        .into_iter()
        .map(|obj| {
            (
                get_ondc_seller_product_mapping_key(
                    &obj.seller_subscriber_id,
                    &obj.provider_id,
                    &obj.item_id,
                ),
                obj,
            )
        })
        .collect();
    Ok(seller_product_map)
}

#[tracing::instrument(name = "Fetch ONDC Order request", skip(pool))]
pub async fn fetch_ondc_order_request(
    pool: &PgPool,
    transaction_id: Uuid,
    message_id: Uuid,
    action_type: &ONDCActionType,
) -> Result<Option<ONDCRequestModel>, anyhow::Error> {
    let row = sqlx::query_as!(
        ONDCRequestModel,
        r#"SELECT transaction_id, message_id, user_id, business_id, device_id, request_payload
        FROM ondc_buyer_order_req
        WHERE transaction_id = $1 AND message_id = $2 AND action_type = $3 ORDER BY created_on DESC
        "#,
        transaction_id,
        message_id,
        &action_type.to_string() as &str
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

// #[tracing::instrument(name = "Fetch order request params", skip(pool))]
// pub async fn fetch_order_params(
//     pool: &PgPool,
//     transaction_id: Uuid,
//     message_id: Uuid,
//     action_type: &ONDCActionType,
// ) -> Result<Option<OrderRequestParamsModel>, anyhow::Error> {
//     let row = sqlx::query_as!(
//         OrderRequestParamsModel,
//         r#"SELECT transaction_id, message_id, user_id, business_id, device_id, commerce_type as "commerce_type: OrderType"
//         FROM ondc_buyer_order_req
//         WHERE transaction_id = $1 AND message_id = $2 AND action_type = $3 ORDER BY created_on DESC
//         "#,
//         transaction_id,
//         message_id,
//         &action_type.to_string() as &str
//     )
//     .fetch_optional(pool)
//     .await?;

//     Ok(row)
// }

#[tracing::instrument(name = "get init context", skip())]
fn get_ondc_context_from_order(
    tranaction_id: Uuid,
    message_id: Uuid,
    order: &Commerce,
    action_type: ONDCActionType,
) -> Result<ONDCContext, anyhow::Error> {
    get_common_context(
        tranaction_id,
        message_id,
        &order.domain_category_code,
        action_type,
        &order.bap.id,
        &order.bap.uri,
        Some(&order.bpp.id),
        Some(&order.bpp.uri),
        &order.country_code,
        &order.city_code,
        Some(ONDC_TTL),
    )
}

fn get_ondc_billing_from_init_billing(billing: &OrderInitBilling) -> ONDCBilling {
    ONDCBilling {
        name: billing.name.clone(),
        address: billing.address.clone(),
        state: ONDCState {
            name: billing.state.clone(),
        },
        city: ONDCCity {
            name: billing.city.name.clone(),
        },
        tax_id: billing.tax_id.clone(),
        email: Some(EmailObject::new(billing.email.clone())),
        phone: billing.mobile_no.clone(),
    }
}

fn get_ondc_billing_from_order_billing(billing: &CommerceBilling) -> ONDCBilling {
    ONDCBilling {
        name: billing.name.clone(),
        address: billing.address.clone(),
        state: ONDCState {
            name: billing.state.clone(),
        },
        city: ONDCCity {
            name: billing.city.clone(),
        },
        tax_id: billing.tax_id.clone(),
        email: billing.email.clone(),
        phone: billing.phone.clone(),
    }
}

fn get_ondc_payment_from_order(payments: &Vec<CommercePayment>) -> Vec<ONDCInitPayment> {
    let mut payment_list = vec![];
    for payment in payments {
        payment_list.push(ONDCInitPayment {
            r#type: payment.payment_type.get_ondc_payment(),
            collected_by: payment
                .collected_by
                .clone()
                .unwrap_or(PaymentCollectedBy::Bpp)
                .get_ondc_type(),
        })
    }
    payment_list
}

#[tracing::instrument(name = "get ondc init items", skip())]
fn get_ondc_items_from_order(items: &Vec<CommerceItem>) -> Vec<ONDCSelectedItem> {
    let mut ondc_item = vec![];

    for item in items {
        ondc_item.push(ONDCSelectedItem {
            id: item.item_id.clone(),
            location_ids: item.location_ids.clone(),
            fulfillment_ids: item.fulfillment_ids.clone(),
            quantity: ONDCQuantitySelect {
                selected: ONDCQuantityCountInt {
                    count: item.qty.to_i32().unwrap_or_default(),
                },
            },
            tags: item.buyer_terms.as_ref().map(|e| {
                vec![ONDCTag::get_item_tags(
                    e.item_req.as_str(),
                    e.packaging_req.as_str(),
                )]
            }),
            payment_ids: None,
        })
    }
    ondc_item
}

fn get_ondc_init_fulfillment_stops(
    fulfillment_type: &FulfillmentType,
    drop_off: &Option<DropOffData>,
    pickup: &PickUpData,
) -> Vec<ONDCOrderFulfillmentEnd> {
    let mut fulfillment_ends = vec![];
    if let Some(drop_off) = drop_off {
        fulfillment_ends.push(ONDCOrderFulfillmentEnd {
            r#type: ONDCFulfillmentStopType::End,
            contact: ONDCContact {
                email: drop_off.contact.email.clone(),
                phone: drop_off.contact.mobile_no.clone(),
            },
            location: ONDCSelectFulfillmentLocation {
                gps: drop_off.location.gps.clone(),
                address: drop_off.location.address.clone(),
                area_code: drop_off.location.area_code.clone(),
                city: ONDCCity {
                    name: drop_off.location.city.clone(),
                },
                country: ONDCCountry {
                    code: drop_off.location.country.clone(),
                },
                state: ONDCState {
                    name: drop_off.location.state.clone(),
                },
            },
        });
    }
    if fulfillment_type == &FulfillmentType::SelfPickup {
        // if let Some(pickup) = pickup {
        fulfillment_ends.push(ONDCOrderFulfillmentEnd {
            r#type: ONDCFulfillmentStopType::Start,
            contact: ONDCContact {
                email: pickup.contact.email.clone(),
                phone: pickup.contact.mobile_no.clone(),
            },
            location: ONDCSelectFulfillmentLocation {
                gps: pickup.location.gps.clone(),
                address: Some(pickup.location.address.clone()),
                area_code: pickup.location.area_code.clone(),
                city: ONDCCity {
                    name: pickup.location.city.clone(),
                },
                country: ONDCCountry {
                    code: pickup.location.country.clone(),
                },
                state: ONDCState {
                    name: pickup.location.state.clone(),
                },
            },
        });
        // }
    }

    fulfillment_ends
}

#[tracing::instrument(name = "get ondc init fulfillment", skip())]
fn get_get_ondc_init_fulfillment(
    fulfillments: &Vec<CommerceFulfillment>,
    business_account: &BusinessAccount,
) -> Vec<ONDCFulfillment> {
    fulfillments
        .iter()
        .map(|fulfillment| {
            let tags_result = fulfillment.delivery_term.as_ref().map(|delivery_term| {
                vec![ONDCTag::get_delivery_terms(
                    &delivery_term.inco_terms,
                    &delivery_term.place_of_delivery,
                )]
            });

            ONDCFulfillment {
                id: fulfillment.fulfillment_id.clone(),
                r#type: fulfillment.fulfillment_type.get_ondc_fulfillment_type(),
                tags: tags_result,
                customer: Some(get_ondc_customer_detail(
                    business_account,
                    fulfillment.trade_type.as_ref(),
                )),
                stops: Some(get_ondc_init_fulfillment_stops(
                    &fulfillment.fulfillment_type,
                    &fulfillment.drop_off,
                    &fulfillment.pickup,
                )),
            }
        })
        .collect()
}

#[tracing::instrument(name = "get ondc init message body", skip())]
fn get_ondc_init_message(
    business_account: &BusinessAccount,
    init_request: &OrderInitRequest,
    order: &Commerce,
) -> Result<ONDCInitMessage, InitOrderError> {
    let location_ids = order.get_ondc_location_ids();
    Ok(ONDCInitMessage {
        order: ONDCInitOrder {
            provider: ONDCInitProvider {
                id: order.seller.id.clone(),
                locations: location_ids
                    .iter()
                    .map(|e| ONDCLocationId { id: e.to_string() })
                    .collect(),
            },
            billing: get_ondc_billing_from_init_billing(&init_request.billing),
            add_ons: None,
            payments: get_ondc_payment_from_order(&order.payments),
            items: get_ondc_items_from_order(&order.items),

            tags: vec![get_buyer_id_tag(business_account)?],
            fulfillments: get_get_ondc_init_fulfillment(&order.fulfillments, business_account),
        },
    })
}

#[tracing::instrument(name = "get ondc init payload", skip())]
pub fn get_ondc_init_payload(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    order: &Commerce,
    init_request: &OrderInitRequest,
) -> Result<ONDCInitRequest, InitOrderError> {
    let context = get_ondc_context_from_order(
        init_request.transaction_id,
        init_request.message_id,
        order,
        ONDCActionType::Init,
    )?;
    let message = get_ondc_init_message(business_account, init_request, order)?;
    Ok(ONDCInitRequest { context, message })
}

pub fn get_ondc_cancel_fee_from_cancel_fee(
    currency: &CurrencyType,
    fee: &CommerceCancellationFee,
) -> ONDCOrderCancellationFee {
    match fee.r#type {
        CancellationFeeType::Percent => ONDCOrderCancellationFee::Percent {
            percentage: fee.val.to_string(),
        },
        CancellationFeeType::Amount => ONDCOrderCancellationFee::Amount {
            amount: ONDCAmount {
                currency: currency.clone(),
                value: fee.val.to_string(),
            },
        },
    }
}

pub fn get_ondc_cancellation_from_cancelletion_terms(
    currency_type: &CurrencyType,
    cancellation_terms: &Vec<CommerceCancellationTerm>,
) -> Vec<ONDCOrderCancellationTerm> {
    let mut ondc_cancel_objs = vec![];
    for cancellation_term in cancellation_terms {
        ondc_cancel_objs.push(ONDCOrderCancellationTerm {
            fulfillment_state: ONDCFulfillmentState {
                descriptor: ONDCFulfillmentDescriptor {
                    code: cancellation_term
                        .fulfillment_state
                        .get_ondc_fulfillment_state(),
                },
            },

            reason_required: cancellation_term.reason_required,
            cancellation_fee: get_ondc_cancel_fee_from_cancel_fee(
                currency_type,
                &cancellation_term.cancellation_fee,
            ),
        })
    }
    ondc_cancel_objs
}

pub fn get_tag_value_from_list<'a>(
    tags: &'a [ONDCTag],
    tag_type: ONDCTagType,
    item_code: &str,
) -> Option<&'a str> {
    let val = tags
        .iter()
        .filter(|tag| tag.descriptor.code == tag_type)
        .flat_map(|tag| tag.get_tag_value(item_code))
        .next();
    val
}

fn get_ondc_confirm_request_payment(
    order: &Commerce,
    bap_detail: &RegisteredNetworkParticipant,
) -> Vec<ONDCOnConfirmPayment> {
    let mut payment_objs = vec![];
    let currency_type = order.currency_type.as_ref().unwrap_or(&CurrencyType::Inr);
    for payment in &order.payments {
        let mut settlement_detail_objs = vec![];
        if payment.collected_by == Some(PaymentCollectedBy::Bpp) {
            settlement_detail_objs.push(ONDCPaymentSettlementDetail {
                settlement_counterparty: ONDCPaymentSettlementCounterparty::BuyerApp,
                settlement_phase: bap_detail.settlement_phase.get_ondc_settlement_phase(),
                settlement_type: bap_detail.settlement_type.get_ondc_settlement_type(),
                settlement_bank_account_no: bap_detail.bank_account_no.to_owned(),
                settlement_ifsc_code: bap_detail.bank_ifsc_code.to_owned(),
                beneficiary_name: bap_detail.bank_beneficiary_name.to_owned(),
                bank_name: bap_detail.bank_name.to_owned(),
            });
        } else if let Some(settlement_details) = &payment.settlement_details {
            for settlement in settlement_details {
                settlement_detail_objs.push(ONDCPaymentSettlementDetail {
                    settlement_counterparty: settlement
                        .settlement_counterparty
                        .get_ondc_settlement_counterparty(),
                    settlement_phase: settlement.settlement_phase.get_ondc_settlement_phase(),
                    settlement_type: settlement.settlement_type.get_ondc_settlement_type(),
                    settlement_bank_account_no: settlement.settlement_bank_account_no.clone(),
                    settlement_ifsc_code: settlement.settlement_ifsc_code.clone(),
                    beneficiary_name: settlement.beneficiary_name.clone(),
                    bank_name: settlement.bank_name.clone(),
                });
            }
        }

        payment_objs.push(ONDCOnConfirmPayment {
            id: None,
            r#type: payment.payment_type.get_ondc_payment(),
            collected_by: payment
                .collected_by
                .clone()
                .unwrap_or(PaymentCollectedBy::Bpp)
                .get_ondc_type(),
            uri: None,
            tags: None,
            params: ONDCPaymentParams {
                amount: order.grand_total.clone().unwrap_or_default().to_string(),
                currency: currency_type.clone(),
                transaction_id: order
                    .payments
                    .iter()
                    .find(|p| p.payment_id.is_some())
                    .and_then(|e| e.payment_id.to_owned()),
            },
            buyer_app_finder_fee_type: payment.buyer_fee_type.clone().unwrap_or(FeeType::Amount),
            buyer_app_finder_fee_amount: payment
                .buyer_fee_amount
                .clone()
                .unwrap_or("0.00".to_owned()),
            settlement_basis: payment
                .settlement_basis
                .clone()
                .unwrap_or(SettlementBasis::Delivery)
                .get_ondc_settlement_basis(),
            settlement_window: payment
                .settlement_window
                .clone()
                .unwrap_or("P1D".to_owned()),
            withholding_amount: payment
                .withholding_amount
                .clone()
                .unwrap_or("0.0".to_owned()),
            settlement_details: Some(settlement_detail_objs),
            status: ONDCPaymentStatus::NotPaid,
        })
    }
    payment_objs
}

fn get_item_breakup(currency_type: &CurrencyType, items: &Vec<CommerceItem>) -> Vec<ONDCBreakUp> {
    let mut break_up_list = vec![];
    for line in items {
        break_up_list.push(ONDCBreakUp::create(
            line.item_name.clone(),
            line.item_id.clone(),
            BreakupTitleType::Item,
            ONDCAmount {
                currency: currency_type.clone(),
                value: line.gross_total.to_string(),
            },
            Some(ONDCOrderItemQuantity {
                count: line.qty.to_i32().unwrap_or_default(),
            }),
            Some(ONDCBreakupItemInfo {
                price: ONDCAmount {
                    currency: currency_type.clone(),
                    value: line.unit_price.to_string(),
                },
            }),
        ));
        break_up_list.push(ONDCBreakUp::create(
            "Tax".to_owned(),
            line.item_id.clone(),
            BreakupTitleType::Tax,
            ONDCAmount {
                currency: currency_type.clone(),
                value: line.tax_value.to_string(),
            },
            None,
            None,
        ));
        break_up_list.push(ONDCBreakUp::create(
            "Discount".to_owned(),
            line.item_id.clone(),
            BreakupTitleType::Discount,
            ONDCAmount {
                currency: currency_type.clone(),
                value: line.discount_amount.to_string(),
            },
            None,
            None,
        ));
    }
    break_up_list
}

fn get_fulfillment_breakup(
    currency_type: &CurrencyType,
    fulfillments: &Vec<CommerceFulfillment>,
) -> Vec<ONDCBreakUp> {
    let mut break_up_list = vec![];
    for fulfillment in fulfillments {
        break_up_list.push(ONDCBreakUp::create(
            "Packing".to_owned(),
            fulfillment.fulfillment_id.clone(),
            BreakupTitleType::Packing,
            ONDCAmount {
                currency: currency_type.clone(),
                value: fulfillment.packaging_charge.to_string(),
            },
            None,
            None,
        ));
        break_up_list.push(ONDCBreakUp::create(
            "Delivery Charge".to_owned(),
            fulfillment.fulfillment_id.clone(),
            BreakupTitleType::Delivery,
            ONDCAmount {
                currency: currency_type.clone(),
                value: fulfillment.delivery_charge.to_string(),
            },
            None,
            None,
        ));
        break_up_list.push(ONDCBreakUp::create(
            "Convenience Fee".to_owned(),
            fulfillment.fulfillment_id.clone(),
            BreakupTitleType::Misc,
            ONDCAmount {
                currency: currency_type.clone(),
                value: fulfillment.convenience_fee.to_string(),
            },
            None,
            None,
        ));
    }
    break_up_list
}

fn get_quote_from_order(order: &Commerce) -> ONDCQuote {
    let currency_type = order.currency_type.as_ref().unwrap_or(&CurrencyType::Inr);
    let mut breakup = get_fulfillment_breakup(currency_type, &order.fulfillments);
    breakup.extend(get_item_breakup(currency_type, &order.items));
    ONDCQuote {
        ttl: order.quote_ttl.clone(),
        price: ONDCAmount {
            currency: order.currency_type.clone().unwrap_or(CurrencyType::Inr),
            value: order.grand_total.clone().unwrap_or_default().to_string(),
        },

        breakup,
    }
}

fn get_ondc_confirm_request_tags(
    order: &Commerce,
    business_account: &BusinessAccount,
) -> Result<Vec<ONDCTag>, anyhow::Error> {
    let mut confirm_tags = vec![];
    match get_buyer_id_tag(business_account) {
        Ok(tag_option) => confirm_tags.push(tag_option),
        Err(e) => return Err(e),
    }
    if let Some(bpp_terms) = &order.bpp_terms {
        confirm_tags.push(ONDCTag::get_bpp_terms_tag(bpp_terms));
        confirm_tags.push(ONDCTag::get_bap_agreement_to_bpp_terms_tag("Y"));
    }

    Ok(confirm_tags)
}

#[tracing::instrument(name = "get ondc confirm message body", skip())]
fn get_ondc_confirm_message(
    business_account: &BusinessAccount,
    order: &Commerce,
    updated_on: &DateTime<Utc>,
    bap_detail: &RegisteredNetworkParticipant,
) -> Result<ONDCConfirmMessage, ConfirmOrderError> {
    let location_ids = order.get_ondc_location_ids();
    let billing = order.billing.as_ref().ok_or_else(|| {
        ConfirmOrderError::ValidationError("Billing Address not found".to_string())
    })?;
    Ok(ONDCConfirmMessage {
        order: ONDCConfirmOrder {
            id: order.urn.to_owned(),
            state: ONDCOrderStatus::Created,
            provider: ONDCConfirmProvider {
                id: order.seller.id.clone(),
                locations: location_ids
                    .iter()
                    .map(|e| ONDCLocationId { id: e.to_string() })
                    .collect(),
            },
            items: get_ondc_items_from_order(&order.items),
            fulfillments: get_get_ondc_init_fulfillment(&order.fulfillments, business_account),
            billing: get_ondc_billing_from_order_billing(billing),
            cancellation_terms: get_ondc_cancellation_from_cancelletion_terms(
                order.currency_type.as_ref().unwrap_or(&CurrencyType::Inr),
                order.cancellation_terms.as_ref().unwrap(),
            ),
            created_at: order.created_on,
            updated_at: *updated_on,
            tags: get_ondc_confirm_request_tags(order, business_account)
                .map_err(|e| ConfirmOrderError::InvalidDataError(e.to_string()))?,
            quote: get_quote_from_order(order),
            payments: get_ondc_confirm_request_payment(order, bap_detail),
        },
    })
}

#[tracing::instrument(name = "get confirm context", skip())]
fn get_ondc_confirm_context(
    tranaction_id: Uuid,
    message_id: Uuid,
    order: &Commerce,
) -> Result<ONDCContext, anyhow::Error> {
    get_common_context(
        tranaction_id,
        message_id,
        &order.domain_category_code,
        ONDCActionType::Init,
        &order.bap.id,
        &order.bap.uri,
        Some(&order.bpp.id),
        Some(&order.bpp.uri),
        &order.country_code,
        &order.city_code,
        Some(ONDC_TTL),
    )
}

#[tracing::instrument(name = "get ondc confirm payload", skip())]
pub fn get_ondc_confirm_payload(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    order: &Commerce,
    confirm_request: &OrderConfirmRequest,
    bap_detail: &RegisteredNetworkParticipant,
) -> Result<ONDConfirmRequest, ConfirmOrderError> {
    let context = get_ondc_context_from_order(
        confirm_request.transaction_id,
        confirm_request.message_id,
        order,
        ONDCActionType::Confirm,
    )?;
    let message =
        get_ondc_confirm_message(business_account, order, &context.timestamp, bap_detail)?;
    Ok(ONDConfirmRequest { context, message })
}

#[tracing::instrument(name = "save ondc seller location info", skip())]
pub fn create_bulk_seller_location_info_objs<'a>(
    body: &'a WSSearchData,
    updated_on: DateTime<Utc>,
) -> BulkSellerLocationInfo<'a> {
    let mut seller_subscriber_ids: Vec<&str> = vec![];
    let mut provider_ids = vec![];
    let mut location_ids: Vec<&str> = vec![];
    let mut latitudes = vec![];
    let mut longitudes = vec![];
    let mut addresses = vec![];
    let mut city_codes = vec![];
    let mut city_names = vec![];
    let mut state_codes = vec![];
    let mut state_names = vec![];
    let mut country_names = vec![];
    let mut country_codes = vec![];
    let mut area_codes = vec![];
    let mut updated_ons = vec![];
    let mut ids = vec![];
    for provider in &body.providers {
        for (key, location) in &provider.locations {
            let gps_data = location
                .gps
                .split(',')
                .map(|s| BigDecimal::from_str(s).unwrap_or_else(|_| BigDecimal::from(0).clone()))
                .collect::<Vec<_>>();

            seller_subscriber_ids.push(&body.bpp.subscriber_id);
            provider_ids.push(provider.description.id.as_str());
            location_ids.push(key.as_str());
            latitudes.push(gps_data.first().cloned().unwrap_or(BigDecimal::from(0)));
            longitudes.push(gps_data.get(1).cloned().unwrap_or(BigDecimal::from(0)));
            addresses.push(location.address.as_str());
            city_codes.push(location.city.code.as_str());
            city_names.push(location.city.name.as_str());
            state_codes.push(location.state.code.as_str());
            state_names.push(location.state.name.as_deref());
            country_names.push(location.country.name.as_deref());
            country_codes.push(&location.country.code);
            area_codes.push(location.area_code.as_str());
            updated_ons.push(updated_on);
            ids.push(Uuid::new_v4());
        }
    }

    return BulkSellerLocationInfo {
        seller_subscriber_ids,
        provider_ids,
        location_ids,
        longitudes,
        latitudes,
        addresses,
        city_codes,
        city_names,
        state_codes,
        state_names,
        country_names,
        country_codes,
        area_codes,
        updated_ons,
        ids,
    };
}

#[tracing::instrument(name = "save ondc seller location info", skip(transaction, data))]
pub async fn save_ondc_seller_location_info<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    data: &'a WSSearchData,
    updated_on: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let seller_data = create_bulk_seller_location_info_objs(data, updated_on);
    let query = sqlx::query!(
        r#"
        INSERT INTO ondc_provider_location_info (
            seller_subscriber_id,
            provider_id,
            location_id,
            latitude,
            longitude,
            address,
            city_code,
            city_name,
            state_code,
            state_name,
            country_code,
            country_name,
            area_code,
            created_on,
            updated_on,
            id
        )
        SELECT *
        FROM UNNEST(
            $1::text[], 
            $2::text[], 
            $3::text[], 
            $4::decimal[], 
            $5::decimal[], 
            $6::text[], 
            $7::text[],
            $8::text[],
            $9::text[],
            $10::text[],
            $11::country_code_type[],
            $12::text[],
            $13::text[],
            $14::timestamptz[],
            $15::timestamptz[],
            $16::uuid[]
        )
        ON CONFLICT (seller_subscriber_id, provider_id, location_id) 
        DO UPDATE SET 
            latitude = EXCLUDED.latitude,
            longitude = EXCLUDED.longitude,
            address = EXCLUDED.address,
            city_code = EXCLUDED.city_code,
            city_name =  EXCLUDED.city_name,
            state_code =  EXCLUDED.state_code,
            state_name =  EXCLUDED.state_name,
            country_code =  EXCLUDED.country_code,
            country_name =  EXCLUDED.country_name,
            updated_on = EXCLUDED.updated_on,
            area_code = EXCLUDED.area_code
        "#,
        &seller_data.seller_subscriber_ids[..] as &[&str],
        &seller_data.provider_ids[..] as &[&str],
        &seller_data.location_ids[..] as &[&str],
        &seller_data.latitudes[..] as &[BigDecimal],
        &seller_data.longitudes[..] as &[BigDecimal],
        &seller_data.addresses[..] as &[&str],
        &seller_data.city_codes[..] as &[&str],
        &seller_data.city_names[..] as &[&str],
        &seller_data.state_codes[..] as &[&str],
        &seller_data.state_names[..] as &[Option<&str>],
        &seller_data.country_codes[..] as &[&CountryCode],
        &seller_data.country_names[..] as &[Option<&str>],
        &seller_data.area_codes[..] as &[&str],
        &seller_data.updated_ons[..] as &[DateTime<Utc>],
        &seller_data.updated_ons[..] as &[DateTime<Utc>],
        &seller_data.ids[..] as &[Uuid],
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving ONDC seller location info")
    })?;

    Ok(())
}

pub fn get_ondc_seller_location_mapping_key(
    bpp_id: &str,
    provider_id: &str,
    location_id: &str,
) -> String {
    format!("{}_{}_{}", bpp_id, provider_id, location_id)
}

#[tracing::instrument(name = "fetch fetch_ondc_seller_location_info", skip(pool))]
pub async fn fetch_ondc_seller_location_info(
    pool: &PgPool,
    bpp_id: &str,
    provider_id: &str,
    location_id_list: &Vec<String>,
) -> Result<Vec<ONDCSellerLocationInfo>, anyhow::Error> {
    let row: Vec<ONDCSellerLocationInfo> = sqlx::query_as!(
        ONDCSellerLocationInfo,
        r#"SELECT location_id, seller_subscriber_id, provider_id, latitude, longitude,
        address, city_code, city_name, state_code, state_name, country_code  as "country_code:CountryCode", area_code,
        country_name from ondc_provider_location_info where 
        provider_id  = $1 AND seller_subscriber_id=$2 AND location_id::text = ANY($3)"#,
        provider_id,
        bpp_id,
        location_id_list as &Vec<String>
    )
    .fetch_all(pool)
    .await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("failed to fetch ondc seller location info data from database")
    })?;
    Ok(row)
}

#[tracing::instrument(name = "fetch ondc seller product info mapping", skip(pool))]
pub async fn get_ondc_seller_location_info_mapping(
    pool: &PgPool,
    bpp_id: &str,
    provider_id: &str,
    location_id_list: &Vec<String>,
) -> Result<HashMap<String, ONDCSellerLocationInfo>, anyhow::Error> {
    let seller_product_info =
        fetch_ondc_seller_location_info(pool, bpp_id, provider_id, location_id_list).await?;
    let seller_product_map: HashMap<String, ONDCSellerLocationInfo> = seller_product_info
        .into_iter()
        .map(|obj| {
            (
                get_ondc_seller_location_mapping_key(
                    &obj.seller_subscriber_id,
                    &obj.provider_id,
                    &obj.location_id,
                ),
                obj,
            )
        })
        .collect();
    Ok(seller_product_map)
}

#[tracing::instrument(name = "save ondc seller info", skip())]
pub fn create_bulk_seller_info_objs<'a>(
    body: &'a WSSearchData,
    updated_on: DateTime<Utc>,
) -> BulkSellerInfo<'a> {
    let mut seller_subscriber_ids: Vec<&str> = vec![];
    let mut provider_ids = vec![];
    let mut provider_names = vec![];
    let mut updated_ons = vec![];
    let mut ids = vec![];
    for provider in &body.providers {
        seller_subscriber_ids.push(&body.bpp.subscriber_id);
        provider_ids.push(provider.description.id.as_str());
        provider_names.push(provider.description.name.as_str());
        updated_ons.push(updated_on);
        ids.push(Uuid::new_v4());
    }

    return BulkSellerInfo {
        seller_subscriber_ids,
        provider_ids,
        provider_names,
        updated_ons,
        ids,
    };
}

#[tracing::instrument(name = "save ondc seller info", skip(transaction, data))]
pub async fn save_ondc_seller_info<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    data: &'a WSSearchData,
    updated_on: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let seller_data = create_bulk_seller_info_objs(data, updated_on);
    let query = sqlx::query!(
        r#"
        INSERT INTO ondc_provider_info (
            seller_subscriber_id,
            provider_id,
            provider_name,
            updated_on,
            created_on,
            id
        )
        SELECT *
        FROM UNNEST(
            $1::text[], 
            $2::text[], 
            $3::text[],
            $4::timestamptz[],
            $5::timestamptz[],
            $6::uuid[]
        )
        ON CONFLICT (seller_subscriber_id, provider_id) 
        DO UPDATE SET 
            provider_name = EXCLUDED.provider_name,
            updated_on = EXCLUDED.updated_on
        "#,
        &seller_data.seller_subscriber_ids[..] as &[&str],
        &seller_data.provider_ids[..] as &[&str],
        &seller_data.provider_names[..] as &[&str],
        &seller_data.updated_ons[..] as &[DateTime<Utc>],
        &seller_data.updated_ons[..] as &[DateTime<Utc>],
        &seller_data.ids[..] as &[Uuid],
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving ONDC seller info")
    })?;

    Ok(())
}

pub async fn fetch_ondc_seller_info(
    pool: &PgPool,
    bpp_id: &str,
    provider_id: &str,
) -> Result<ONDCSellerInfo, anyhow::Error> {
    let row: ONDCSellerInfo = sqlx::query_as!(
        ONDCSellerInfo,
        r#"SELECT  seller_subscriber_id, provider_id, provider_name from ondc_provider_info where 
        provider_id  = $1 AND seller_subscriber_id=$2"#,
        provider_id,
        bpp_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

fn get_ondc_status_message(commerce_id: &str) -> ONDCStatusMessage {
    ONDCStatusMessage {
        order_id: commerce_id.to_owned(),
    }
}
#[tracing::instrument(name = "get ondc status payload", skip())]
pub fn get_ondc_status_payload(
    order: &Commerce,
    status_request: &OrderStatusRequest,
) -> Result<ONDCStatusRequest, OrderStatusError> {
    let context = get_ondc_context_from_order(
        status_request.transaction_id,
        status_request.message_id,
        order,
        ONDCActionType::Status,
    )?;

    let message = get_ondc_status_message(&order.urn);
    Ok(ONDCStatusRequest { context, message })
}

fn get_ondc_cancel_message(commerce_id: &str, reason_id: &str) -> ONDCCancelMessage {
    ONDCCancelMessage {
        order_id: commerce_id.to_owned(),
        cancellation_reason_id: reason_id.to_owned(),
    }
}

#[tracing::instrument(name = "get ondc cancel payload", skip())]
pub fn get_ondc_cancel_payload(
    order: &Commerce,
    cancel_request: &OrderCancelRequest,
) -> Result<ONDCCancelRequest, OrderCancelError> {
    let context = get_ondc_context_from_order(
        cancel_request.transaction_id,
        cancel_request.message_id,
        order,
        ONDCActionType::Cancel,
    )?;

    let message = get_ondc_cancel_message(&order.urn, &cancel_request.reason_id);
    Ok(ONDCCancelRequest { context, message })
}

fn get_ondc_update_items(order: &Commerce) -> Vec<ONDCUpdateItem> {
    let mut items_obj = vec![];
    for item in &order.items {
        items_obj.push(ONDCUpdateItem {
            id: item.item_id.clone(),
            quantity: ONDCQuantitySelect {
                selected: ONDCQuantityCountInt {
                    count: item.qty.with_scale(0).to_i32().unwrap_or(0),
                },
            },
        })
    }
    items_obj
}

fn get_ondc_update_message_for_payment(
    order: &Commerce,
    body: &UpdateOrderPaymentRequest,
    bap_detail: &RegisteredNetworkParticipant,
) -> ONDCUpdateMessage {
    ONDCUpdateMessage {
        update_target: body.target_type.get_ondc_type(),
        order: ONDCUpdateOrder {
            id: order.urn.clone(),
            state: order.record_status.get_ondc_order_status(),
            provider: ONDCUpdateProvider {
                id: order.seller.id.clone(),
            },
            payments: get_ondc_confirm_request_payment(order, bap_detail),
            items: get_ondc_update_items(order),
        },
    }
}

#[tracing::instrument(name = "get ondc update payload", skip())]
pub fn get_ondc_update_payload(
    order: &Commerce,
    update_request: &OrderUpdateRequest,
    bap_detail: &RegisteredNetworkParticipant,
) -> Result<ONDCUpdateRequest, OrderUpdateError> {
    let context = get_ondc_context_from_order(
        update_request.transaction_id(),
        update_request.message_id(),
        order,
        ONDCActionType::Update,
    )?;

    let message = match update_request {
        OrderUpdateRequest::UpdatePayment(body) => {
            get_ondc_update_message_for_payment(order, body, bap_detail)
        }
        OrderUpdateRequest::UpdateItem(_) => Err(OrderUpdateError::NotImplemented(
            "Item Updation not implemented".to_string(),
        ))?,
        OrderUpdateRequest::UpdateFulfillment(_) => Err(OrderUpdateError::NotImplemented(
            "Fulfillment Updation not implemented".to_string(),
        ))?,
    };

    Ok(ONDCUpdateRequest { context, message })
}

pub async fn process_on_search(
    pool: &PgPool,
    body: ONDCOnSearchRequest,
    extracted_search_obj: SearchRequestModel,
    websocket_srv: &WebSocketClient,
    elastic_search_client: &ElasticSearchClient,
) -> Result<(), anyhow::Error> {
    let final_objs: Option<WSSearchData> =
        get_product_from_on_search_request(&body).map_err(|op| anyhow!("error:{}", op))?;

    if let Some(final_objs) = final_objs {
        if !final_objs.providers.is_empty() {
            let mut transaction = pool
                .begin()
                .await
                .context("Failed to acquire a Postgres connection from the pool")?;

            let _ = save_ondc_seller_info(&mut transaction, &final_objs, body.context.timestamp)
                .await
                .map_err(|e| anyhow!(e));
            let _ = save_ondc_seller_product_info(
                &mut transaction,
                &final_objs,
                &body.context.location.country.code,
                body.context.timestamp,
            )
            .await
            .map_err(|e| anyhow!(e));

            let _ = save_ondc_seller_location_info(
                &mut transaction,
                &final_objs,
                body.context.timestamp,
            )
            .await
            .map_err(|e| anyhow!(e));

            if !extracted_search_obj.update_cache {
                let ws_params = get_websocket_params_from_search_req(extracted_search_obj);
                let ws_body = get_search_ws_body(
                    body.context.message_id,
                    body.context.transaction_id,
                    final_objs,
                );
                let ws_json = serde_json::to_value(ws_body).unwrap();
                let _ = websocket_srv
                    .send_msg(
                        ws_params,
                        WebSocketActionType::ProductSearch,
                        ws_json,
                        Some(NotificationProcessType::Immediate),
                    )
                    .await;
                transaction
                    .commit()
                    .await
                    .context("Failed to commit SQL transaction to store save products")?;
            } else {
                let data = save_cache_to_db(
                    &mut transaction,
                    &body.context.location.country.code,
                    &body.context.domain.get_category_domain(),
                    &final_objs,
                    body.context.timestamp,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .context("Failed to commit SQL transaction to store save products")?;
                save_cache_to_elastic_search(&pool, elastic_search_client, data).await?;
            }
        }
    }
    Ok(())
}
