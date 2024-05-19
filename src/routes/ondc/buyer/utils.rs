use bigdecimal::num_traits::float;
use chrono::Utc;
//use fake::faker::address::raw::Longitude;
use uuid::Uuid;

use super::schemas::{
    ONDCFulfillmentStopType, ONDCFulfillmentType, ONDCIntentTag, ONDCPaymentType,
    ONDCSearchFulfillment, ONDCSearchIntent, ONDCSearchLocation, ONDCSearchMessage,
    ONDCSearchPayment, ONDCSearchRequest, ONDCSearchStop,
};
use crate::{
    configuration::ONDCSetting,
    constants::ONDC_TTL,
    routes::{
        ondc::{
            buyer::schemas::BuyerFeeType,
            schemas::{
                ONDCActionType, ONDCContext, ONDCContextCity, ONDCContextCountry,
                ONDCContextLocation, ONDCDomain, ONDCVersion,
            },
        },
        product::schemas::{ProductFulFillmentLocations, ProductSearchRequest},
        schemas::UserAccount,
    },
    schemas::CountryCode,
    utils::get_gps_string,
};

pub fn get_common_context(
    domain_category_id: &String,
    action: ONDCActionType,
    bap_id: &str,
    bap_uri: &str,
    country_code: &CountryCode,
) -> Result<ONDCContext, anyhow::Error> {
    // todo!()
    // let ondc_domain: ONDCDomain = serde_json::from_str(&format!("ONDC:{}", domain_category_id))?;
    let ondc_domain = ONDCDomain::get_ondc_domain(&domain_category_id)?;
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
        action: action,
        version: ONDCVersion::V2point2,
        transaction_id: Uuid::new_v4(),
        message_id: Uuid::new_v4(),
        bap_id: bap_id.to_string(),
        bap_uri: bap_uri.to_string(),
        timestamp: Utc::now(),
        bpp_id: None,
        bpp_uri: None,
        ttl: ONDC_TTL.to_owned(),
    })
}

pub fn set_search_tag(user_account: UserAccount) -> Vec<ONDCIntentTag> {
    vec![ONDCIntentTag::get_buyer_tag(
        BuyerFeeType::Percentage,
        "0".to_string(),
    )]
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
    return ondc_fulfillment_stops;
}

pub fn get_ondc_search_message_obj(
    user_account: UserAccount,
    search_request: &ProductSearchRequest,
) -> Result<ONDCSearchMessage, anyhow::Error> {
    Ok(ONDCSearchMessage {
        intent: ONDCSearchIntent {
            fulfillment: ONDCSearchFulfillment {
                r#type: ONDCFulfillmentType::get_ondc_fulfillment(&search_request.fulfillment_type),
                stops: get_search_fulfillment_stops(&search_request.fulfillment_locations),
            },
            item: todo!(),
            tags: todo!(),
            payment: ONDCSearchPayment {
                r#type: ONDCPaymentType::get_ondc_payment(search_request.payment_type),
            },
            provider: None,
        },
    })
}

pub fn get_ondc_payload_from_search_request(
    user_account: UserAccount,
    search_request: ProductSearchRequest,
    ondc_obj: &ONDCSetting,
) -> Result<ONDCSearchRequest, anyhow::Error> {
    let ondc_context = get_common_context(
        &search_request.domain_category_id,
        ONDCActionType::Search,
        &ondc_obj.bap.id,
        &ondc_obj.bap.uri,
        &search_request.country_code,
    )?;
    let ondc_seach_message = get_ondc_search_message_obj(user_account, &search_request)?;
    Ok(ONDCSearchRequest {
        context: ondc_context,
        message: ondc_seach_message,
    })
}

pub fn get_ondc_search_payload(
    user_account: UserAccount,
    search_request: ProductSearchRequest,
    ondc_obj: &ONDCSetting,
) -> Result<String, anyhow::Error> {
    let ondc_search_request_obj =
        get_ondc_payload_from_search_request(user_account, search_request, &ondc_obj)?;
    let ondc_search_payload = serde_json::to_string(&ondc_search_request_obj)?;
    Ok(ondc_search_payload)
}
