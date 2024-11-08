use super::{
    BreakupTitleType, LookupData, LookupRequest, ONDCActionType, ONDCBreakUp, ONDCCancelMessage,
    ONDCCancelRequest, ONDCConfirmMessage, ONDCConfirmOrder, ONDCConfirmProvider, ONDCContext,
    ONDCContextCity, ONDCContextCountry, ONDCContextLocation, ONDCDomain, ONDCFeeType,
    ONDCSearchStop, ONDCStatusMessage, ONDCStatusRequest, ONDCTag, ONDCVersion, OndcUrl,
};

use crate::user_client::{BusinessAccount, UserAccount};
use crate::{
    constants::ONDC_TTL, routes::product::ProductSearchError, utils::get_default_vector_value,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Serializer;
use sqlx::PgPool;
use uuid::Uuid;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use bigdecimal::{BigDecimal, ToPrimitive};

use super::schemas::{
    BulkSellerInfo, BulkSellerLocationInfo, BulkSellerProductInfo, ONDCAmount, ONDCBilling,
    ONDCBreakupItemInfo, ONDCContact, ONDCCountry, ONDCCustomer, ONDCFulfillment,
    ONDCFulfillmentDescriptor, ONDCFulfillmentState, ONDCFulfillmentStopType, ONDCFulfillmentType,
    ONDCImage, ONDCInitMessage, ONDCInitOrder, ONDCInitPayment, ONDCInitProvider, ONDCInitRequest,
    ONDCLocationId, ONDCOnConfirmPayment, ONDCOnSearchItemPrice, ONDCOnSearchItemQuantity,
    ONDCOnSearchItemTag, ONDCOnSearchPayment, ONDCOnSearchProviderDescriptor,
    ONDCOnSearchProviderLocation, ONDCOnSearchRequest, ONDCOrderCancellationFee,
    ONDCOrderCancellationTerm, ONDCOrderFulfillmentEnd, ONDCOrderItemQuantity, ONDCOrderParams,
    ONDCOrderStatus, ONDCPaymentParams, ONDCPaymentSettlementCounterparty,
    ONDCPaymentSettlementDetail, ONDCPaymentStatus, ONDCQuantityCountInt, ONDCQuantitySelect,
    ONDCQuote, ONDCRequestModel, ONDCSearchCategory, ONDCSearchDescriptor, ONDCSearchFulfillment,
    ONDCSearchIntent, ONDCSearchItem, ONDCSearchLocation, ONDCSearchMessage, ONDCSearchPayment,
    ONDCSearchRequest, ONDCSelectFulfillmentLocation, ONDCSelectMessage, ONDCSelectOrder,
    ONDCSelectPayment, ONDCSelectProvider, ONDCSelectRequest, ONDCSelectedItem,
    ONDCSellerLocationInfo, ONDCSellerProductInfo, ONDCState, ONDCTagItemCode, ONDCTagType,
    ONDConfirmRequest, OnSearchContentType, TagTrait,
};
use serde_json::Value;

use crate::domain::EmailObject;
use crate::routes::ondc::schemas::{ONDCCity, ONDCPerson, ONDCSellerInfo, OrderRequestParamsModel};
use crate::routes::ondc::{ONDCErrorCode, ONDCResponse};
use crate::routes::order::errors::{
    ConfirmOrderError, InitOrderError, OrderCancelError, OrderStatusError, SelectOrderError,
};
use crate::routes::order::schemas::{
    BuyerTerms, CancellationFeeType, Commerce, CommerceBilling, CommerceCancellationFee,
    CommerceCancellationTerm, CommerceFulfillment, CommerceItem, CommercePayment, DropOffData,
    OrderCancelRequest, OrderConfirmRequest, OrderDeliveyTerm, OrderInitBilling, OrderInitRequest,
    OrderSelectFulfillment, OrderSelectItem, OrderSelectRequest, OrderStatusRequest, OrderType,
    PaymentCollectedBy, PickUpData, SelectFulfillmentLocation, SettlementBasis,
};
use crate::routes::product::schemas::{
    CategoryDomain, FulfillmentType, PaymentType, ProductFulFillmentLocations,
    ProductSearchRequest, ProductSearchType, SearchRequestModel, UnitizedProductQty,
    WSCreatorContactData, WSItemPayment, WSProductCategory, WSProductCreator, WSSearch,
    WSSearchBPP, WSSearchCity, WSSearchCountry, WSSearchData, WSSearchItem, WSSearchItemPrice,
    WSSearchItemQty, WSSearchItemQtyMeasure, WSSearchItemQuantity, WSSearchProductProvider,
    WSSearchProvider, WSSearchProviderLocation, WSSearchState,
};

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
    .await?;
    Ok(row)
}

