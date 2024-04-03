use serde::de::IntoDeserializer;
use uuid::Uuid;

use super::schemas::{ONDCSearchMessage, ONDCSearchRequest};
use crate::{
    configuration::ONDCSetting,
    routes::{
        ondc::schemas::{
            ONDCActionType, ONDCContext, ONDCContextCity, ONDCContextCountry, ONDCContextLocation,
            ONDCDomain, ONDCVersion,
        },
        product::schemas::ProductSearchRequest,
        schemas::UserAccount,
    },
};

pub fn get_common_context(
    domain_category_id: String,
    action: ONDCActionType,
    bap_id: String,
    bap_uri: String,
) -> Result<ONDCContext, anyhow::Error> {
    // todo!()
    let ondc_domain: ONDCDomain = serde_json::from_str(&format!("ONDC:{}", domain_category_id))?;
    Ok(ONDCContext {
        // domain: ONDCDomain::from(ondc_domain),
        domain: ondc_domain,
        location: ONDCContextLocation {
            city: ONDCContextCity {
                code: "".to_string(),
            },
            country: ONDCContextCountry {
                code: "".to_string(),
            },
        },
        action: action,
        version: ONDCVersion::V2point2,
        transaction_id: Uuid::new_v4(),
        message_id: Uuid::new_v4(),
        bap_id: bap_id,
        bap_uri: bap_uri,
        timestamp: todo!(),
        bpp_id: None,
        bpp_uri: None,
        ttl: "PT30".to_string(),
    })
}

pub fn get_ondc_search_message_obj() -> Result<ONDCSearchMessage, anyhow::Error> {
    todo!()
}

pub fn get_ondc_payload_from_search_request(
    _user_account: UserAccount,
    search_request: ProductSearchRequest,
    ondc_obj: ONDCSetting,
) -> Result<ONDCSearchRequest, anyhow::Error> {
    let ondc_context = get_common_context(
        search_request.domain_category_id,
        ONDCActionType::Search,
        ondc_obj.bap.id,
        ondc_obj.bap.uri,
    )?;
    let ondc_seach_message = get_ondc_search_message_obj()?;
    Ok(ONDCSearchRequest {
        context: ondc_context,
        message: ondc_seach_message,
    })
}

pub fn get_ondc_search_payload(
    user_account: UserAccount,
    search_request: ProductSearchRequest,
    ondc_obj: ONDCSetting,
) -> Result<String, anyhow::Error> {
    let ondc_search_request_obj =
        get_ondc_payload_from_search_request(user_account, search_request, ondc_obj)?;
    let ondc_search_payload = serde_json::to_string(&ondc_search_request_obj)?;
    Ok(ondc_search_payload)
}
