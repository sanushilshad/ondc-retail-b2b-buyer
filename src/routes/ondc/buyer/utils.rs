use std::collections::HashMap;

use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
//use fake::faker::address::raw::Longitude;
use uuid::Uuid;

use super::schemas::{
    ONDCFeeType, ONDCFulfillmentStopType, ONDCFulfillmentType, ONDCIntentTag, ONDCPaymentType,
    ONDCSearchCategory, ONDCSearchDescriptor, ONDCSearchFulfillment, ONDCSearchIntent,
    ONDCSearchItem, ONDCSearchLocation, ONDCSearchMessage, ONDCSearchPayment, ONDCSearchRequest,
    ONDCSearchStop,
};

use crate::constants::ONDC_TTL;
use crate::errors::GenericError;

use crate::routes::ondc::schemas::{
    ONDCActionType, ONDCContext, ONDCContextCity, ONDCContextCountry, ONDCContextLocation,
    ONDCDomain, ONDCVersion,
};
use crate::routes::product::schemas::{
    ProductFulFillmentLocations, ProductSearchRequest, ProductSearchType,
};
use crate::routes::product::ProductSearchError;
use crate::routes::schemas::{BusinessAccount, UserAccount};
use crate::routes::user::utils::get_default_vector_value;
use crate::schemas::{CountryCode, NetworkCall, RegisteredNetworkParticipant};
use crate::utils::get_gps_string;

pub fn get_common_context(
    transaction_id: Uuid,
    message_id: Uuid,
    domain_category_code: &str,
    action: ONDCActionType,
    bap_id: &str,
    bap_uri: &str,
    country_code: &CountryCode,
) -> Result<ONDCContext, GenericError> {
    // todo!()
    // let ondc_domain: ONDCDomain = serde_json::from_str(&format!("ONDC:{}", domain_category_code))?;
    let ondc_domain = ONDCDomain::get_ondc_domain(domain_category_code)?;
    Ok(ONDCContext {
        // domain: ONDCDomain::from(ondc_domain),
        domain: ondc_domain,
        location: ONDCContextLocation {
            city: ONDCContextCity {
                code: "std:080".to_string(),
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

pub fn get_search_tag(
    business_account: &BusinessAccount,
    np_detail: &RegisteredNetworkParticipant,
) -> Result<Vec<ONDCIntentTag>, ProductSearchError> {
    let buyer_id: Option<&str> = get_default_vector_value(
        &business_account.default_vector_type,
        &business_account.vectors,
    );
    match buyer_id {
        None => Err(ProductSearchError::ValidationError(format!(
            "{} doesn't exist for buyer",
            &business_account.default_vector_type.to_string()
        ))),
        Some(id) => Ok(vec![
            ONDCIntentTag::get_buyer_fee_tag(
                ONDCFeeType::get_fee_type(&np_detail.fee_type),
                &np_detail.fee_value.to_string(),
            ),
            ONDCIntentTag::get_buyer_id_tag(&business_account.default_vector_type, id),
        ]),
    }
}

pub fn get_search_fulfillment_stops(
    fulfillment_locations: &Vec<ProductFulFillmentLocations>,
) -> Vec<ONDCSearchStop> {
    let mut ondc_fulfillment_stops: Vec<ONDCSearchStop> = Vec::new();
    for fulfillment_location in fulfillment_locations {
        let search_fulfillment_end_obj = ONDCSearchStop {
            r#type: ONDCFulfillmentStopType::End,
            location: ONDCSearchLocation {
                gps: get_gps_string(
                    fulfillment_location.latitude,
                    fulfillment_location.longitude,
                ),
                area_code: fulfillment_location.area_code.to_string(),
            },
        };

        ondc_fulfillment_stops.push(search_fulfillment_end_obj);
    }
    ondc_fulfillment_stops
}

pub fn get_search_by_item(search_request: &ProductSearchRequest) -> Option<ONDCSearchItem> {
    if search_request.search_type == ProductSearchType::Item {
        return Some(ONDCSearchItem {
            descriptor: Some(ONDCSearchDescriptor {
                name: search_request.query.to_owned(),
            }),
        });
    }

    None
}

pub fn get_search_by_category(search_request: &ProductSearchRequest) -> Option<ONDCSearchCategory> {
    if search_request.search_type == ProductSearchType::Category {
        return Some(ONDCSearchCategory {
            id: search_request.query.to_owned(),
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
    Ok(ONDCSearchMessage {
        intent: ONDCSearchIntent {
            fulfillment: Some(ONDCSearchFulfillment {
                r#type: ONDCFulfillmentType::get_ondc_fulfillment(&search_request.fulfillment_type),
                stops: get_search_fulfillment_stops(&search_request.fulfillment_locations),
            }),
            tags: get_search_tag(business_account, np_detail)?,
            payment: ONDCSearchPayment {
                r#type: ONDCPaymentType::get_ondc_payment(&search_request.payment_type),
            },
            item: get_search_by_item(search_request),

            provider: None,
            category: get_search_by_category(search_request),
        },
    })
}

pub fn get_ondc_payload_from_search_request(
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    search_request: &ProductSearchRequest,
    np_detail: &RegisteredNetworkParticipant,
) -> Result<ONDCSearchRequest, ProductSearchError> {
    let ondc_context = get_common_context(
        search_request.transaction_id,
        search_request.message_id,
        &search_request.domain_category_code,
        ONDCActionType::Search,
        &np_detail.subscriber_id,
        &np_detail.subscriber_uri,
        &search_request.country_code,
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
) -> Result<Value, anyhow::Error> {
    let final_url = format!("{}/{}", url, action);
    let client = Client::new();
    let mut header_map = HashMap::new();
    header_map.insert("Authorization", header);
    let network_call = NetworkCall { client };
    println!("{}", final_url);
    println!("{}", payload);
    println!("{}", header);
    let result = network_call
        .async_post_call_with_retry(&final_url, Some(payload), Some(header_map))
        .await;

    match result {
        Ok(value) => Ok(value),
        Err(err) => Err(anyhow::Error::from(err)), // Convert NetworkError to anyhow::Error
    }
}
