use std::collections::HashMap;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::Utc;
use reqwest::Client;
use sqlx::PgPool;
//use fake::faker::address::raw::Longitude;
use uuid::Uuid;

use super::schemas::{
    ONDCFeeType, ONDCFulfillmentStopType, ONDCFulfillmentType, ONDCImage, ONDCOnSearchItemPrice,
    ONDCOnSearchItemQuantity, ONDCOnSearchItemTag, ONDCOnSearchPayment,
    ONDCOnSearchProviderDescriptor, ONDCOnSearchProviderLocation, ONDCOnSearchRequest,
    ONDCSearchCategory, ONDCSearchDescriptor, ONDCSearchFulfillment, ONDCSearchIntent,
    ONDCSearchItem, ONDCSearchLocation, ONDCSearchMessage, ONDCSearchPayment, ONDCSearchRequest,
    ONDCSearchStop, ONDCTag, ONDCTagItemCode, ONDCTagType, OnSearchContentType, TagTrait,
    UnitizedProductQty, WSCreatorContactData, WSItemPayment, WSProductCategory, WSProductCreator,
    WSSearch, WSSearchBPP, WSSearchCity, WSSearchCountry, WSSearchData, WSSearchItem,
    WSSearchItemPrice, WSSearchItemQty, WSSearchItemQtyMeasure, WSSearchItemQuantity,
    WSSearchProductProvider, WSSearchProvider, WSSearchProviderLocation, WSSearchState,
};

use crate::constants::ONDC_TTL;

use crate::routes::ondc::schemas::{
    ONDCActionType, ONDCContext, ONDCContextCity, ONDCContextCountry, ONDCContextLocation,
    ONDCDomain, ONDCVersion,
};
use crate::routes::ondc::{ONDCErrorCode, ONDCResponse};
use crate::routes::product::schemas::{
    CategoryDomain, FulfillmentType, PaymentType, ProductFulFillmentLocations,
    ProductSearchRequest, ProductSearchType, SearchRequestModel,
};
use crate::routes::product::ProductSearchError;
use crate::routes::user::schemas::{BusinessAccount, UserAccount};
use crate::routes::user::utils::get_default_vector_value;
use crate::schemas::{
    CountryCode, NetworkCall, ONDCNetworkType, RegisteredNetworkParticipant, WebSocketParam,
};
use crate::utils::get_gps_string;
use anyhow::anyhow;

#[allow(clippy::too_many_arguments)]
pub fn get_common_context(
    transaction_id: Uuid,
    message_id: Uuid,
    domain_category_code: &CategoryDomain,
    action: ONDCActionType,
    bap_id: &str,
    bap_uri: &str,
    country_code: &CountryCode,
    city_code: &str,
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
        transaction_id,
        message_id,
        bap_id: bap_id.to_string(),
        bap_uri: bap_uri.to_string(),
        timestamp: Utc::now(),
        bpp_id: None,
        bpp_uri: None,
        ttl: ONDC_TTL.to_owned(),
    })
}

fn get_search_tag(
    business_account: &BusinessAccount,
    np_detail: &RegisteredNetworkParticipant,
) -> Result<Vec<ONDCTag>, ProductSearchError> {
    let buyer_id: Option<&str> = get_default_vector_value(
        &business_account.default_vector_type,
        &business_account.vectors,
    );
    let ondc_buyer_id_type = &business_account
        .default_vector_type
        .get_ondc_vector_type()?;
    match buyer_id {
        None => Err(ProductSearchError::ValidationError(format!(
            "{} doesn't exist for buyer",
            &business_account.default_vector_type.to_string()
        ))),
        Some(id) => Ok(vec![
            ONDCTag::get_buyer_fee_tag(
                ONDCFeeType::get_fee_type(&np_detail.fee_type),
                &np_detail.fee_value.to_string(),
            ),
            ONDCTag::get_buyer_id_tag(ondc_buyer_id_type, id),
        ]),
    }
}

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
pub fn get_search_fulfillment_obj(
    fulfillment_type: &Option<FulfillmentType>,
    locations: Option<&Vec<ProductFulFillmentLocations>>,
) -> Option<ONDCSearchFulfillment> {
    if let Some(fulfillment_type) = fulfillment_type {
        return Some(ONDCSearchFulfillment {
            r#type: fulfillment_type.get_ondc_fulfillment(),
            stops: get_search_fulfillment_stops(locations),
        });
    }

    None
}

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
            tags: get_search_tag(business_account, np_detail)?,
            payment: payment_obj,
            item: get_search_by_item(search_request),

            provider: None,
            category: get_search_by_category(search_request),
        },
    })
}

