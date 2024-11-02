#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::routes::ondc::buyer::schemas::{ONDCFulfillmentType, ONDCPaymentType};
    use crate::routes::ondc::buyer::utils::{
        get_ondc_search_message_obj, get_ondc_search_payment_obj, get_search_fulfillment_obj,
    };
    use crate::routes::product::schemas::{
        CategoryDomain, FulfillmentType, PaymentType, ProductFulFillmentLocations,
        ProductSearchRequest, ProductSearchType,
    };
    use crate::schemas::{CountryCode, RegisteredNetworkParticipant};
    use crate::user_client::BusinessAccount;
    use crate::utils::tests::{
        get_dummy_business_account, get_dummy_registed_np_detail, get_dummy_user_account,
    };
    #[tokio::test]
    async fn test_payment_type() {
        let ondc_payment_type_res = get_ondc_search_payment_obj(&Some(PaymentType::CashOnDelivery));
        assert!(ondc_payment_type_res.is_some());
        let ondc_payment_type = ondc_payment_type_res.unwrap();
        assert!(ondc_payment_type.r#type == ONDCPaymentType::OnFulfillment);

        let ondc_payment_type_res = get_ondc_search_payment_obj(&Some(PaymentType::PrePaid));
        assert!(ondc_payment_type_res.is_some());
        let ondc_payment_type = ondc_payment_type_res.unwrap();
        assert!(ondc_payment_type.r#type == ONDCPaymentType::PreFulfillment);

        let ondc_payment_type_res = get_ondc_search_payment_obj(&Some(PaymentType::Credit));
        assert!(ondc_payment_type_res.is_some());
        let ondc_payment_type = ondc_payment_type_res.unwrap();
        assert!(ondc_payment_type.r#type == ONDCPaymentType::PostFulfillment);
    }

    #[tokio::test]
    async fn test_fulfillment_type() {
        let location_obj = ProductFulFillmentLocations {
            latitude: 1.2323,
            longitude: 1.2323,
            area_code: "673642".to_string(),
        };
        let location_list = vec![location_obj];
        let fullfillment_obj =
            get_search_fulfillment_obj(&Some(FulfillmentType::Delivery), Some(&location_list));

        match fullfillment_obj {
            Some(ref obj) => {
                assert!(obj.r#type == ONDCFulfillmentType::Delivery);
                assert!(obj.stops.is_some());
            }
            None => assert!(false, "Expected Some, but got None"),
        }

        let fullfillment_obj =
            get_search_fulfillment_obj(&Some(FulfillmentType::SelfPickup), Some(&location_list));

        match fullfillment_obj {
            Some(ref obj) => {
                assert!(obj.r#type == ONDCFulfillmentType::SelfPickup);
                assert!(obj.stops.is_some());
            }
            None => assert!(false, "Expected Some, but got None"),
        }
    }

    #[tokio::test]
    async fn test_ondc_search_payload_generation() {
        let user_obj = get_dummy_user_account(
            "sanu".to_string(),
            "9562279968".to_string(),
            "sanushilshad@gmail.com".to_string(),
        );
        let business_obj: BusinessAccount = get_dummy_business_account();
        let np_detail: RegisteredNetworkParticipant = get_dummy_registed_np_detail();
        let mut search_req = ProductSearchRequest {
            query: "RET".to_string(),
            transaction_id: Uuid::new_v4(),
            message_id: Uuid::new_v4(),
            domain_category_code: CategoryDomain::Grocery,
            country_code: CountryCode::IND,
            payment_type: Some(PaymentType::CashOnDelivery),
            fulfillment_type: Some(FulfillmentType::Delivery),
            search_type: ProductSearchType::Item,
            fulfillment_locations: None,
            city_code: "std:080".to_string(),
            update_cache: false,
        };

        // seach by item
        let message_res =
            get_ondc_search_message_obj(&user_obj, &business_obj, &search_req, &np_detail);
        assert!(message_res.is_ok());
        assert!(message_res.unwrap().intent.item.is_some());

        // seach by category
        search_req.search_type = ProductSearchType::Category;
        let message_res =
            get_ondc_search_message_obj(&user_obj, &business_obj, &search_req, &np_detail);
        assert!(message_res.is_ok());
        let message_obj = message_res.unwrap();
        assert!(message_obj.intent.item.is_none());
        assert!(message_obj.intent.category.is_some());

        // seach by fulfillment
        search_req.search_type = ProductSearchType::Fulfillment;
        let message_res =
            get_ondc_search_message_obj(&user_obj, &business_obj, &search_req, &np_detail);

        assert!(message_res.is_ok());
        let message_obj = message_res.unwrap();
        assert!(message_obj.intent.category.is_none());
        assert!(message_obj.intent.item.is_none());
        assert!(message_obj.intent.fulfillment.is_some());

        search_req.search_type = ProductSearchType::City;
        let message_res =
            get_ondc_search_message_obj(&user_obj, &business_obj, &search_req, &np_detail);

        assert!(message_res.is_ok());
        let message_obj = message_res.unwrap();
        assert!(message_obj.intent.category.is_some());
        assert!(message_obj.intent.item.is_none());
        assert!(message_obj.intent.fulfillment.is_none());
        assert!(message_obj.intent.payment.is_none());
    }
}
