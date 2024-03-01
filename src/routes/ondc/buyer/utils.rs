use super::schemas::{ONDCSearchMessage, ONDCSearchRequest};
use crate::routes::{
    ondc::schemas::ONDCContext, product::schemas::ProductSearchRequest, schemas::UserAccount,
};

pub fn get_common_context() -> Result<ONDCContext, anyhow::Error> {
    todo!()

    // ONDCContext {
    //     domain
    // - location
    // - country
    // - city
    // - action
    // - version
    // - transaction_id
    // - message_id
    // - timestamp
    // - bap_id
    // - bap_uri
    // - bpp_id
    // - bpp_uri
    // - ttl
    //     }
    // }
}

pub fn get_ondc_search_message_obj() -> Result<ONDCSearchMessage, anyhow::Error> {
    todo!()
}

pub fn get_ondc_payload_from_search_request(
    _user_account: UserAccount,
    _search_request: ProductSearchRequest,
) -> Result<ONDCSearchRequest, anyhow::Error> {
    let ondc_context = get_common_context()?;
    let ondc_seach_message = get_ondc_search_message_obj()?;
    Ok(ONDCSearchRequest {
        context: ondc_context,
        message: ondc_seach_message,
    })
}

pub fn get_ondc_search_payload(
    user_account: UserAccount,
    search_request: ProductSearchRequest,
) -> Result<String, anyhow::Error> {
    let ondc_search_request_obj =
        get_ondc_payload_from_search_request(user_account, search_request)?;
    let ondc_search_payload = serde_json::to_string(&ondc_search_request_obj)?;
    Ok(ondc_search_payload)
}