fn get_ondc_payload_from_search_request(
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
        &search_request.country_code,
        &search_request.city_code,
    )?;
    let ondc_seach_message =
        get_ondc_search_message_obj(user_account, business_account, search_request, np_detail)?;
    Ok(ONDCSearchRequest {
        context: ondc_context,
        message: ondc_seach_message,
    })
}

pub fn get_ondc_search_payload(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    search_request: &ProductSearchRequest,
    np_detail: &RegisteredNetworkParticipant,
) -> Result<ONDCSearchRequest, ProductSearchError> {
    let ondc_search_request_obj = get_ondc_payload_from_search_request(
        user_account,
        business_account,
        search_request,
        np_detail,
    )?;

    Ok(ondc_search_request_obj)
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
                Err(anyhow!(error.message))
            } else {
                Ok(response_obj)
            }
        }
        Err(err) => Err(anyhow::Error::from(err)),
    }
}

#[tracing::instrument(name = "Fetch Search WebSocket Params", skip())]
pub fn get_websocket_params_from_search_req(search_model: SearchRequestModel) -> WebSocketParam {
    WebSocketParam {
        user_id: search_model.user_id,
        business_id: search_model.business_id,
        device_id: search_model.device_id,
    }
}

#[tracing::instrument(name = "Fetch Product Search Params", skip(pool))]
pub async fn get_product_search_params(
    pool: &PgPool,
    transaction_id: &Uuid,
    message_id: &Uuid,
) -> Result<Option<SearchRequestModel>, anyhow::Error> {
    let row = sqlx::query_as!(
        SearchRequestModel,
        r#"SELECT transaction_id, user_id, business_id, device_id, update_cache
        FROM search_request
        WHERE transaction_id = $1 AND message_id = $2 ORDER BY created_at DESC
        "#,
        transaction_id,
        message_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

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

fn get_ws_location_mapping(
    ondc_location: &ONDCOnSearchProviderLocation,
) -> WSSearchProviderLocation {
    WSSearchProviderLocation {
        id: &ondc_location.id,
        gps: &ondc_location.gps,
        address: &ondc_location.address,
        city: WSSearchCity {
            code: &ondc_location.city.code,
            name: ondc_location.city.name.as_deref(),
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

fn get_ws_search_item_payment_objs(ondc_payment_obj: &ONDCOnSearchPayment) -> WSItemPayment {
    WSItemPayment {
        r#type: ondc_payment_obj.r#type.get_payment(),
        collected_by: ondc_payment_obj
            .collected_by
            .as_ref()
            .unwrap_or(&ONDCNetworkType::Bap),
    }
}
pub fn get_product_from_on_search_request(
    on_search_obj: &ONDCOnSearchRequest,
) -> Result<Option<WSSearchData>, anyhow::Error> {
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
                code: catalog.descriptor.code.as_deref(),
                short_desc: &catalog.descriptor.short_desc,
                long_desc: &catalog.descriptor.long_desc,
                images: urls,
            },
        }));
    }

    Ok(None)
}
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

fn search_tag_item_list_from_tag<'a>(
    tag: &'a [ONDCOnSearchItemTag],
    tag_descriptor_code: &ONDCTagType,
) -> Vec<&'a ONDCOnSearchItemTag> {
    tag.iter()
        .filter(|t| &t.descriptor.code == tag_descriptor_code)
        .collect()
}

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