#[tracing::instrument(name = "Save lookup data to db", skip(pool))]
pub async fn save_lookup_data_to_db(pool: &PgPool, data: &LookupData) -> Result<(), anyhow::Error> {
    let uuid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO network_participant (id, subscriber_id, br_id, subscriber_url, signing_public_key, domain, encr_public_key, type, uk_id, created_on)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) ON CONFLICT (subscriber_id, type) DO NOTHING;
        "#,
        &uuid,
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
    let buyer_id: Option<&str> = get_default_vector_value(
        &business_account.default_vector_type,
        &business_account.vectors,
    );
    let ondc_buyer_id_type = &business_account
        .default_vector_type
        .get_ondc_vector_type()?;
    match buyer_id {
        Some(id) => Ok(ONDCTag::get_buyer_id_tag(ondc_buyer_id_type, id)),
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
    fulfillment_locations: Option<&Vec<ProductFulFillmentLocations>>,
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
    if search_request.search_type == ProductSearchType::Category
        || search_request.search_type == ProductSearchType::City
    {
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
    locations: Option<&Vec<ProductFulFillmentLocations>>,
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
        user_id: search_model.user_id,
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

#[tracing::instrument(name = "Fetch ONDC Order Params", skip(pool))]
pub async fn get_ondc_order_params(
    pool: &PgPool,
    transaction_id: Uuid,
    message_id: Uuid,
    action_type: ONDCActionType,
) -> Result<Option<ONDCOrderParams>, anyhow::Error> {
    let row = sqlx::query_as!(
        ONDCOrderParams,
        r#"SELECT message_id, transaction_id, user_id, business_id, device_id
        FROM ondc_buyer_order_req
        WHERE transaction_id = $1 AND message_id = $2 AND action_type = $3 AND user_id is not NULL AND business_id is not NULL ORDER BY created_on DESC
        "#,
        &transaction_id,
        &message_id,
        &action_type.to_string()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub fn get_ondc_order_param_from_req(ondc_req: &ONDCRequestModel) -> WebSocketParam {
    WebSocketParam {
        device_id: ondc_req.device_id.clone(),
        user_id: ondc_req.user_id,
        business_id: ondc_req.business_id,
    }
}

pub fn _get_order_param_from_param_req(ondc_req: &OrderRequestParamsModel) -> WebSocketParam {
    WebSocketParam {
        device_id: ondc_req.device_id.clone(),
        user_id: ondc_req.user_id,
        business_id: ondc_req.business_id,
    }
}

#[tracing::instrument(name = "get price obj from ondc price obj", skip())]
pub fn get_price_obj_from_ondc_price_obj(
    price: &ONDCOnSearchItemPrice,
) -> Result<WSSearchItemPrice, anyhow::Error> {
    return Ok(WSSearchItemPrice {
        currency: price.currency.to_owned(),
        value: BigDecimal::from_str(&price.value).unwrap(),
        offered_value: price
            .offered_value
            .as_ref()
            .map(|v| BigDecimal::from_str(v).unwrap_or_else(|_| BigDecimal::from(0))),
        maximum_value: BigDecimal::from_str(&price.maximum_value).unwrap(),
    });
}

#[tracing::instrument(name = "get ws location mapping", skip())]
fn get_ws_location_mapping(
    ondc_location: &ONDCOnSearchProviderLocation,
) -> WSSearchProviderLocation {
    WSSearchProviderLocation {
        id: &ondc_location.id,
        gps: &ondc_location.gps,
        address: &ondc_location.address,
        city: WSSearchCity {
            code: &ondc_location.city.code,
            name: ondc_location.city.name.as_str(),
        },
        country: WSSearchCountry {
            code: &ondc_location.country.code,
            name: ondc_location.country.name.as_deref(),
        },
        state: WSSearchState {
            code: &ondc_location.state.code,
            name: ondc_location.state.name.as_deref(),
        },
        area_code: &ondc_location.area_code,
    }
}

#[tracing::instrument(name = "ws search provider from ondc provider", skip())]
pub fn ws_search_provider_from_ondc_provider<'a>(
    id: &'a str,
    rating: Option<&'a str>,
    descriptor: &'a ONDCOnSearchProviderDescriptor,
) -> WSSearchProductProvider<'a> {
    let images: Vec<&str> = descriptor
        .images
        .iter()
        .map(|image| image.get_value())
        .collect();
    let videos: Vec<&str> = descriptor
        .additional_desc
        .iter()
        .filter_map(|desc| {
            if desc.content_type == OnSearchContentType::Mp4 {
                Some(desc.url.as_str())
            } else {
                None
            }
        })
        .collect();
    WSSearchProductProvider {
        id,
        rating,
        name: &descriptor.name,
        code: &descriptor.code,
        short_desc: &descriptor.short_desc,
        long_desc: &descriptor.long_desc,
        images,
        videos,
    }
}

// #[tracing::instrument(name = "get ws quantity from ondc quantity", skip())]
fn get_ws_quantity_from_ondc_quantity(
    ondc_quantity: &ONDCOnSearchItemQuantity,
) -> WSSearchItemQuantity {
    WSSearchItemQuantity {
        unitized: UnitizedProductQty {
            unit: ondc_quantity.unitized.measure.unit.clone(),
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

fn map_ws_item_categories(category_ids: &[String]) -> Vec<WSProductCategory> {
    category_ids
        .iter()
        .map(|f| WSProductCategory {
            code: f.to_string(),
            name: "".to_owned(),
        })
        .collect()
}

fn map_item_images(images: &[ONDCImage]) -> Vec<&str> {
    images.iter().map(|image| image.get_value()).collect()
}

fn get_payment_mapping(
    payment_objs: &[ONDCOnSearchPayment],
) -> HashMap<&str, &ONDCOnSearchPayment> {
    payment_objs.iter().map(|f| (f.id.as_str(), f)).collect()
}

// #[tracing::instrument(name = "get ws search item payment objs", skip())]
fn get_ws_search_item_payment_objs(ondc_payment_obj: &ONDCOnSearchPayment) -> WSItemPayment {
    WSItemPayment {
        r#type: ondc_payment_obj.r#type.get_payment(),
        collected_by: ondc_payment_obj
            .collected_by
            .as_ref()
            .unwrap_or(&ONDCNetworkType::Bap),
    }
}

#[tracing::instrument(name = "get product from on search request", skip())]
pub fn get_product_from_on_search_request(
    on_search_obj: &ONDCOnSearchRequest,
) -> Result<Option<WSSearchData>, anyhow::Error> {
    let subscriber_id = on_search_obj.context.bpp_id.as_deref().unwrap_or("");
    let subscriber_uri = on_search_obj.context.bpp_uri.as_deref().unwrap_or("");
    if let Some(catalog) = &on_search_obj.message.catalog {
        let mut payment_mapping = get_payment_mapping(&catalog.payments);
        let mut provider_list: Vec<WSSearchProvider> = vec![];
        let descriptor_image = &catalog.descriptor.images;
        let fullfllments = &catalog.fulfillments;
        let fulfillment_map: HashMap<&String, &ONDCFulfillmentType> =
            fullfllments.iter().map(|f| (&f.id, &f.r#type)).collect();
        let urls: Vec<&str> = descriptor_image
            .iter()
            .map(|image| image.get_value())
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
                    &ONDCTagType::G3,
                    &ONDCTagItemCode::TaxRate.to_string(),
                )
                .unwrap_or("0.00");
                let payment_obj = item
                    .payment_ids
                    .iter()
                    .filter_map(|key| {
                        payment_mapping
                            .get(key.as_str())
                            .map(|f| get_ws_search_item_payment_objs(f))
                    })
                    .collect();
                let fulfillment_type_list: Vec<FulfillmentType> = item
                    .fulfillment_ids
                    .iter()
                    .filter_map(|key| {
                        fulfillment_map
                            .get(key)
                            .map(|f| f.get_fulfillment_from_ondc())
                    })
                    .collect();
                let images: Vec<&str> = map_item_images(&item.descriptor.images);
                let categories: Vec<WSProductCategory> = map_ws_item_categories(&item.category_ids);
                let prod_obj = WSSearchItem {
                    id: &item.id,
                    name: &item.descriptor.name,
                    code: item.descriptor.code.as_deref(),
                    domain_category: on_search_obj.context.domain.get_category_domain(),
                    price: get_price_obj_from_ondc_price_obj(&item.price)?,
                    parent_item_id: item.parent_item_id.as_deref(),
                    recommended: item.recommended,
                    creator: WSProductCreator {
                        name: &item.creator.descriptor.name,
                        contact: WSCreatorContactData {
                            name: &item.creator.descriptor.contact.name,
                            address: &item.creator.descriptor.contact.address.full,
                            phone: &item.creator.descriptor.contact.phone,
                            email: &item.creator.descriptor.contact.email,
                        },
                    },
                    fullfillment_type: fulfillment_type_list,
                    images,
                    location_ids: item.location_ids.iter().map(|s| s.as_str()).collect(),
                    categories,
                    tax_rate: BigDecimal::from_str(tax_rate)
                        .unwrap_or_else(|_| BigDecimal::from(0)),

                    quantity: get_ws_quantity_from_ondc_quantity(&item.quantity),
                    payment_types: payment_obj, // payment_types: todo!(),
                };
                product_list.push(prod_obj)
            }
            let provider = WSSearchProvider {
                items: product_list,
                locations: location_obj,
                provider_detail: ws_search_provider_from_ondc_provider(
                    &provider_obj.id,
                    provider_obj.rating.as_deref(),
                    &provider_obj.descriptor,
                ),
            };
            provider_list.push(provider)
        }
        return Ok(Some(WSSearchData {
            providers: provider_list,
            bpp: WSSearchBPP {
                name: &catalog.descriptor.name,
                subscriber_id,
                subscriber_uri,
                code: catalog.descriptor.code.as_deref(),
                short_desc: &catalog.descriptor.short_desc,
                long_desc: &catalog.descriptor.long_desc,
                images: urls,
            },
        }));
    }

    Ok(None)
}

#[tracing::instrument(name = "get search ws body", skip())]
pub fn get_search_ws_body<'a>(
    message_id: Uuid,
    transaction_id: Uuid,
    search_data: &'a WSSearchData,
) -> WSSearch<'a> {
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

fn get_ondc_select_tags(business_account: &BusinessAccount) -> Result<Vec<ONDCTag>, anyhow::Error> {
    match get_buyer_id_tag(business_account) {
        Ok(tag_option) => Ok(vec![tag_option]),
        Err(e) => Err(anyhow!("Failed to get buyer ID tag: {}", e)),
    }
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

#[tracing::instrument(name = "get ondc select message body", skip())]
fn get_ondc_select_fulfillments(
    seller_location_mapping: &HashMap<String, ONDCSellerLocationInfo>,
    fulfillments: &Vec<OrderSelectFulfillment>,
) -> Vec<ONDCFulfillment> {
    let mut fulfillment_objs: Vec<ONDCFulfillment> = vec![];
    let location_obj = seller_location_mapping.iter().next().unwrap();
    for fulfillment in fulfillments {
        let stops: Option<Vec<ONDCOrderFulfillmentEnd>> =
            if fulfillment.r#type == FulfillmentType::Delivery {
                Some(get_ondc_select_fulfillment_end(&fulfillment.location))
            } else {
                None
            };

        let tags = if location_obj.1.country_code != fulfillment.location.country.code {
            get_fulfillment_tags(&fulfillment.delivery_terms)
        } else {
            None
        };
        fulfillment_objs.push(ONDCFulfillment {
            id: fulfillment.id.clone(),
            r#type: fulfillment.r#type.get_ondc_fulfillment_type(),
            tags,
            stops,
            customer: None,
        })
    }

    return fulfillment_objs;
}

#[tracing::instrument(name = "get ondc select message body", skip())]
fn get_ondc_select_message(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    order_request: &OrderSelectRequest,
    seller_location_mapping: &HashMap<String, ONDCSellerLocationInfo>,
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
    let select_tag = get_ondc_select_tags(business_account)
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
) -> Result<ONDCSelectRequest, SelectOrderError> {
    let context = get_ondc_select_context(order_request, bap_detail, bpp_detail)?;
    let message = get_ondc_select_message(
        user_account,
        business_account,
        order_request,
        seller_location_mapping,
    )?;
    Ok(ONDCSelectRequest { context, message })
}

#[tracing::instrument(name = "save ondc seller product info", skip())]
pub fn create_bulk_seller_product_info_objs<'a>(
    body: &'a WSSearchData,
) -> BulkSellerProductInfo<'a> {
    let mut seller_subscriber_ids: Vec<&str> = vec![];
    let mut provider_ids: Vec<&str> = vec![];
    let mut item_codes: Vec<Option<&str>> = vec![];
    let mut item_names: Vec<&str> = vec![];
    let mut item_ids: Vec<&str> = vec![];
    let mut tax_rates: Vec<BigDecimal> = vec![];
    let mut image_objs: Vec<Value> = vec![];
    let mut mrps: Vec<BigDecimal> = vec![];
    let mut unit_prices: Vec<BigDecimal> = vec![];
    for provider in &body.providers {
        for item in &provider.items {
            seller_subscriber_ids.push(body.bpp.subscriber_id);
            provider_ids.push(provider.provider_detail.id);
            item_ids.push(item.id);
            item_codes.push(item.code);
            item_names.push(item.name);
            tax_rates.push(item.tax_rate.clone());
            mrps.push(item.price.maximum_value.clone());
            unit_prices.push(item.price.maximum_value.clone());
            // for image_url in item.images.iter() {
            image_objs.push(serde_json::to_value(&item.images).unwrap());
            // }
            // image_objs.push(item_image_objs)
        }
    }

    return BulkSellerProductInfo {
        seller_subscriber_ids,
        provider_ids,
        item_codes,
        item_ids,
        item_names,
        tax_rates,
        image_objs,
        mrps,
        unit_prices,
    };
}

#[tracing::instrument(name = "save ondc seller product info", skip(pool, data))]
pub async fn save_ondc_seller_product_info<'a>(
    pool: &PgPool,
    data: &'a WSSearchData<'a>,
) -> Result<(), anyhow::Error> {
    let product_data = create_bulk_seller_product_info_objs(data);
    sqlx::query!(
        r#"
        INSERT INTO ondc_seller_product_info (
            seller_subscriber_id,
            provider_id,
            item_id,
            item_code,
            item_name,
            tax_rate,
            images,
            unit_price,
            mrp
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
            $9::decimal[]
        )
        ON CONFLICT (seller_subscriber_id, provider_id, item_id) 
        DO UPDATE SET 
            item_name = EXCLUDED.item_name,
            tax_rate = EXCLUDED.tax_rate,
            images = EXCLUDED.images,
            unit_price = EXCLUDED.unit_price,
            mrp =  EXCLUDED.mrp;
        "#,
        &product_data.seller_subscriber_ids[..] as &[&str],
        &product_data.provider_ids[..] as &[&str],
        &product_data.item_ids[..] as &[&str],
        &product_data.item_codes[..] as &[Option<&str>],
        &product_data.item_names[..] as &[&str],
        &product_data.tax_rates[..] as &[BigDecimal],
        &product_data.image_objs[..],
        &product_data.unit_prices[..] as &[BigDecimal],
        &product_data.mrps[..] as &[BigDecimal],
    )
    .execute(pool)
    .await
    .map_err(|e| {
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
) -> Result<Vec<ONDCSellerProductInfo>, anyhow::Error> {
    let row: Vec<ONDCSellerProductInfo> = sqlx::query_as!(
        ONDCSellerProductInfo,
        r#"SELECT item_name, item_id, item_code, seller_subscriber_id, provider_id, tax_rate, unit_price, mrp, images  from ondc_seller_product_info where 
        provider_id  = $1 AND seller_subscriber_id=$2 AND item_id::text = ANY($3)"#,
        provider_id,
        bpp_id,
        item_id_list as &Vec<&str>
    )
    .fetch_all(pool)
    .await?;
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
) -> Result<HashMap<String, ONDCSellerProductInfo>, anyhow::Error> {
    let seller_product_info =
        fetch_ondc_seller_product_info(pool, bpp_id, provider_id, item_id_list).await?;
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

#[tracing::instrument(name = "Fetch order request params", skip(pool))]
pub async fn fetch_order_params(
    pool: &PgPool,
    transaction_id: Uuid,
    message_id: Uuid,
    action_type: &ONDCActionType,
) -> Result<Option<OrderRequestParamsModel>, anyhow::Error> {
    let row = sqlx::query_as!(
        OrderRequestParamsModel,
        r#"SELECT transaction_id, message_id, user_id, business_id, device_id
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
                customer: Some(ONDCCustomer {
                    person: ONDCPerson {
                        creds: None,
                        name: business_account.company_name.clone(),
                    },
                }),
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
                transaction_id: None,
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
            id: "RAP:001".to_string(),
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
) -> BulkSellerLocationInfo<'a> {
    let mut seller_subscriber_ids: Vec<&str> = vec![];
    let mut provider_ids = vec![];
    let mut location_ids = vec![];
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
    for provider in &body.providers {
        for (key, location) in &provider.locations {
            let gps_data = location
                .gps
                .split(',')
                .map(|s| BigDecimal::from_str(s).unwrap_or_else(|_| BigDecimal::from(0).clone()))
                .collect::<Vec<_>>();

            seller_subscriber_ids.push(body.bpp.subscriber_id);
            provider_ids.push(provider.provider_detail.id);
            location_ids.push(key.as_str());
            latitudes.push(gps_data.first().cloned().unwrap_or(BigDecimal::from(0)));
            longitudes.push(gps_data.get(1).cloned().unwrap_or(BigDecimal::from(0)));
            addresses.push(location.address);
            city_codes.push(location.city.code);
            city_names.push(location.city.name);
            state_codes.push(location.state.code);
            state_names.push(location.state.name);
            country_names.push(location.country.name);
            country_codes.push(location.country.code);
            area_codes.push(location.area_code);
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
    };
}

#[tracing::instrument(name = "save ondc seller location info", skip(pool, data))]
pub async fn save_ondc_seller_location_info<'a>(
    pool: &PgPool,
    data: &'a WSSearchData<'a>,
) -> Result<(), anyhow::Error> {
    let seller_data = create_bulk_seller_location_info_objs(data);
    sqlx::query!(
        r#"
        INSERT INTO ondc_seller_location_info (
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
            area_code 
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
            $11::country_code[],
            $12::text[],
            $13::text[]
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
    )
    .execute(pool)
    .await
    .map_err(|e| {
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
        country_name from ondc_seller_location_info where 
        provider_id  = $1 AND seller_subscriber_id=$2 AND location_id::text = ANY($3)"#,
        provider_id,
        bpp_id,
        location_id_list as &Vec<String>
    )
    .fetch_all(pool)
    .await?;
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
pub fn create_bulk_seller_info_objs<'a>(body: &'a WSSearchData) -> BulkSellerInfo<'a> {
    let mut seller_subscriber_ids: Vec<&str> = vec![];
    let mut provider_ids = vec![];
    let mut provider_names = vec![];

    for provider in &body.providers {
        seller_subscriber_ids.push(body.bpp.subscriber_id);
        provider_ids.push(provider.provider_detail.id);
        provider_names.push(provider.provider_detail.name);
    }

    return BulkSellerInfo {
        seller_subscriber_ids,
        provider_ids,
        provider_names,
    };
}

#[tracing::instrument(name = "save ondc seller info", skip(pool, data))]
pub async fn save_ondc_seller_info<'a>(
    pool: &PgPool,
    data: &'a WSSearchData<'a>,
) -> Result<(), anyhow::Error> {
    let seller_data = create_bulk_seller_info_objs(data);
    sqlx::query!(
        r#"
        INSERT INTO ondc_seller_info (
            seller_subscriber_id,
            provider_id,
            provider_name
        )
        SELECT *
        FROM UNNEST(
            $1::text[], 
            $2::text[], 
            $3::text[]
        )
        ON CONFLICT (seller_subscriber_id, provider_id) 
        DO UPDATE SET 
            provider_name = EXCLUDED.provider_name
        "#,
        &seller_data.seller_subscriber_ids[..] as &[&str],
        &seller_data.provider_ids[..] as &[&str],
        &seller_data.provider_names[..] as &[&str]
    )
    .execute(pool)
    .await
    .map_err(|e| {
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
        r#"SELECT  seller_subscriber_id, provider_id, provider_name from ondc_seller_info where 
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
    let order_id = order
        .urn
        .as_deref()
        .ok_or_else(|| OrderStatusError::ValidationError("Order id is missing".to_owned()))?;
    let message = get_ondc_status_message(order_id);
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
    let order_id = order
        .urn
        .as_deref()
        .ok_or_else(|| OrderCancelError::ValidationError("Order id is missing".to_owned()))?;
    let message = get_ondc_cancel_message(order_id, &cancel_request.reason_id);
    Ok(ONDCCancelRequest { context, message })
}
