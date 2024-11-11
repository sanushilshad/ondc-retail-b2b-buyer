use super::models::{
    CommerceBppTermsModel, CommerceDataModel, CommerceDocumentModel, CommerceFulfillmentModel,
    CommerceItemModel, CommercePaymentModel, DropOffContactModel, DropOffDataModel,
    DropOffLocationModel, FulfillmentInstruction, OrderBillingModel, OrderCancellationFeeModel,
    OrderCancellationTermModel, PaymentSettlementDetailModel, PickUpContactModel, PickUpDataModel,
    PickUpLocationModel, SellerPaymentDetailModel, TimeRangeModel,
};
use super::schemas::{
    BasicNetworkData, BulkCancelFulfillmentData, BulkCancelItemData, BulkConfirmFulfillmentData,
    BulkStatusFulfillmentData, BuyerTerm, Commerce, CommerceBPPTerms, CommerceBilling,
    CommerceCancellationFee, CommerceCancellationTerm, CommerceDocument, CommerceFulfillment,
    CommerceItem, CommercePayment, CommerceSeller, DocumentType, DropOffData, FulfillmentContact,
    FulfillmentLocation, OrderSelectFulfillment, OrderSelectRequest, PaymentSettlementDetail,
    PickUpData, PickUpFulfillmentLocation, SelectFulfillmentLocation, SellerPaymentDetail,
    TimeRange, TradeType,
};
use crate::constants::ONDC_TTL;
use crate::routes::ondc::schemas::{
    BreakupTitleType, ONDCBilling, ONDCBreakUp, ONDCConfirmFulfillmentStartLocation, ONDCContact,
    ONDCFulfillment, ONDCFulfillmentCategoryType, ONDCFulfillmentStopType, ONDCFulfillmentTime,
    ONDCOnConfirmFulfillment, ONDCOnConfirmPayment, ONDCOnConfirmRequest, ONDCOnInitPayment,
    ONDCOnInitRequest, ONDCOnSelectFulfillment, ONDCOnSelectPayment, ONDCOnSelectRequest,
    ONDCOrderCancellationTerm, ONDCOrderFulfillmentEnd, ONDCPaymentCollectedBy, ONDCSelectRequest,
    ONDCSellerLocationInfo, ONDCSellerProductInfo, ONDCTag, ONDCTagItemCode, ONDCTagType, TagTrait,
};
use crate::routes::ondc::utils::{
    fetch_ondc_seller_info, get_ondc_seller_location_info_mapping,
    get_ondc_seller_product_info_mapping, get_ondc_seller_product_mapping_key,
    get_tag_value_from_list,
};
use crate::routes::ondc::{
    LookupData, ONDCActionType, ONDCConfirmFulfillmentEndLocation, ONDCDocument,
    ONDCFulfillmentInstruction, ONDCOnCancelRequest, ONDCOnStatusRequest, ONDCOnUpdateRequest,
    ONDCPaymentType, ONDCTitleName,
};
use crate::routes::order::schemas::{
    CommerceStatusType, DeliveryTerm, FulfillmentCategoryType, FulfillmentStatusType, IncoTermType,
    OrderType, PaymentCollectedBy, PaymentStatus, ServiceableType, SettlementBasis,
};
use crate::routes::product::schemas::{CategoryDomain, FulfillmentType, PaymentType};
use crate::schemas::DataSource;
use crate::schemas::{
    CountryCode, CurrencyType, FeeType, RegisteredNetworkParticipant, RequestMetaData,
};
use crate::user_client::{BusinessAccount, UserAccount};
use crate::utils::get_gps_string;
use anyhow::{anyhow, Context};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::Utc;
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(name = "Save Product Search Request", skip(pool))]
pub async fn save_ondc_order_request(
    pool: &PgPool,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    meta_data: &RequestMetaData,
    request_payload: &Value,
    transaction_id: Uuid,
    message_id: Uuid,
    action_type: ONDCActionType,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO ondc_buyer_order_req (message_id, transaction_id, device_id,  user_id, business_id, action_type, request_payload)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        &message_id,
        &transaction_id,
        &meta_data.device_id,
        &user_account.id,
        &business_account.id,
        &action_type.to_string(),
        &request_payload

    )
    .execute(pool).await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving ONDC order request")
    })?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(name = "save rfq", skip(transaction))]
pub async fn save_rfq_order(
    transaction: &mut Transaction<'_, Postgres>,
    select_request: &OrderSelectRequest,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    bpp_detail: &LookupData,
    bap_detail: &RegisteredNetworkParticipant,
    provider_name: &str,
    currency_code: &CurrencyType,
) -> Result<Uuid, anyhow::Error> {
    let order_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_data (id, external_urn, record_type, record_status, 
        domain_category_code, buyer_id, seller_id, seller_name, buyer_name, source, created_on, created_by, bpp_id, bpp_uri, bap_id, bap_uri, quote_ttl, city_code, country_code, currency_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        ON CONFLICT (external_urn) 
        DO NOTHING
        "#,
        order_id,
        &select_request.transaction_id,
        &select_request.order_type as &OrderType,
        CommerceStatusType::QuoteRequested as CommerceStatusType,
        &select_request.domain_category_code as &CategoryDomain,
        &business_account.id,
        &select_request.provider_id,
        &provider_name,
        &business_account.company_name,
        DataSource::PlaceOrder as DataSource,
        Utc::now(),
        &user_account.id,
        &select_request.bpp_id,
        bpp_detail.subscriber_url,
        &bap_detail.subscriber_id,
        &bap_detail.subscriber_uri,
        &select_request.ttl,
        &select_request.fulfillments[0].location.city.code,
        &select_request.fulfillments[0].location.country.code as &CountryCode,
        &currency_code as &CurrencyType,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(order_id)
}

#[tracing::instrument(name = "delete  fulfillment", skip(transaction))]
pub async fn delete_fulfillment_by_order_id(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query(
        r#"
        DELETE FROM commerce_fulfillment_data
        WHERE commerce_data_id = $1
        "#,
    )
    .bind(order_id);

    transaction
        .execute(query) // Dereference the transaction
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute delete query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while deleting the order fulfillment")
        })?;

    Ok(())
}

pub fn create_drop_off_from_rfq_select_fulfullment(
    fulfillment: &SelectFulfillmentLocation,
) -> DropOffDataModel {
    // let mut drop_list = vec![];
    // for fulfillment in select_fulfillments {
    DropOffDataModel {
        location: DropOffLocationModel {
            gps: fulfillment.gps.clone(),
            area_code: fulfillment.area_code.clone(),
            address: Some(fulfillment.address.clone()),
            city: fulfillment.city.name.clone(),
            country: fulfillment.country.code.clone(),
            state: fulfillment.state.clone(),
        },
        contact: DropOffContactModel {
            mobile_no: fulfillment.contact_mobile_no.clone(),
            email: None,
        },
        time_range: None,
        instruction: None,
    }
}

fn get_pick_up_location_from_ondc_seller_fulfillment(
    pick_up_location: &ONDCSellerLocationInfo,
) -> PickUpDataModel {
    PickUpDataModel {
        location: PickUpLocationModel {
            gps: get_gps_string(
                pick_up_location.latitude.to_f64().unwrap_or(0.00),
                pick_up_location.longitude.to_f64().unwrap_or(0.00),
            ),
            area_code: pick_up_location.area_code.clone(),
            address: pick_up_location.address.clone(),
            city: pick_up_location.city_name.clone(),
            country: pick_up_location.country_code.clone(),
            state: pick_up_location
                .state_name
                .clone()
                .unwrap_or(pick_up_location.state_code.clone()),
        },
        contact: PickUpContactModel {
            mobile_no: "".to_owned(),
            email: None,
        },
        time_range: None,
        instruction: None,
    }
}

#[tracing::instrument(name = "save rfq fulfillment", skip(transaction))]
pub async fn save_rfq_fulfillment(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
    fulfillments: &Vec<OrderSelectFulfillment>,
    pick_up_location: &ONDCSellerLocationInfo,
    seller_location_map: &HashMap<String, ONDCSellerLocationInfo>,
) -> Result<(), anyhow::Error> {
    // delete_fulfillment_by_order_id(transaction, order_id).await?;
    let mut id_list = vec![];
    let mut order_list = vec![];
    let mut fulfillment_id_list = vec![];
    let mut fulfillment_type_list = vec![];
    let mut drop_off_data_list = vec![];
    let mut incoterms_list = vec![];
    let mut delivery_place_list = vec![];
    let mut pick_up_data_list = vec![];
    let mut trade_type_list = vec![];
    let pick_up_data = get_pick_up_location_from_ondc_seller_fulfillment(pick_up_location);
    let location_obj = seller_location_map.iter().next().unwrap();
    for fulfillment in fulfillments {
        order_list.push(order_id);
        id_list.push(Uuid::new_v4());
        fulfillment_id_list.push(fulfillment.id.as_str());
        fulfillment_type_list.push(&fulfillment.r#type);
        if fulfillment.r#type == FulfillmentType::Delivery {
            drop_off_data_list.push(
                serde_json::to_value(create_drop_off_from_rfq_select_fulfullment(
                    &fulfillment.location,
                ))
                .unwrap(),
            );
        }

        incoterms_list.push(fulfillment.delivery_terms.as_ref().map(|e| &e.inco_terms));
        delivery_place_list.push(
            fulfillment
                .delivery_terms
                .as_ref()
                .map(|e| e.place_of_delivery.as_str()),
        );

        pick_up_data_list.push(serde_json::to_value(pick_up_data.clone()).unwrap());
        trade_type_list.push(
            if location_obj.1.country_code != fulfillment.location.country.code {
                TradeType::Import
            } else {
                TradeType::Domestic
            },
        );
    }
    if !id_list.is_empty() {
        let query = sqlx::query!(
            r#"
            INSERT INTO commerce_fulfillment_data (id, commerce_data_id, fulfillment_id, fulfillment_type, inco_terms, place_of_delivery, drop_off_data, pickup_data, trade_type)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::text[], $4::fulfillment_type[],$5::inco_term_type [], $6::text[], $7::jsonb[],  $8::jsonb[], $9::trade_type[]);
            "#,
            &id_list[..] as &[Uuid],
            &order_list[..] as &[Uuid],
            &fulfillment_id_list[..] as &[&str],
            &fulfillment_type_list[..] as &[&FulfillmentType],
            &incoterms_list[..] as &[Option<&IncoTermType>],
            &delivery_place_list[..] as &[Option<&str>],
            &drop_off_data_list[..] as &[Value],
            &pick_up_data_list[..] as &[Value],
            &trade_type_list[..] as &[TradeType]
        );

        transaction.execute(query).await.map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while saving RFQ to database request")
        })?;
    }

    Ok(())
}

#[tracing::instrument(name = "save rfq items", skip(transaction))]
pub async fn save_order_select_items(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
    select_request: &OrderSelectRequest,
    product_map: &HashMap<String, ONDCSellerProductInfo>,
) -> Result<(), anyhow::Error> {
    let item_count = select_request.items.len();
    let line_id_list: Vec<Uuid> = (0..item_count).map(|_| Uuid::new_v4()).collect();
    let order_id_list: Vec<Uuid> = vec![order_id; item_count];
    let mut item_id_list = vec![];
    let mut item_code_list: Vec<Option<&str>> = vec![];
    let mut item_name_list = vec![];
    let mut location_id_list = vec![];
    let mut fulfillment_id_list = vec![];
    let mut item_image_list = vec![];
    let mut qty_list = vec![];
    let mut mrp_list = vec![];
    let mut unit_price_list = vec![];
    let mut tax_rate_list = vec![];
    let mut item_req_list = vec![];
    let mut packagin_req_list = vec![];
    for item in &select_request.items {
        let key = get_ondc_seller_product_mapping_key(
            &select_request.bpp_id,
            &select_request.provider_id,
            &item.item_id,
        );
        if let Some(seller_item_obj) = product_map.get(&key) {
            item_code_list.push(seller_item_obj.item_code.as_deref());
            item_name_list.push(seller_item_obj.item_name.as_str());
            item_image_list.push(
                seller_item_obj
                    .images
                    .as_array()
                    .and_then(|images| images.first())
                    .and_then(|image| image.as_str())
                    .unwrap_or(""),
            );
            mrp_list.push(seller_item_obj.mrp.clone());
            unit_price_list.push(seller_item_obj.unit_price.clone());
            tax_rate_list.push(seller_item_obj.tax_rate.clone());
        } else {
            item_code_list.push(None);
            item_name_list.push("");
            item_image_list.push("");
            mrp_list.push(BigDecimal::from(0));
            unit_price_list.push(BigDecimal::from(0));
            tax_rate_list.push(BigDecimal::from(0));
        }
        // let item_name = '';
        // let item_image = ''.as_str();
        item_id_list.push(item.item_id.as_str());

        location_id_list.push(serde_json::to_value(&item.location_ids)?); // Serialize to JSON
        fulfillment_id_list.push(serde_json::to_value(&item.fulfillment_ids)?);

        qty_list.push(BigDecimal::from(item.qty));
        item_req_list.push(item.buyer_term.as_ref().map(|e| e.item_req.as_str()));
        packagin_req_list.push(item.buyer_term.as_ref().map(|e| e.packaging_req.as_str()));
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_data_line (id, commerce_data_id, item_id, item_name, item_code, item_image, 
            qty, location_ids, fulfillment_ids, tax_rate, mrp, unit_price, item_req, packaging_req)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::text[], $4::text[], $5::text[], $6::text[],
             $7::decimal[], $8::jsonb[], $9::jsonb[], $10::decimal[], $11::decimal[], $12::decimal[], $13::text[], $14::text[])
        ON CONFLICT (commerce_data_id, item_code) 
        DO NOTHING
        "#,
        &line_id_list[..] as &[Uuid],
        &order_id_list[..] as &[Uuid],
        &item_id_list[..] as &[&str],
        &item_name_list[..] as &[&str],
        &item_code_list[..] as &[Option<&str>], //change
        &item_image_list[..] as &[&str],        //change
        &qty_list[..] as &[BigDecimal],
        &location_id_list as &[Value],
        &fulfillment_id_list as &[Value],
        &tax_rate_list[..] as &[BigDecimal],
        &mrp_list[..] as &[BigDecimal],
        &unit_price_list[..] as &[BigDecimal],
        &item_req_list[..] as &[Option<&str>],
        &packagin_req_list[..] as &[Option<&str>],
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "delete order", skip(transaction))]
pub async fn delete_order(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query(
        r#"
        DELETE FROM commerce_data
        WHERE external_urn = $1
        "#,
    )
    .bind(id);

    transaction
        .execute(query) // Dereference the transaction
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute delete query: {:?}", e);
            anyhow::Error::new(e).context("A database failure occurred while deleting the order")
        })?;

    Ok(())
}

#[tracing::instrument(name = "save select payments", skip(transaction))]
pub async fn save_payment_obj_select(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
    payments: &Vec<PaymentType>,
) -> Result<(), anyhow::Error> {
    // delete_on_select_payment(transaction, order_id).await?;
    let mut id_list = vec![];
    let mut commerce_data_id_list = vec![];
    for _ in 0..payments.len() {
        id_list.push(Uuid::new_v4());
        commerce_data_id_list.push(order_id);
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_payment_data(id, commerce_data_id, payment_type)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::payment_type[])
        "#,
        &id_list[..] as &[Uuid],
        &commerce_data_id_list[..] as &[Uuid],
        &payments[..] as &[PaymentType]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving select payment to database request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "save request for quote", skip(pool))]
pub async fn initialize_order_select(
    pool: &PgPool,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    select_request: &OrderSelectRequest,
    bap_detail: &RegisteredNetworkParticipant,
    bpp_detail: &LookupData,
    seller_location_map: &HashMap<String, ONDCSellerLocationInfo>,
) -> Result<(), anyhow::Error> {
    let item_code_list: Vec<&str> = select_request
        .items
        .iter()
        .map(|item| item.item_id.as_str())
        .collect();

    let task1 = get_ondc_seller_product_info_mapping(
        pool,
        &bpp_detail.subscriber_id,
        &select_request.provider_id,
        &item_code_list,
    );
    let task2 =
        fetch_ondc_seller_info(pool, &bpp_detail.subscriber_id, &select_request.provider_id);
    let (seller_product_map_res, seller_info_map_res) = futures::future::join(task1, task2).await;
    let seller_product_map = seller_product_map_res?;
    let seller_info_map = seller_info_map_res?;
    let currency_code = seller_product_map
        .iter()
        .next()
        .map(|(_, product)| &product.currency_code) // Get the currency code from the first item
        .unwrap_or(&CurrencyType::Inr);
    let pick_up_location = seller_location_map
        .values()
        .next()
        .ok_or_else(|| anyhow!("Invalid Location Id"))?;

    let provider_name = seller_info_map
        .provider_name
        .ok_or_else(|| anyhow!("Invalid Product / Location"))?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    delete_order(&mut transaction, select_request.transaction_id).await?;

    let order_id = save_rfq_order(
        &mut transaction,
        select_request,
        user_account,
        business_account,
        bpp_detail,
        bap_detail,
        &provider_name,
        currency_code,
    )
    .await?;
    save_rfq_fulfillment(
        &mut transaction,
        order_id,
        &select_request.fulfillments,
        pick_up_location,
        seller_location_map,
    )
    .await?;

    save_order_select_items(
        &mut transaction,
        order_id,
        select_request,
        &seller_product_map,
    )
    .await?;

    save_payment_obj_select(&mut transaction, order_id, &select_request.payment_types).await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a order")?;

    Ok(())
}

pub fn create_drop_off_from_ondc_select_fulfullment(
    ondc_select_fulfillment: &[ONDCFulfillment],
    // contact: &ONDCContact,
) -> Option<DropOffDataModel> {
    if let Some(stops) = &ondc_select_fulfillment[0].stops {
        let location = &stops[0].location;
        let contact = &stops[0].contact;
        Some(DropOffDataModel {
            location: DropOffLocationModel {
                gps: location.gps.clone(),
                area_code: location.area_code.clone(),
                address: location.address.clone(),
                city: location.city.name.clone(),
                country: location.country.code.clone(),
                state: location.state.name.clone(),
            },
            contact: DropOffContactModel {
                mobile_no: contact.phone.clone(),
                email: contact.email.clone(),
            },
            time_range: None,
            instruction: None,
        })
    } else {
        None
    }
}

pub fn create_pick_off_from_ondc_select_fulfillment(
    ondc_select_fulfillment_ends: &Option<Vec<ONDCOrderFulfillmentEnd>>,
    // contact: &ONDCContact,
) -> Option<PickUpDataModel> {
    if let Some(ondc_select_fulfillment_end_res) = ondc_select_fulfillment_ends {
        for ondc_select_fulfillment_end in ondc_select_fulfillment_end_res {
            if ondc_select_fulfillment_end.r#type == ONDCFulfillmentStopType::Start {
                return Some(PickUpDataModel {
                    location: PickUpLocationModel {
                        gps: ondc_select_fulfillment_end.location.gps.clone(),
                        area_code: ondc_select_fulfillment_end.location.area_code.clone(),
                        address: ondc_select_fulfillment_end
                            .location
                            .address
                            .clone()
                            .unwrap_or("NA".to_owned()),
                        city: ondc_select_fulfillment_end.location.city.name.clone(),
                        country: ondc_select_fulfillment_end.location.country.code.clone(),
                        state: ondc_select_fulfillment_end.location.state.name.clone(),
                    },
                    contact: PickUpContactModel {
                        mobile_no: ondc_select_fulfillment_end.contact.phone.clone(),
                        email: ondc_select_fulfillment_end.contact.email.clone(),
                    },
                    time_range: None,
                    instruction: None,
                });
            }
        }
    }
    None
}

#[tracing::instrument(name = "save on select fulfillment", skip(transaction))]
pub async fn save_on_select_fulfillment(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
    select_fulfillment: &Vec<ONDCFulfillment>,
    on_select_fulfillments: &Vec<ONDCOnSelectFulfillment>,
    ondc_quote: &Vec<ONDCBreakUp>,
    pick_up_location: &ONDCSellerLocationInfo,
) -> Result<(), anyhow::Error> {
    // delete_fulfillment_by_order_id(transaction, order_id).await?;
    let mut id_list = vec![];
    let mut order_list = vec![];
    let mut fulfillment_id_list = vec![];
    let mut fulfillment_type_list = vec![];
    let mut drop_off_data_list = vec![];
    let mut incoterms_list = vec![];
    let mut delivery_place_list = vec![];
    let mut provider_name_list = vec![];
    let mut tat_list = vec![];
    let mut category_list = vec![];
    let mut servicable_status_list = vec![];
    let mut tracking_list = vec![];
    let drop_off_data = create_drop_off_from_ondc_select_fulfullment(select_fulfillment);
    let trade_type = if let Some(drop_off_data_obj) = &drop_off_data {
        if pick_up_location.country_code != drop_off_data_obj.location.country {
            Some(&TradeType::Import)
        } else {
            Some(&TradeType::Domestic)
        }
    } else {
        None
    };
    let drop_off_data_json = serde_json::to_value(drop_off_data).unwrap_or_default();
    let mut pickup_data_list = vec![];
    let mut delivery_charge_list = vec![];
    let mut packaging_charge_list = vec![];
    let mut convenience_fee_list = vec![];
    let mut trade_type_list = vec![];
    let delivery_charge_mapping =
        get_quote_item_value_mapping(ondc_quote, &BreakupTitleType::Delivery);
    let packaging_charge_mapping =
        get_quote_item_value_mapping(ondc_quote, &BreakupTitleType::Packing);
    let convenience_fee_mapping = get_quote_item_value_mapping(ondc_quote, &BreakupTitleType::Misc);
    let global_pick_up_data = get_pick_up_location_from_ondc_seller_fulfillment(pick_up_location);
    for fulfillment in on_select_fulfillments {
        order_list.push(order_id);
        id_list.push(Uuid::new_v4());
        fulfillment_id_list.push(fulfillment.id.as_str());

        fulfillment_type_list.push(
            if fulfillment.category == ONDCFulfillmentCategoryType::SelfPickup {
                &FulfillmentType::SelfPickup
            } else {
                &FulfillmentType::Delivery
            },
        );
        incoterms_list.push(select_fulfillment[0].tags.as_ref().map(|e| {
            e[0].get_tag_value(&ONDCTagItemCode::IncoTerms.to_string())
                .unwrap_or("")
        }));
        delivery_place_list.push(select_fulfillment[0].tags.as_ref().map(|e| {
            e[0].get_tag_value(&ONDCTagItemCode::NamedPlaceOfDelivery.to_string())
                .unwrap_or("")
        }));
        provider_name_list.push(fulfillment.provider_name.as_deref());
        tat_list.push(fulfillment.tat.as_str());
        category_list.push(fulfillment.category.get_category_type());
        servicable_status_list.push(fulfillment.state.descriptor.code.get_servicable_type());
        tracking_list.push(fulfillment.tracking);
        drop_off_data_list.push(drop_off_data_json.clone());
        // pickup_data_list.push(
        //     serde_json::to_value(create_pick_off_from_ondc_select_fulfillment(
        //         &fulfillment.stops,
        //     ))
        //     .unwrap_or_default(),
        // );
        if let Some(pick_up) = create_pick_off_from_ondc_select_fulfillment(&fulfillment.stops) {
            pickup_data_list.push(serde_json::to_value(pick_up).unwrap_or_default());
        } else {
            pickup_data_list.push(serde_json::to_value(global_pick_up_data.clone()).unwrap());
        }
        delivery_charge_list.push(
            delivery_charge_mapping
                .get(&fulfillment.id)
                .cloned()
                .unwrap_or(BigDecimal::from(0)),
        );
        packaging_charge_list.push(
            packaging_charge_mapping
                .get(&fulfillment.id)
                .cloned()
                .unwrap_or(BigDecimal::from(0)),
        );
        convenience_fee_list.push(
            convenience_fee_mapping
                .get(&fulfillment.id)
                .cloned()
                .unwrap_or(BigDecimal::from(0)),
        );
        trade_type_list.push(trade_type);
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_fulfillment_data (id, commerce_data_id, fulfillment_id, fulfillment_type, inco_terms, 
            place_of_delivery, drop_off_data, pickup_data, provider_name, servicable_status, tracking, tat, category, packaging_charge,
            delivery_charge, convenience_fee, trade_type)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::text[], $4::fulfillment_type[],$5::inco_term_type [], $6::text[],
             $7::jsonb[], $8::jsonb[], $9::text[], $10::fulfillment_servicability_status[], $11::bool[], $12::text[],
            $13::fulfillment_category_type[], $14::decimal[], $15::decimal[], $16::decimal[], $17::trade_type[]);
        "#,
        &id_list[..] as &[Uuid],
        &order_list[..] as &[Uuid],
        &fulfillment_id_list[..] as &[&str],
        &fulfillment_type_list[..] as &[&FulfillmentType],
        &incoterms_list[..] as &[Option<&str>],
        &delivery_place_list[..] as &[Option<&str>],
        &drop_off_data_list[..] as &[Value],
        &pickup_data_list[..] as &[Value],
        &provider_name_list[..] as &[Option<&str>],
        &servicable_status_list[..] as &[ServiceableType],
        &tracking_list[..] as &[bool],
        &tat_list[..] as &[&str],
        &category_list[..] as &[FulfillmentCategoryType],
        &packaging_charge_list[..] as &[BigDecimal],
        &delivery_charge_list[..] as &[BigDecimal],
        &convenience_fee_list[..] as &[BigDecimal],
        &trade_type_list[..] as &[Option<&TradeType>]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "save order on select", skip(transaction))]
pub async fn save_buyer_order_data_on_select(
    transaction: &mut Transaction<'_, Postgres>,
    ondc_select_req: &ONDCSelectRequest,
    ondc_on_select_req: &ONDCOnSelectRequest,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    provider_name: &str,
) -> Result<Uuid, anyhow::Error> {
    let grand_total =
        BigDecimal::from_str(&ondc_on_select_req.message.order.quote.price.value).unwrap();
    let order_id = Uuid::new_v4();
    let mut created_on = ondc_on_select_req.context.timestamp;
    let mut order_type = OrderType::SaleOrder;
    if ondc_select_req.context.ttl != ONDC_TTL {
        order_type = OrderType::PurchaseOrder;
        created_on = ondc_select_req.context.timestamp;
    };
    let order_status = if ondc_on_select_req.error.is_none() {
        CommerceStatusType::QuoteAccepted
    } else {
        CommerceStatusType::QuoteRejected
    };
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_data (id, external_urn, record_type, record_status,
        domain_category_code, buyer_id, seller_id, seller_name, buyer_name, source, created_on, created_by, bpp_id, bpp_uri,
        bap_id, bap_uri, quote_ttl, updated_on, currency_code, grand_total, city_code, country_code)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
        ON CONFLICT (external_urn)
        DO UPDATE SET
        record_status = EXCLUDED.record_status,
        updated_on = EXCLUDED.updated_on,
        currency_code = EXCLUDED.currency_code
        RETURNING id
        "#,
        order_id,
        &ondc_on_select_req.context.transaction_id,
        &order_type as &OrderType,
        &order_status as &CommerceStatusType,
        &ondc_on_select_req.context.domain.get_category_domain() as &CategoryDomain,
        &business_account.id,
        &ondc_on_select_req.message.order.provider.id,
        &provider_name,
        &business_account.company_name,
        DataSource::PlaceOrder as DataSource,
        created_on,
        &user_account.id,
        ondc_on_select_req.context.bpp_id.as_deref().unwrap_or(""),
        ondc_on_select_req.context.bpp_uri.as_deref().unwrap_or(""),
        &ondc_on_select_req.context.bap_id,
        &ondc_on_select_req.context.bap_uri,
        &ondc_select_req.context.ttl,
        ondc_select_req.context.timestamp,
        &ondc_on_select_req.message.order.quote.price.currency as &CurrencyType,
        &grand_total,
        &ondc_select_req.context.location.city.code,
        &ondc_select_req.context.location.country.code as &CountryCode,
    );

    let result = query.fetch_one(&mut **transaction).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(result.id)
}

#[tracing::instrument(name = "save order on on_select", skip(pool))]
pub async fn initialize_order_on_select(
    pool: &PgPool,
    on_select_request: &ONDCOnSelectRequest,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    ondc_select_req: &ONDCSelectRequest,
) -> Result<(), anyhow::Error> {
    let bpp_id = on_select_request.context.bpp_id.as_deref().unwrap_or("");
    let item_code_list: Vec<&str> = on_select_request
        .message
        .order
        .items
        .iter()
        .map(|item| item.id.as_str()) // Assuming item_id is a String
        .collect();
    let location_id_list: Vec<String> = on_select_request
        .message
        .order
        .provider
        .locations
        .iter()
        .map(|location| location.id.to_owned())
        .collect();
    let task1 = get_ondc_seller_product_info_mapping(
        pool,
        bpp_id,
        &on_select_request.message.order.provider.id,
        &item_code_list,
    );
    let task2 = get_ondc_seller_location_info_mapping(
        pool,
        bpp_id,
        &on_select_request.message.order.provider.id,
        &location_id_list,
    );
    let task3 = fetch_ondc_seller_info(pool, bpp_id, &on_select_request.message.order.provider.id);
    let (seller_product_map_res, seller_location_map_res, seller_info_map_res) =
        futures::future::join3(task1, task2, task3).await;
    let seller_product_map = seller_product_map_res?;
    let seller_location_map = seller_location_map_res?;
    let pick_up_location = seller_location_map
        .values()
        .next()
        .ok_or_else(|| anyhow!("Invalid Location Id"))?;
    let seller_info_map = seller_info_map_res?;

    let provider_name = seller_info_map.provider_name.unwrap_or_default();
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    delete_order(&mut transaction, ondc_select_req.context.transaction_id).await?;

    let order_id = save_buyer_order_data_on_select(
        &mut transaction,
        ondc_select_req,
        on_select_request,
        user_account,
        business_account,
        &provider_name,
    )
    .await?;

    let _ = save_order_on_select_items(
        &mut transaction,
        order_id,
        on_select_request,
        &seller_product_map,
    )
    .await;

    save_payment_obj_on_select(
        &mut transaction,
        order_id,
        &on_select_request.message.order.payments,
    )
    .await?;
    save_on_select_fulfillment(
        &mut transaction,
        order_id,
        &ondc_select_req.message.order.fulfillments,
        &on_select_request.message.order.fulfillments,
        &on_select_request.message.order.quote.breakup,
        pick_up_location,
    )
    .await?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")?;

    Ok(())
}

pub fn get_quote_item_value_mapping<'a>(
    breakups: &'a Vec<ONDCBreakUp>,
    title_type: &BreakupTitleType,
) -> HashMap<&'a String, BigDecimal> {
    let mut header_map = HashMap::new();
    for breakup in breakups {
        if &breakup.title_type == title_type {
            let break_up_value =
                BigDecimal::from_str(&breakup.price.value).unwrap_or_else(|_| BigDecimal::from(0));
            header_map.insert(&breakup.item_id, break_up_value);
        }
    }
    header_map
}

pub fn get_quote_item_breakup_mapping<'a>(
    breakups: &'a Vec<ONDCBreakUp>,
    title_type: &BreakupTitleType,
) -> HashMap<&'a String, &'a ONDCBreakUp> {
    let mut header_map = HashMap::new();
    for breakup in breakups {
        if &breakup.title_type == title_type {
            header_map.insert(&breakup.item_id, breakup);
        }
    }
    header_map
}

pub fn get_quote_item_breakup_mapping_for_refund(
    breakups: &Vec<ONDCBreakUp>,
) -> HashMap<String, &ONDCBreakUp> {
    let mut header_map = HashMap::new();
    for breakup in breakups {
        if breakup.title_type == BreakupTitleType::Refund {
            header_map.insert(format!("{}_{}", breakup.title, breakup.item_id), breakup);
        }
    }
    header_map
}

#[tracing::instrument(name = "delete on select payment", skip(transaction))]
pub async fn delete_on_select_payment(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query(
        r#"
        DELETE FROM commerce_payment
        WHERE commerce_data_id = $1
        "#,
    )
    .bind(id);

    transaction
        .execute(query) // Dereference the transaction
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute delete query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while deleting the on select payment")
        })?;

    Ok(())
}

#[tracing::instrument(name = "save on select payments", skip(transaction))]
pub async fn save_payment_obj_on_select(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
    payments: &Vec<ONDCOnSelectPayment>,
) -> Result<(), anyhow::Error> {
    // delete_on_select_payment(transaction, order_id).await?;
    let mut id_list = vec![];
    let mut commerce_data_id_list = vec![];
    let mut collected_by_list = vec![];
    let mut payment_type_list = vec![];
    for payment in payments {
        id_list.push(Uuid::new_v4());
        commerce_data_id_list.push(order_id);
        collected_by_list.push(payment.collected_by.get_type());
        payment_type_list.push(payment.r#type.get_payment());
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_payment_data(id, commerce_data_id, collected_by, payment_type)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::payment_collected_by_type[],  $4::payment_type[])
        "#,
        &id_list[..] as &[Uuid],
        &commerce_data_id_list[..] as &[Uuid],
        &collected_by_list[..] as &[PaymentCollectedBy],
        &payment_type_list[..] as &[PaymentType]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "save on select items", skip(transaction))]
pub async fn save_order_on_select_items(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
    ondc_on_select_request: &ONDCOnSelectRequest,
    product_map: &HashMap<String, ONDCSellerProductInfo>,
) -> Result<(), anyhow::Error> {
    let item_count = ondc_on_select_request.message.order.items.len();
    let line_id_list: Vec<Uuid> = (0..item_count).map(|_| Uuid::new_v4()).collect();
    let order_id_list: Vec<Uuid> = vec![order_id; item_count];
    let mut item_id_list = vec![];
    let mut item_code_list: Vec<Option<&str>> = vec![];
    let mut item_name_list = vec![];
    let mut location_id_list = vec![];
    let mut fulfillment_id_list = vec![];
    let mut item_image_list = vec![];
    let mut qty_list = vec![];
    let mut mrp_list = vec![];
    let mut unit_price_list = vec![];
    let mut tax_rate_list = vec![];
    let mut tax_amount_list = vec![];
    let mut discount_amount_list = vec![];
    let mut gross_amount_list = vec![];
    let mut available_qty_list = vec![];
    let mut item_req_list = vec![];
    let mut packaging_req_list = vec![];
    let discount_mapping = get_quote_item_value_mapping(
        &ondc_on_select_request.message.order.quote.breakup,
        &BreakupTitleType::Discount,
    );
    let tax_mapping = get_quote_item_value_mapping(
        &ondc_on_select_request.message.order.quote.breakup,
        &BreakupTitleType::Tax,
    );

    let item_breakup_mapping = get_quote_item_breakup_mapping(
        &ondc_on_select_request.message.order.quote.breakup,
        &BreakupTitleType::Item,
    );
    for item in &ondc_on_select_request.message.order.items {
        let key = get_ondc_seller_product_mapping_key(
            ondc_on_select_request
                .context
                .bpp_id
                .as_ref()
                .unwrap_or(&String::new()),
            &ondc_on_select_request.message.order.provider.id,
            &item.id,
        );
        let discount_amount = discount_mapping
            .get(&item.id)
            .cloned()
            .unwrap_or(BigDecimal::from(0));
        discount_amount_list.push(discount_amount);

        let tax_amount = tax_mapping
            .get(&item.id)
            .cloned()
            .unwrap_or(BigDecimal::from(0));

        if let Some(break_up) = item_breakup_mapping.get(&item.id) {
            unit_price_list.push(break_up.item.as_ref().map_or(BigDecimal::from(0), |a| {
                BigDecimal::from_str(&a.price.value).unwrap_or_else(|_| BigDecimal::from(0))
            }));
            available_qty_list.push(
                break_up
                    .quantity
                    .as_ref()
                    .map_or(BigDecimal::from(0), |a| BigDecimal::from(a.count)),
            );
            gross_amount_list.push(
                BigDecimal::from_str(&break_up.price.value).unwrap_or_else(|_| BigDecimal::from(0)),
            );
        } else {
            unit_price_list.push(BigDecimal::from(0));
            gross_amount_list.push(BigDecimal::from(0));
            available_qty_list.push(BigDecimal::from(0));
        }

        tax_amount_list.push(tax_amount);
        packaging_req_list.push(item.tags.as_ref().map(|tag| {
            get_tag_value_from_list(
                tag,
                ONDCTagType::BuyerTerms,
                &ONDCTagItemCode::PackagingsReq.to_string(),
            )
            .unwrap_or_default()
        }));
        item_req_list.push(item.tags.as_ref().map(|tag| {
            get_tag_value_from_list(
                tag,
                ONDCTagType::BuyerTerms,
                &ONDCTagItemCode::ItemReq.to_string(),
            )
            .unwrap_or_default()
        }));
        // if let Some(tags) = item.tags {
        //     packaging_req_list.push(get_tag_value_from_list(
        //         &tags,
        //         ONDCTagType::BuyerTerms,
        //         ONDCTagItemCode::PackagingsReq.to_string(),
        //     ));
        // } else
        // packagin_req_list.push(item.tags.as_ref().map(|a| {
        //     a[0].get_tag_value(&ONDCTagItemCode::PackagingsReq.to_string())
        //         .unwrap_or_default()
        // }));
        // item_req_list.push(item.tags.as_ref().map(|a| {
        //     a[0].get_tag_value(&ONDCTagItemCode::ItemReq.to_string())
        //         .unwrap_or_default()
        // }));
        if let Some(seller_item_obj) = product_map.get(&key) {
            item_code_list.push(seller_item_obj.item_code.as_deref());
            item_name_list.push(seller_item_obj.item_name.as_str());
            item_image_list.push(
                seller_item_obj
                    .images
                    .as_array()
                    .and_then(|images| images.first())
                    .and_then(|image| image.as_str())
                    .unwrap_or_default(),
            );
            mrp_list.push(seller_item_obj.mrp.clone());
            // unit_price_list.push(seller_item_obj.unit_price.clone());
            tax_rate_list.push(seller_item_obj.tax_rate.clone());
        } else {
            item_code_list.push(None);
            item_name_list.push("");
            item_image_list.push("");
            mrp_list.push(BigDecimal::from(0));

            tax_rate_list.push(BigDecimal::from(0));
        }
        item_id_list.push(item.id.as_str());

        location_id_list.push(serde_json::to_value(&item.location_ids)?); // Serialize to JSON
        fulfillment_id_list.push(serde_json::to_value(&item.fulfillment_ids)?);

        qty_list.push(BigDecimal::from(item.quantity.selected.count));
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_data_line (id, commerce_data_id, item_id, item_name, item_code, item_image, 
            qty, location_ids, fulfillment_ids, tax_rate, mrp, unit_price, discount_amount, tax_value, gross_total,
            available_qty,item_req, packaging_req)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::text[], $4::text[], $5::text[], $6::text[],
             $7::decimal[], $8::jsonb[], $9::jsonb[], $10::decimal[], $11::decimal[], $12::decimal[], $13::decimal[],
            $14::decimal[], $15::decimal[], $16::decimal[], $17::text[], $18::text[])
        ON CONFLICT (commerce_data_id, item_code) 
        DO UPDATE SET 
        fulfillment_ids = EXCLUDED.fulfillment_ids,
        unit_price = EXCLUDED.unit_price,
        discount_amount = EXCLUDED.discount_amount,
        tax_value = EXCLUDED.tax_value,
        gross_total = EXCLUDED.gross_total,
        available_qty = EXCLUDED.available_qty
        "#,
        &line_id_list[..] as &[Uuid],
        &order_id_list[..] as &[Uuid],
        &item_id_list[..] as &[&str],
        &item_name_list[..] as &[&str],
        &item_code_list[..] as &[Option<&str>],
        &item_image_list[..] as &[&str],
        &qty_list[..] as &[BigDecimal],
        &location_id_list as &[Value],
        &fulfillment_id_list as &[Value],
        &tax_rate_list[..] as &[BigDecimal],
        &mrp_list[..] as &[BigDecimal],
        &unit_price_list[..] as &[BigDecimal],
        &discount_amount_list[..] as &[BigDecimal],
        &tax_amount_list[..] as &[BigDecimal],
        &gross_amount_list[..] as &[BigDecimal],
        &available_qty_list[..] as &[BigDecimal],
        &item_req_list[..] as &[Option<&str>],
        &packaging_req_list[..] as &[Option<&str>],
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "fetch buyer commerce data", skip(pool))]
async fn get_commerce_data(
    pool: &PgPool,
    transaction_id: Uuid,
) -> Result<Option<CommerceDataModel>, anyhow::Error> {
    //vectors:sqlx::types::Json<Vec<UserVector>>
    let record = sqlx::query_as!(
        CommerceDataModel,
        r#"
        
        SELECT id, urn, external_urn, record_type as "record_type:OrderType", 
           record_status as "record_status:CommerceStatusType",
           domain_category_code as "domain_category_code:CategoryDomain", 
           buyer_id, seller_id, buyer_name, seller_name, source as "source:DataSource", 
           created_on, updated_on, deleted_on, is_deleted, created_by, grand_total, 
           bpp_id, bpp_uri, bap_id, bap_uri, quote_ttl,
           currency_code as "currency_code?:CurrencyType", city_code,
           country_code as "country_code:CountryCode",
           billing as "billing?:  Json<OrderBillingModel>",
           cancellation_terms as "cancellation_terms?: Json<Vec<OrderCancellationTermModel>>",
           bpp_terms as "bpp_terms?: Json<CommerceBppTermsModel>", documents as "documents?: Json<Vec<CommerceDocumentModel>>"
        FROM commerce_data where external_urn= $1;"#,
        transaction_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context(
            "A database failure occurred while saving fetching commerce data from database",
        )
    })?;

    Ok(record)
}

#[tracing::instrument(name = "fetch buyer commerce data line", skip(pool))]
async fn get_commerce_data_line(
    pool: &PgPool,
    order_id: Uuid,
) -> Result<Vec<CommerceItemModel>, anyhow::Error> {
    let records = sqlx::query_as!(
        CommerceItemModel,
        r#"
        SELECT 
            id, 
            item_id, 
            commerce_data_id, 
            item_name, 
            item_code, 
            item_image, 
            qty, 
            packaging_req, 
            item_req,
            tax_rate, 
            tax_value, 
            unit_price, 
            gross_total, 
            available_qty, 
            discount_amount, 
            location_ids as "location_ids?: Json<Vec<String>>", 
            fulfillment_ids as "fulfillment_ids?: Json<Vec<String>>"
        FROM commerce_data_line 
        WHERE commerce_data_id = $1
        "#,
        order_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: at buyer commerce line{:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while fetching commerce data line from database")
    })?;

    Ok(records)
}

#[tracing::instrument(name = "fetch buyer commerce payments", skip(pool))]
async fn get_commerce_payments(
    pool: &PgPool,
    order_id: Uuid,
) -> Result<Vec<CommercePaymentModel>, anyhow::Error> {
    let records = sqlx::query_as!(
        CommercePaymentModel,
        r#"
        SELECT 
            id, 
            collected_by as "collected_by?: PaymentCollectedBy",
            payment_type as "payment_type!: PaymentType", 
            commerce_data_id,
            seller_payment_detail as "seller_payment_detail?: Json<SellerPaymentDetailModel>",
            buyer_fee_type  as "buyer_fee_type?: FeeType",
            buyer_fee_amount,
            settlement_window,
            settlement_basis as "settlement_basis?: SettlementBasis",
            withholding_amount,
            settlement_details as "settlement_details?: Json<Vec<PaymentSettlementDetailModel>>"
        FROM commerce_payment_data 
        WHERE commerce_data_id = $1
        "#,
        order_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to execute query while fetching commerce data payment: {:?}",
            e
        );
        anyhow::Error::new(e).context(
            "A database failure occurred while fetching commerce data payment from database",
        )
    })?;

    Ok(records)
}

#[tracing::instrument(name = "fetch buyer commerce fulfillments", skip(pool))]
async fn get_commerce_fulfillments(
    pool: &PgPool,
    order_id: Uuid,
) -> Result<Vec<CommerceFulfillmentModel>, anyhow::Error> {
    let records = sqlx::query_as!(
        CommerceFulfillmentModel,
        r#"
        SELECT 
            id,
            commerce_data_id,
            fulfillment_id,
            tat,
            fulfillment_type as "fulfillment_type: FulfillmentType",
            fulfillment_status as "fulfillment_status: FulfillmentStatusType",
            inco_terms as "inco_terms?: IncoTermType",
            place_of_delivery,
            provider_name,
            category as "category?: FulfillmentCategoryType",
            servicable_status as "servicable_status!: ServiceableType", 
            drop_off_data as "drop_off_data!:  Json<Option<DropOffDataModel>>",
            pickup_data as "pickup_data!:  Json<PickUpDataModel>",
            tracking,
            packaging_charge,
            delivery_charge,
            convenience_fee,
            trade_type as "trade_type?: TradeType"
        FROM commerce_fulfillment_data 
        WHERE commerce_data_id = $1
        "#,
        order_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to execute query for fetch buyer commerce fulfillment fetch: {:?}",
            e
        );
        anyhow::Error::new(e).context(
            "A database failure occurred while fetching commerce data fulfillments from database",
        )
    })?;

    Ok(records)
}

fn get_order_payment_from_model(payments: Vec<CommercePaymentModel>) -> Vec<CommercePayment> {
    let mut payment_obj = vec![];
    for payment in payments {
        let mut settlement_details_list = vec![];
        if let Some(settlement_details) = payment.settlement_details {
            for settlement_model in settlement_details.0 {
                settlement_details_list.push(PaymentSettlementDetail {
                    settlement_counterparty: settlement_model.settlement_counterparty,
                    settlement_phase: settlement_model.settlement_phase,
                    settlement_type: settlement_model.settlement_type,
                    settlement_bank_account_no: settlement_model.settlement_bank_account_no,
                    settlement_ifsc_code: settlement_model.settlement_ifsc_code,
                    beneficiary_name: settlement_model.beneficiary_name,
                    bank_name: settlement_model.bank_name,
                })
            }
        }
        let seller_payment_detail =
            if let Some(seller_payment_detail_obj) = payment.seller_payment_detail {
                Some(SellerPaymentDetail {
                    uri: seller_payment_detail_obj.uri.clone(),
                    ttl: seller_payment_detail_obj.ttl.clone(),
                    dsa: seller_payment_detail_obj.dsa.clone(),
                    signature: seller_payment_detail_obj.signature.clone(),
                })
            } else {
                None
            };

        payment_obj.push(CommercePayment {
            id: payment.id,
            collected_by: payment.collected_by,
            payment_type: payment.payment_type,
            seller_payment_detail,
            buyer_fee_type: payment.buyer_fee_type,
            buyer_fee_amount: payment.buyer_fee_amount.map(|v| v.to_string()),
            settlement_basis: payment.settlement_basis,
            settlement_window: payment.settlement_window,
            withholding_amount: payment.withholding_amount.map(|v| v.to_string()),
            settlement_details: Some(settlement_details_list),
        })
    }
    payment_obj
}

fn get_order_items_from_model(items: Vec<CommerceItemModel>) -> Vec<CommerceItem> {
    let mut item_obj = vec![];
    for item in items {
        let buyer_term = if item.item_req.is_some() && item.packaging_req.is_some() {
            Some(BuyerTerm {
                item_req: item.item_req.unwrap(),
                packaging_req: item.packaging_req.unwrap(),
            })
        } else {
            None
        };
        let location_ids = item
            .location_ids
            .map(|json| json.0)
            .unwrap_or_else(Vec::new);
        let fulfillment_ids = item
            .fulfillment_ids
            .map(|json| json.0)
            .unwrap_or_else(Vec::new);
        item_obj.push(CommerceItem {
            id: item.id,
            item_id: item.item_id,
            item_name: item.item_name,
            item_code: item.item_code,
            item_image: item.item_image,
            qty: item.qty,
            buyer_terms: buyer_term,
            tax_rate: item.tax_rate,
            tax_value: item.tax_value,
            unit_price: item.unit_price,
            gross_total: item.gross_total,
            available_qty: item.available_qty,
            discount_amount: item.discount_amount,
            location_ids,
            fulfillment_ids,
        })
    }
    item_obj
}

fn get_drop_off_from_model(drop_off: DropOffDataModel) -> DropOffData {
    DropOffData {
        location: FulfillmentLocation {
            gps: drop_off.location.gps,
            area_code: drop_off.location.area_code,
            address: drop_off.location.address,
            city: drop_off.location.city,
            country: drop_off.location.country,
            state: drop_off.location.state,
        },
        contact: FulfillmentContact {
            mobile_no: drop_off.contact.mobile_no,
            email: drop_off.contact.email,
        },
    }
}

fn get_pick_up_from_model(pick_up: PickUpDataModel) -> PickUpData {
    PickUpData {
        location: PickUpFulfillmentLocation {
            name: None,
            gps: pick_up.location.gps,
            area_code: pick_up.location.area_code,
            address: pick_up.location.address,
            city: pick_up.location.city,
            country: pick_up.location.country,
            state: pick_up.location.state,
        },
        contact: FulfillmentContact {
            mobile_no: pick_up.contact.mobile_no,
            email: pick_up.contact.email,
        },
        time_range: pick_up.time_range.map(|time_range_model| TimeRange {
            start: time_range_model.start,
            end: time_range_model.end,
        }),
    }
}

fn get_order_fulfillment_from_model(
    fulfillments: Vec<CommerceFulfillmentModel>,
) -> Vec<CommerceFulfillment> {
    let mut fulfillment_obj = vec![];
    for fulfillment in fulfillments {
        let delivery_term =
            if fulfillment.inco_terms.is_some() && fulfillment.place_of_delivery.is_some() {
                Some(DeliveryTerm {
                    inco_terms: fulfillment.inco_terms.unwrap(),
                    place_of_delivery: fulfillment.place_of_delivery.unwrap(),
                })
            } else {
                None
            };
        fulfillment_obj.push(CommerceFulfillment {
            id: fulfillment.id,
            fulfillment_id: fulfillment.fulfillment_id,
            fulfillment_type: fulfillment.fulfillment_type,
            tat: fulfillment.tat,
            fulfillment_status: fulfillment.fulfillment_status,
            delivery_term,
            provider_name: fulfillment.provider_name,
            category: fulfillment.category,
            servicable_status: fulfillment.servicable_status,
            tracking: fulfillment.tracking,
            drop_off: fulfillment
                .drop_off_data
                .as_ref()
                .clone()
                .map(get_drop_off_from_model),
            pickup: get_pick_up_from_model(fulfillment.pickup_data.0),
            // .as_ref()
            // .clone()
            // .map(get_pick_up_from_model),
            packaging_charge: fulfillment.packaging_charge,
            delivery_charge: fulfillment.delivery_charge,
            convenience_fee: fulfillment.convenience_fee,
        })
    }
    fulfillment_obj
}

fn get_cancelletion_term_from_model(
    cancellation_term_models: Vec<OrderCancellationTermModel>,
) -> Vec<CommerceCancellationTerm> {
    let mut cancellation_term_objs = vec![];
    for cancellation_term_model in cancellation_term_models {
        cancellation_term_objs.push(CommerceCancellationTerm {
            fulfillment_state: cancellation_term_model.fulfillment_state.clone(),
            reason_required: cancellation_term_model.reason_required,
            cancellation_fee: CommerceCancellationFee {
                r#type: cancellation_term_model.cancellation_fee.r#type.clone(),
                val: cancellation_term_model.cancellation_fee.val.clone(),
            },
        });
    }

    cancellation_term_objs
}

#[tracing::instrument(name = "billing model to struct", skip())]
fn get_order_billing_from_model(billing: &OrderBillingModel) -> CommerceBilling {
    CommerceBilling {
        name: billing.name.clone(),
        address: billing.address.clone(),
        state: billing.state.clone(),
        city: billing.city.clone(),
        tax_id: billing.tax_id.clone(),
        email: billing.email.clone(),
        phone: billing.phone.clone(),
    }
}

fn get_bpp_terms_from_model(bpp_model: CommerceBppTermsModel) -> CommerceBPPTerms {
    CommerceBPPTerms {
        max_liability: bpp_model.max_liability,
        max_liability_cap: bpp_model.max_liability_cap,
        mandatory_arbitration: bpp_model.mandatory_arbitration,
        court_jurisdiction: bpp_model.court_jurisdiction,
        delay_interest: bpp_model.delay_interest,
    }
}

fn get_document_from_document_model(
    documents: Vec<CommerceDocumentModel>,
) -> Vec<CommerceDocument> {
    let mut document_list = vec![];
    for document in documents {
        document_list.push(CommerceDocument {
            r#type: document.r#type,
            url: document.url,
        })
    }
    document_list
}

#[tracing::instrument(name = "model to struct", skip())]
fn get_order_from_model(
    order: CommerceDataModel,
    lines: Vec<CommerceItemModel>,
    payments: Vec<CommercePaymentModel>,
    fulfillments: Vec<CommerceFulfillmentModel>,
) -> Commerce {
    let cancelletion_model_obj = order
        .cancellation_terms
        .map(|e| get_cancelletion_term_from_model(e.0));
    Commerce {
        id: order.id,
        urn: order.urn,
        external_urn: order.external_urn,
        record_type: order.record_type,
        record_status: order.record_status,
        domain_category_code: order.domain_category_code,
        seller: CommerceSeller {
            id: order.seller_id,
            name: order.seller_name,
        },
        source: order.source,
        created_on: order.created_on,
        updated_on: order.updated_on,
        created_by: order.created_by,
        grand_total: order.grand_total,
        bap: BasicNetworkData {
            id: order.bap_id,
            uri: order.bap_uri,
        },
        bpp: BasicNetworkData {
            id: order.bpp_id,
            uri: order.bpp_uri,
        },
        quote_ttl: order.quote_ttl,
        city_code: order.city_code,
        country_code: order.country_code,
        payments: get_order_payment_from_model(payments),
        items: get_order_items_from_model(lines),
        fulfillments: get_order_fulfillment_from_model(fulfillments),
        billing: order
            .billing
            .as_ref()
            .map(|billing| get_order_billing_from_model(billing)),
        cancellation_terms: cancelletion_model_obj,
        currency_type: order.currency_code,
        bpp_terms: order
            .bpp_terms
            .map(|term_model| get_bpp_terms_from_model(term_model.0)),
        documents: order
            .documents
            .map(|f| get_document_from_document_model(f.0)),
    }
}

#[tracing::instrument(name = "fetch order", skip(pool))]
pub async fn fetch_order_by_id(
    pool: &PgPool,
    transaction_id: Uuid,
) -> Result<Option<Commerce>, anyhow::Error> {
    if let Some(order_data) = get_commerce_data(pool, transaction_id).await? {
        let lines = get_commerce_data_line(pool, order_data.id).await?;
        //let payments_2 = get_commerce_payments_2(pool, &order_data.id).await?;
        let payments = get_commerce_payments(pool, order_data.id).await?;

        let fulfillmets = get_commerce_fulfillments(pool, order_data.id).await?;
        Ok(Some(get_order_from_model(
            order_data,
            lines,
            payments,
            fulfillmets,
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(name = "delete payment", skip(transaction))]
async fn delete_payment_in_commerce(
    transaction: &mut Transaction<'_, Postgres>,
    transaction_id: Uuid,
) -> Result<Uuid, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM commerce_payment_data USING commerce_data 
        WHERE commerce_payment_data.commerce_data_id = commerce_data.id 
        AND commerce_data.external_urn = $1 
        RETURNING commerce_data.id AS bc_id;
        "#,
        transaction_id // Pass the parameter here
    );

    let result = query.fetch_one(&mut **transaction).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(result.bc_id)
}

#[tracing::instrument(name = "save payment on on_init", skip(transaction))]
pub async fn initialize_payment_on_init(
    transaction: &mut Transaction<'_, Postgres>,
    commerce_id: Uuid,
    payments: &Vec<ONDCOnInitPayment>,
) -> Result<(), anyhow::Error> {
    let mut id_list = vec![];
    let mut commerce_data_id_list = vec![];
    let mut collected_by_list = vec![];
    let mut payment_type_list = vec![];
    let mut buyer_fee_type_list = vec![];
    let mut buyer_fee_amount_list = vec![];
    let mut settlement_window_list = vec![];
    let mut withholding_amount_list = vec![];
    let mut settlement_basis_list = vec![];

    let mut settlement_detail_list = vec![];
    let mut seller_payment_detail_list = vec![];
    for payment in payments {
        id_list.push(Uuid::new_v4());
        commerce_data_id_list.push(commerce_id);
        collected_by_list.push(payment.collected_by.get_type());
        payment_type_list.push(payment.r#type.get_payment());
        buyer_fee_type_list.push(&payment.buyer_app_finder_fee_type);
        buyer_fee_amount_list
            .push(BigDecimal::from_str(&payment.buyer_app_finder_fee_amount).unwrap());
        settlement_window_list.push(payment.settlement_window.as_deref());

        withholding_amount_list.push(
            payment
                .withholding_amount
                .clone()
                .map(|e| BigDecimal::from_str(&e).unwrap_or(BigDecimal::from(0)))
                .unwrap_or(BigDecimal::from(0)),
        );
        // seller_payment_uri_list.push(payment.uri.as_deref());
        settlement_basis_list.push(
            payment
                .settlement_basis
                .as_ref()
                .map(|e| e.get_settlement_basis_from_ondc_type()),
        );
        if payment.r#type == ONDCPaymentType::OnFulfillment
            && payment.collected_by == ONDCPaymentCollectedBy::Bpp
        {
            // let payment_uri = &payment.uri.unwrap();
            let seller_payment_ttl = payment.tags.as_ref().map(|tag| {
                get_tag_value_from_list(
                    tag,
                    ONDCTagType::BPPPayment,
                    &ONDCTagItemCode::Ttl.to_string(),
                )
                .unwrap_or_default()
            });
            let seller_payment_dsa = payment.tags.as_ref().map(|tag| {
                get_tag_value_from_list(
                    tag,
                    ONDCTagType::BPPPayment,
                    &ONDCTagItemCode::Dsa.to_string(),
                )
                .unwrap_or_default()
            });
            let seller_payment_signature = payment.tags.as_ref().map(|tag| {
                get_tag_value_from_list(
                    tag,
                    ONDCTagType::BPPPayment,
                    &ONDCTagItemCode::Signature.to_string(),
                )
                .unwrap_or_default()
            });
            let seller_payment_detail = SellerPaymentDetailModel {
                uri: payment.uri.to_owned().unwrap(),
                ttl: seller_payment_ttl.map(|s| s.to_owned()),
                dsa: seller_payment_dsa.map(|s| s.to_owned()),
                signature: seller_payment_signature.map(|s| s.to_owned()),
            };
            seller_payment_detail_list
                .push(Some(serde_json::to_value(seller_payment_detail).unwrap()));
        } else {
            seller_payment_detail_list.push(None);
        }
        if let Some(settlement_details) = &payment.settlement_details {
            let settlement_details: Vec<PaymentSettlementDetailModel> = settlement_details
                .iter()
                .map(|e| e.to_payment_settlement_detail())
                .collect::<Vec<PaymentSettlementDetailModel>>();
            settlement_detail_list.push(Some(serde_json::to_value(settlement_details).unwrap()));
        } else {
            settlement_detail_list.push(None);
        }
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_payment_data(id, commerce_data_id, collected_by, payment_type, buyer_fee_type,
             buyer_fee_amount, settlement_window, withholding_amount, settlement_basis, settlement_details, seller_payment_detail)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::payment_collected_by_type[],
            $4::payment_type[], $5::ondc_np_fee_type[], $6::decimal[], $7::text[], $8::decimal[],
            $9::settlement_basis_type[],$10::jsonb[], $11::jsonb[])
        "#,
        &id_list[..] as &[Uuid],
        &commerce_data_id_list[..] as &[Uuid],
        &collected_by_list[..] as &[PaymentCollectedBy],
        &payment_type_list[..] as &[PaymentType],
        &buyer_fee_type_list[..] as &[&FeeType],
        &buyer_fee_amount_list[..] as &[BigDecimal],
        &settlement_window_list[..] as &[Option<&str>],
        &withholding_amount_list[..] as &[BigDecimal],
        &settlement_basis_list[..] as &[Option<SettlementBasis>],
        &settlement_detail_list[..] as &[Option<Value>],
        &seller_payment_detail_list[..] as &[Option<Value>]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

fn convert_ondc_billing_to_model_billing(billing: &ONDCBilling) -> OrderBillingModel {
    OrderBillingModel {
        name: billing.name.clone(),
        address: billing.address.clone(),
        state: billing.state.name.clone(),
        city: billing.city.name.clone(),
        tax_id: billing.tax_id.clone(),
        email: billing.email.clone(),
        phone: billing.phone.clone(),
    }
}

pub fn get_bpp_term_model_from_tag(tags: &[ONDCTag]) -> CommerceBppTermsModel {
    let max_liability = get_tag_value_from_list(
        tags,
        ONDCTagType::BppTerms,
        &ONDCTagItemCode::MaxLiability.to_string(),
    )
    .unwrap_or_default()
    .to_owned();

    let max_liability_cap = get_tag_value_from_list(
        tags,
        ONDCTagType::BppTerms,
        &ONDCTagItemCode::MaxLiabilityCap.to_string(),
    )
    .unwrap_or_default()
    .to_owned();

    let mandatory_arbitration = get_tag_value_from_list(
        tags,
        ONDCTagType::BppTerms,
        &ONDCTagItemCode::MandatoryArbitration.to_string(),
    )
    .unwrap_or_default()
        == "false";

    let court_jurisdiction = get_tag_value_from_list(
        tags,
        ONDCTagType::BppTerms,
        &ONDCTagItemCode::CourtJurisdiction.to_string(),
    )
    .unwrap_or_default()
    .to_owned();
    let delay_interest = get_tag_value_from_list(
        tags,
        ONDCTagType::BppTerms,
        &ONDCTagItemCode::DelayInterest.to_string(),
    )
    .unwrap_or_default()
    .to_owned();
    CommerceBppTermsModel {
        max_liability,
        max_liability_cap,
        mandatory_arbitration,
        court_jurisdiction,
        delay_interest,
    }
}

pub fn get_cancel_term_model_from_ondc_cancel_term(
    cancellation_terms: &Vec<ONDCOrderCancellationTerm>,
) -> Vec<OrderCancellationTermModel> {
    let mut cancelletion_list = vec![];
    for cancellation_term in cancellation_terms {
        cancelletion_list.push(OrderCancellationTermModel {
            fulfillment_state: cancellation_term
                .fulfillment_state
                .descriptor
                .code
                .get_fulfillment_state(),
            reason_required: cancellation_term.reason_required,
            cancellation_fee: OrderCancellationFeeModel {
                r#type: cancellation_term.cancellation_fee.get_type(),
                val: cancellation_term.cancellation_fee.get_amount(),
            },
        })
    }
    cancelletion_list
}

#[tracing::instrument(name = "update buyer commerce data on on_init", skip(transaction))]
async fn update_commerce_in_on_init(
    transaction: &mut Transaction<'_, Postgres>,
    on_init_request: &ONDCOnInitRequest,
) -> Result<(), anyhow::Error> {
    let billing = convert_ondc_billing_to_model_billing(&on_init_request.message.order.billing);
    let bpp_terms = get_bpp_term_model_from_tag(&on_init_request.message.order.tags);
    let cancellation_terms = get_cancel_term_model_from_ondc_cancel_term(
        &on_init_request.message.order.cancellation_terms,
    );

    let query = sqlx::query!(
        r#"
        UPDATE commerce_data SET billing=$1, bpp_terms=$2, record_status=$3, cancellation_terms=$4, updated_on=$5 WHERE external_urn=$6
        "#,
        serde_json::to_value(billing).unwrap(),
        serde_json::to_value(bpp_terms).unwrap(),
        CommerceStatusType::Initialized as CommerceStatusType,
        serde_json::to_value(cancellation_terms).unwrap(),
        &on_init_request.context.timestamp,
        on_init_request.context.transaction_id,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving on_init buyer commerce to database")
    })?;
    Ok(())
}

#[tracing::instrument(name = "save order on on_init", skip(pool))]
pub async fn initialize_order_on_init(
    pool: &PgPool,
    on_init_request: &ONDCOnInitRequest,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let commerce_id =
        delete_payment_in_commerce(&mut transaction, on_init_request.context.transaction_id)
            .await?;
    initialize_payment_on_init(
        &mut transaction,
        commerce_id,
        &on_init_request.message.order.payments,
    )
    .await?;
    update_commerce_in_on_init(&mut transaction, on_init_request).await?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to update order on init")?;
    Ok(())
}

#[tracing::instrument(name = "save buyer commerce on on_confirm", skip(transaction))]
async fn update_commerce_in_on_confirm(
    transaction: &mut Transaction<'_, Postgres>,
    order: &Commerce,
    confirm_req: &ONDCOnConfirmRequest,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE commerce_data SET record_status=$1, updated_on=$2, urn=$3 WHERE external_urn=$4
        "#,
        confirm_req
            .message
            .order
            .state
            .get_commerce_status(&order.record_type, None) as CommerceStatusType,
        confirm_req.message.order.updated_at,
        confirm_req.message.order.id,
        confirm_req.context.transaction_id,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context(
            "A database failure occurred while saving on_confirm buyer commerce to database",
        )
    })?;
    Ok(())
}

pub fn create_pickup_off_from_on_confirm_fulfullment(
    fulfillment: &ONDCConfirmFulfillmentStartLocation,
    contact: &ONDCContact,
    commerce_pickup_data: &PickUpData,
    time_rage: Option<&ONDCFulfillmentTime>,
    fulfillment_instruction: Option<&ONDCFulfillmentInstruction>,
) -> PickUpDataModel {
    let instruction: Option<FulfillmentInstruction> =
        fulfillment_instruction.map(|fulfillment_instruction| FulfillmentInstruction {
            short_desc: fulfillment_instruction.short_desc.clone(),
            name: fulfillment_instruction.name.clone(),
            images: fulfillment_instruction.images.clone(),
            long_desc: fulfillment_instruction.long_desc.clone(),
        });
    PickUpDataModel {
        location: PickUpLocationModel {
            gps: fulfillment.gps.clone(),
            area_code: fulfillment
                .area_code
                .clone()
                .unwrap_or(commerce_pickup_data.location.area_code.clone()),
            address: commerce_pickup_data.location.address.clone(),
            city: commerce_pickup_data.location.city.clone(),
            country: commerce_pickup_data.location.country.clone(),
            state: commerce_pickup_data.location.state.clone(),
        },
        contact: PickUpContactModel {
            mobile_no: contact.phone.clone(),
            email: contact.email.clone(),
        },
        time_range: time_rage.map(|e| TimeRangeModel {
            start: e.range.start,
            end: e.range.end,
        }),
        instruction,
    }
}

fn get_bulk_update_on_confirm_data(
    commerce_id: Uuid,
    confirm_fulfillments: &Vec<ONDCOnConfirmFulfillment>,
    order_fulfillments: &[CommerceFulfillment],
) -> BulkConfirmFulfillmentData {
    let mut fulfillment_statuses = vec![];
    let mut pickup_datas = vec![];
    let mut commerce_data_ids = vec![];
    let mut fulfillment_ids = vec![];
    let order_fulfillment_map: HashMap<String, &CommerceFulfillment> = order_fulfillments
        .iter()
        .map(|fulfillment| (fulfillment.fulfillment_id.clone(), fulfillment))
        .collect();

    for confirm_fulfillment in confirm_fulfillments {
        if let Some(order_fulfillment) = order_fulfillment_map.get(&confirm_fulfillment.id) {
            if let Some(ondc_start) = confirm_fulfillment.get_fulfillment_start() {
                let pick_up_data = create_pickup_off_from_on_confirm_fulfullment(
                    ondc_start,
                    confirm_fulfillment
                        .get_fulfillment_contact(ONDCFulfillmentStopType::Start)
                        .unwrap(),
                    &order_fulfillment.pickup,
                    confirm_fulfillment.get_fulfillment_time(ONDCFulfillmentStopType::Start),
                    confirm_fulfillment.get_fulfillment_instruction(ONDCFulfillmentStopType::Start),
                );
                fulfillment_statuses.push(
                    confirm_fulfillment
                        .state
                        .descriptor
                        .code
                        .get_fulfillment_state(),
                );

                pickup_datas.push(serde_json::to_value(pick_up_data).unwrap());
                fulfillment_ids.push(confirm_fulfillment.id.to_owned());
                commerce_data_ids.push(commerce_id);
            }
        }
    }

    BulkConfirmFulfillmentData {
        fulfillment_statuses,
        pickup_datas,
        commerce_data_ids,
        fulfillment_ids,
    }
}

// CHANGE THE FOR LOOP UPDATE TO BULK UPDATE
#[tracing::instrument(name = "save fulfillment  on on_confirm", skip(transaction))]
async fn update_commerce_fulfillment_in_on_confirm(
    transaction: &mut Transaction<'_, Postgres>,
    commerce_id: Uuid,
    confirm_fulfillments: &Vec<ONDCOnConfirmFulfillment>,
    order_fulfillments: &Vec<CommerceFulfillment>,
) -> Result<(), anyhow::Error> {
    let data =
        get_bulk_update_on_confirm_data(commerce_id, confirm_fulfillments, order_fulfillments);

    let query = sqlx::query!(
        r#"
        UPDATE commerce_fulfillment_data
        SET
        fulfillment_status = t.fulfillment_status,
        pickup_data  = t.pickup_data
        FROM UNNEST($1::uuid[],  $2::commerce_fulfillment_status_type[], $3::jsonb[],$4::text[])
        AS t(commerce_data_id, fulfillment_status, pickup_data, fulfillment_id)
        WHERE commerce_fulfillment_data.commerce_data_id = t.commerce_data_id and 
        commerce_fulfillment_data.fulfillment_id=t.fulfillment_id;
        "#,
        &data.commerce_data_ids,
        &data.fulfillment_statuses as &Vec<FulfillmentStatusType>,
        &data.pickup_datas,
        &data.fulfillment_ids,
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;

    Ok(())
}

#[tracing::instrument(name = "save payment on on_confirm", skip(transaction))]
pub async fn initialize_payment_on_confirm(
    transaction: &mut Transaction<'_, Postgres>,
    order: &Commerce,
    payments: &Vec<ONDCOnConfirmPayment>,
) -> Result<(), anyhow::Error> {
    let mut id_list = vec![];
    let mut commerce_data_id_list = vec![];
    let mut collected_by_list = vec![];
    let mut payment_type_list = vec![];
    let mut buyer_fee_type_list = vec![];
    let mut buyer_fee_amount_list = vec![];
    let mut settlement_window_list = vec![];
    let mut withholding_amount_list = vec![];
    let mut settlement_basis_list = vec![];
    let mut settlement_detail_list = vec![];
    let mut payment_statuses = vec![];
    let mut payment_transaction_ids = vec![];
    let mut payment_paid_amounts = vec![];
    let mut seller_payment_detail_list = vec![];
    let mut payment_id_list = vec![];
    for payment in payments {
        id_list.push(Uuid::new_v4());
        commerce_data_id_list.push(order.id);
        collected_by_list.push(payment.collected_by.get_type());
        payment_type_list.push(payment.r#type.get_payment());
        buyer_fee_type_list.push(&payment.buyer_app_finder_fee_type);
        buyer_fee_amount_list
            .push(BigDecimal::from_str(&payment.buyer_app_finder_fee_amount).unwrap());
        settlement_window_list.push(payment.settlement_window.as_str());
        withholding_amount_list.push(BigDecimal::from_str(&payment.withholding_amount).unwrap());
        payment_id_list.push(payment.id.as_deref());
        if payment.r#type == ONDCPaymentType::PreFulfillment
            && payment.collected_by == ONDCPaymentCollectedBy::Bpp
        {
            // let payment_uri = &payment.uri.unwrap();
            let seller_payment_ttl = payment.tags.as_ref().map(|tag| {
                get_tag_value_from_list(
                    tag,
                    ONDCTagType::BPPPayment,
                    &ONDCTagItemCode::Ttl.to_string(),
                )
                .unwrap_or_default()
            });
            let seller_payment_dsa = payment.tags.as_ref().map(|tag| {
                get_tag_value_from_list(
                    tag,
                    ONDCTagType::BPPPayment,
                    &ONDCTagItemCode::Dsa.to_string(),
                )
                .unwrap_or_default()
            });
            let seller_payment_signature = payment.tags.as_ref().map(|tag| {
                get_tag_value_from_list(
                    tag,
                    ONDCTagType::BPPPayment,
                    &ONDCTagItemCode::Signature.to_string(),
                )
                .unwrap_or_default()
            });
            let seller_payment_detail = SellerPaymentDetailModel {
                uri: payment.uri.to_owned().unwrap_or("".to_string()),
                ttl: seller_payment_ttl.map(|s| s.to_owned()),
                dsa: seller_payment_dsa.map(|s| s.to_owned()),
                signature: seller_payment_signature.map(|s| s.to_owned()),
            };
            seller_payment_detail_list
                .push(Some(serde_json::to_value(seller_payment_detail).unwrap()));
        } else {
            seller_payment_detail_list.push(None);
        }

        settlement_basis_list.push(
            payment
                .settlement_basis
                .get_settlement_basis_from_ondc_type(),
        );

        payment_transaction_ids.push(payment.params.transaction_id.as_deref());
        payment_paid_amounts.push(BigDecimal::from_str(&payment.params.amount).unwrap_or_default());
        payment_statuses.push(payment.status.get_payment_status());
        if let Some(settlement_details) = &payment.settlement_details {
            let settlement_details: Vec<PaymentSettlementDetailModel> = settlement_details
                .iter()
                .map(|e| e.to_payment_settlement_detail())
                .collect::<Vec<PaymentSettlementDetailModel>>();
            settlement_detail_list.push(Some(serde_json::to_value(settlement_details).unwrap()));
        } else {
            settlement_detail_list.push(None);
        }
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO commerce_payment_data(id, commerce_data_id, collected_by, payment_type, buyer_fee_type,
             buyer_fee_amount, settlement_window, withholding_amount, settlement_basis,
             settlement_details, transaction_id,payment_status, payment_amount, seller_payment_detail, payment_id)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::payment_collected_by_type[],
            $4::payment_type[], $5::ondc_np_fee_type[], $6::decimal[], $7::text[], $8::decimal[],
            $9::settlement_basis_type[],  $10::jsonb[],
            $11::text[], $12::payment_status[], $13::decimal[], $14::jsonb[], $15::text[])
        "#,
        &id_list[..] as &[Uuid],
        &commerce_data_id_list[..] as &[Uuid],
        &collected_by_list[..] as &[PaymentCollectedBy],
        &payment_type_list[..] as &[PaymentType],
        &buyer_fee_type_list[..] as &[&FeeType],
        &buyer_fee_amount_list[..] as &[BigDecimal],
        &settlement_window_list[..] as &[&str],
        &withholding_amount_list[..] as &[BigDecimal],
        &settlement_basis_list[..] as &[SettlementBasis],
        &settlement_detail_list[..] as &[Option<Value>],
        &payment_transaction_ids[..] as &[Option<&str>],
        &payment_statuses[..] as &[PaymentStatus],
        &payment_paid_amounts[..] as &[BigDecimal],
        &seller_payment_detail_list[..] as &[Option<Value>],
        &payment_id_list[..] as &[Option<&str>]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "save order on on_confirm", skip(pool))]
pub async fn initialize_order_on_confirm(
    pool: &PgPool,
    on_confirm_request: &ONDCOnConfirmRequest,
    order: &Commerce,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    update_commerce_in_on_confirm(&mut transaction, order, on_confirm_request).await?;
    update_commerce_fulfillment_in_on_confirm(
        &mut transaction,
        order.id,
        &on_confirm_request.message.order.fulfillments,
        &order.fulfillments,
    )
    .await?;
    let _ = delete_payment_in_commerce(&mut transaction, on_confirm_request.context.transaction_id)
        .await?;

    initialize_payment_on_confirm(
        &mut transaction,
        order,
        &on_confirm_request.message.order.payments,
    )
    .await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to update order on init")?;
    Ok(())
}

fn get_ondc_document(
    documents: &Option<Vec<CommerceDocument>>,
    ondc_documents: &Vec<ONDCDocument>,
) -> Vec<CommerceDocument> {
    let mut final_documents = vec![];

    let documents_map: HashMap<DocumentType, &CommerceDocument> = if let Some(docs) = documents {
        docs.iter().map(|doc| (doc.r#type.clone(), doc)).collect()
    } else {
        HashMap::new()
    };

    let mut included_types = HashSet::new();

    for ondc_document in ondc_documents {
        let doc_type = ondc_document.label.get_document_type();

        let commerce_document = if let Some(existing_doc) = documents_map.get(&doc_type) {
            (*existing_doc).clone()
        } else {
            CommerceDocument {
                r#type: doc_type.clone(),
                url: ondc_document.url.clone(),
            }
        };

        final_documents.push(commerce_document);
        included_types.insert(doc_type);
    }

    if let Some(docs) = documents {
        for doc in docs {
            if !included_types.contains(&doc.r#type) {
                final_documents.push(doc.clone());
            }
        }
    }

    final_documents
}
#[tracing::instrument(name = "save buyer commerce on on_status", skip(transaction))]
async fn update_commerce_in_on_status(
    transaction: &mut Transaction<'_, Postgres>,
    order: &Commerce,
    status_req: &ONDCOnStatusRequest,
) -> Result<(), anyhow::Error> {
    let document_type = status_req.message.order.documents.is_some();
    let query = sqlx::query!(
        r#"
        UPDATE commerce_data SET record_status=$1, updated_on=$2, documents=$3 WHERE external_urn=$4
        "#,
        status_req
            .message
            .order
            .state
            .get_commerce_status(&order.record_type, Some(document_type))
            as CommerceStatusType,
        status_req.message.order.updated_at,
        status_req
            .message
            .order
            .documents
            .as_ref()
            .map(|docs| serde_json::to_value(get_ondc_document(&order.documents, docs)).unwrap()),
        status_req.context.transaction_id,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context(
            "A database failure occurred while saving on_confirm buyer commerce to database",
        )
    })?;
    Ok(())
}

pub fn create_drop_off_from_on_status_fulfullment(
    fulfillment: &ONDCConfirmFulfillmentEndLocation,
    contact: &ONDCContact,
    commerce_drop_off_data: &DropOffData,
    time_rage: Option<&ONDCFulfillmentTime>,
    fulfillment_instruction: Option<&ONDCFulfillmentInstruction>,
) -> DropOffDataModel {
    let instruction =
        fulfillment_instruction.map(|fulfillment_instruction| FulfillmentInstruction {
            short_desc: fulfillment_instruction.short_desc.clone(),
            name: fulfillment_instruction.name.clone(),
            images: fulfillment_instruction.images.clone(),
            long_desc: fulfillment_instruction.long_desc.clone(),
        });
    DropOffDataModel {
        location: DropOffLocationModel {
            gps: fulfillment.gps.clone(),
            area_code: fulfillment.area_code.clone(),
            address: commerce_drop_off_data.location.address.clone(),
            city: commerce_drop_off_data.location.city.clone(),
            country: commerce_drop_off_data.location.country.clone(),
            state: commerce_drop_off_data.location.state.clone(),
        },
        contact: DropOffContactModel {
            mobile_no: contact.phone.clone(),
            email: contact.email.clone(),
        },
        time_range: time_rage.map(|e| TimeRangeModel {
            start: e.range.start,
            end: e.range.end,
        }),
        instruction,
    }
}

#[tracing::instrument(name = "create_pickup_off_from_on_status_fulfullment", skip())]
pub fn create_pickup_off_from_on_status_fulfullment(
    fulfillment: &ONDCConfirmFulfillmentStartLocation,
    contact: &ONDCContact,
    commerce_pickup_data: &PickUpData,
    time_rage: Option<&ONDCFulfillmentTime>,
    fulfillment_instruction: Option<&ONDCFulfillmentInstruction>,
) -> PickUpDataModel {
    let instruction =
        fulfillment_instruction.map(|fulfillment_instruction| FulfillmentInstruction {
            short_desc: fulfillment_instruction.short_desc.clone(),
            name: fulfillment_instruction.name.clone(),
            images: fulfillment_instruction.images.clone(),
            long_desc: fulfillment_instruction.long_desc.clone(),
        });
    PickUpDataModel {
        location: PickUpLocationModel {
            gps: fulfillment.gps.clone(),
            area_code: fulfillment
                .area_code
                .clone()
                .unwrap_or(commerce_pickup_data.location.area_code.clone()),
            address: commerce_pickup_data.location.address.clone(),
            city: commerce_pickup_data.location.city.clone(),
            country: commerce_pickup_data.location.country.clone(),
            state: commerce_pickup_data.location.state.clone(),
        },
        contact: PickUpContactModel {
            mobile_no: contact.phone.clone(),
            email: contact.email.clone(),
        },
        time_range: time_rage.map(|e| TimeRangeModel {
            start: e.range.start,
            end: e.range.end,
        }),
        instruction,
    }
}

fn get_bulk_update_on_status_data(
    commerce_id: Uuid,
    status_fulfillments: &Vec<ONDCOnConfirmFulfillment>,
    order_fulfillments: &[CommerceFulfillment],
) -> BulkStatusFulfillmentData {
    let mut fulfillment_statuses = vec![];
    let mut pickup_datas = vec![];
    let mut commerce_data_ids = vec![];
    let mut fulfillment_ids = vec![];
    let mut drop_off_datas = vec![];
    let order_fulfillment_map: HashMap<String, &CommerceFulfillment> = order_fulfillments
        .iter()
        .map(|fulfillment| (fulfillment.fulfillment_id.clone(), fulfillment))
        .collect();

    for status_fulfillment in status_fulfillments {
        if let Some(order_fulfillment) = order_fulfillment_map.get(&status_fulfillment.id) {
            if let Some(ondc_start) = status_fulfillment.get_fulfillment_start() {
                let pick_up_data = create_pickup_off_from_on_status_fulfullment(
                    ondc_start,
                    status_fulfillment
                        .get_fulfillment_contact(ONDCFulfillmentStopType::Start)
                        .unwrap(),
                    &order_fulfillment.pickup,
                    status_fulfillment.get_fulfillment_time(ONDCFulfillmentStopType::Start),
                    status_fulfillment.get_fulfillment_instruction(ONDCFulfillmentStopType::Start),
                );

                let drop_off_data = if let Some(order_drop_off) = &order_fulfillment.drop_off {
                    status_fulfillment.get_fulfillment_end().map(|ondc_end| {
                        create_drop_off_from_on_status_fulfullment(
                            ondc_end,
                            status_fulfillment
                                .get_fulfillment_contact(ONDCFulfillmentStopType::End)
                                .unwrap(),
                            order_drop_off,
                            status_fulfillment.get_fulfillment_time(ONDCFulfillmentStopType::End),
                            status_fulfillment
                                .get_fulfillment_instruction(ONDCFulfillmentStopType::End),
                        )
                    })
                } else {
                    None
                };
                fulfillment_statuses.push(
                    status_fulfillment
                        .state
                        .descriptor
                        .code
                        .get_fulfillment_state(),
                );

                pickup_datas.push(serde_json::to_value(pick_up_data).unwrap());
                drop_off_datas.push(serde_json::to_value(drop_off_data).unwrap());
                fulfillment_ids.push(status_fulfillment.id.to_owned());
                commerce_data_ids.push(commerce_id);
            }
        }
    }

    BulkStatusFulfillmentData {
        fulfillment_statuses,
        pickup_datas,
        commerce_data_ids,
        fulfillment_ids,
        drop_off_datas,
    }
}

// CHANGE THE FOR LOOP UPDATE TO BULK UPDATE
#[tracing::instrument(name = "save fulfillment  on on_status", skip(transaction))]
async fn update_commerce_fulfillment_in_on_status(
    transaction: &mut Transaction<'_, Postgres>,
    commerce_id: Uuid,
    status_fulfillments: &Vec<ONDCOnConfirmFulfillment>,
    order_fulfillments: &Vec<CommerceFulfillment>,
) -> Result<(), anyhow::Error> {
    let data = get_bulk_update_on_status_data(commerce_id, status_fulfillments, order_fulfillments);
    println!("{:?}", data.pickup_datas);
    println!("{:?}", data.commerce_data_ids);
    let query = sqlx::query!(
        r#"
        UPDATE commerce_fulfillment_data
        SET
        fulfillment_status = t.fulfillment_status,
        pickup_data  = t.pickup_data,
        drop_off_data = t.drop_off_data
        FROM UNNEST($1::uuid[],  $2::commerce_fulfillment_status_type[], $3::jsonb[], $4::jsonb[], $5::text[])
        AS t(commerce_data_id, fulfillment_status, pickup_data, drop_off_data, fulfillment_id)
        WHERE commerce_fulfillment_data.commerce_data_id = t.commerce_data_id and 
        commerce_fulfillment_data.fulfillment_id=t.fulfillment_id;
        "#,
        &data.commerce_data_ids,
        &data.fulfillment_statuses as &Vec<FulfillmentStatusType>,
        &data.pickup_datas,
        &data.drop_off_datas,
        &data.fulfillment_ids,
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;

    Ok(())
}

#[tracing::instrument(name = "save order on on_status", skip(pool))]
pub async fn initialize_order_on_status(
    pool: &PgPool,
    on_status_request: &ONDCOnStatusRequest,
    order: &Commerce,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    update_commerce_in_on_status(&mut transaction, order, on_status_request).await?;

    update_commerce_fulfillment_in_on_status(
        &mut transaction,
        order.id,
        &on_status_request.message.order.fulfillments,
        &order.fulfillments,
    )
    .await?;

    let _ = delete_payment_in_commerce(&mut transaction, on_status_request.context.transaction_id)
        .await?;

    initialize_payment_on_confirm(
        &mut transaction,
        order,
        &on_status_request.message.order.payments,
    )
    .await?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to update order on status")?;
    Ok(())
}

#[tracing::instrument(name = "save buyer commerce on on_cancel", skip(transaction))]
async fn update_commerce_in_on_cancel(
    transaction: &mut Transaction<'_, Postgres>,
    order: &Commerce,
    status_req: &ONDCOnCancelRequest,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE commerce_data SET record_status=$1, updated_on=$2, refund_grand_total=$3 WHERE external_urn=$4
        "#,
        status_req
            .message
            .order
            .state
            .get_commerce_status(&order.record_type, None) as CommerceStatusType,
        status_req.message.order.updated_at,
        BigDecimal::from_str(&status_req.message.order.quote.price.value)
            .unwrap_or(BigDecimal::from(0)),
        status_req.context.transaction_id,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context(
            "A database failure occurred while saving on_confirm buyer commerce to database",
        )
    })?;
    Ok(())
}

#[tracing::instrument(name = "bulk_update_on_cancel_fulfillments", skip(transaction))]
async fn update_on_cancel_fulfillments(
    transaction: &mut Transaction<'_, Postgres>,
    commerce_id: Uuid,
    request: &ONDCOnCancelRequest,
) -> Result<(), anyhow::Error> {
    let data = get_on_cancel_bulk_fulfillment_data(commerce_id, request);
    let query = sqlx::query!(
        r#"
        UPDATE commerce_fulfillment_data
        SET
            refunded_convenience_fee  = t.refunded_convenience_fee,
            refunded_delivery_charge = t.refunded_delivery_charge,
            refunded_packaging_charge = t.refunded_packaging_charge
        FROM UNNEST($1::uuid[], $2::text[], $3::decimal[], $4::decimal[], $5::decimal[]) AS t(id, fulfillment_id, 
            refunded_convenience_fee, refunded_delivery_charge, refunded_packaging_charge)
        WHERE commerce_data_id = t.id and commerce_fulfillment_data.fulfillment_id=t.fulfillment_id;
        "#,
        &data.commerce_ids,
        &data.fulfillment_ids,
        &data.refunded_convenience_fees,
        &data.refunded_delivery_charges,
        &data.refunded_packaging_charges,
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

fn get_on_cancel_bulk_fulfillment_data(
    commerce_id: Uuid,
    request: &ONDCOnCancelRequest,
) -> BulkCancelFulfillmentData {
    let mut commerce_ids = vec![];
    let mut refunded_convenience_fees = vec![];
    let mut fulfillment_ids = vec![];
    let mut refunded_delivery_charges = vec![];
    let mut refunded_packaging_charges = vec![];
    let item_breakup_mapping =
        get_quote_item_breakup_mapping_for_refund(&request.message.order.quote.breakup);
    for fulfillment in &request.message.order.fulfillments {
        commerce_ids.push(commerce_id);
        fulfillment_ids.push(fulfillment.id.to_owned());
        let convenience_fee = item_breakup_mapping
            .get(&format!(
                "{}_{}",
                ONDCTitleName::ConvenienceFee,
                fulfillment.id,
            ))
            .map(|e| BigDecimal::from_str(&e.price.value).unwrap_or(BigDecimal::from(0)))
            .unwrap_or(BigDecimal::from(0));
        let delivery_charge = item_breakup_mapping
            .get(&format!(
                "{}_{}",
                ONDCTitleName::DeliveryCharge,
                fulfillment.id,
            ))
            .map(|e| BigDecimal::from_str(&e.price.value).unwrap_or(BigDecimal::from(0)))
            .unwrap_or(BigDecimal::from(0));
        let packaging_charge = item_breakup_mapping
            .get(&format!("{}_{}", ONDCTitleName::Packing, fulfillment.id,))
            .map(|e| BigDecimal::from_str(&e.price.value).unwrap_or(BigDecimal::from(0)))
            .unwrap_or(BigDecimal::from(0));
        refunded_convenience_fees.push(convenience_fee);
        refunded_delivery_charges.push(delivery_charge);
        refunded_packaging_charges.push(packaging_charge);
    }

    BulkCancelFulfillmentData {
        commerce_ids,
        refunded_convenience_fees,
        fulfillment_ids,
        refunded_delivery_charges,
        refunded_packaging_charges,
    }
}
// save refund amount in commerce data and commerce line data and fulfillment

#[tracing::instrument(name = "save order on on_cancel", skip(pool))]
pub async fn initialize_order_on_cancel(
    pool: &PgPool,
    on_cancel_request: &ONDCOnCancelRequest,
    order: &Commerce,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    update_commerce_in_on_cancel(&mut transaction, order, on_cancel_request).await?;

    let _ = delete_payment_in_commerce(&mut transaction, on_cancel_request.context.transaction_id)
        .await?;

    initialize_payment_on_confirm(
        &mut transaction,
        order,
        &on_cancel_request.message.order.payments,
    )
    .await?;

    update_on_cancel_fulfillments(&mut transaction, order.id, on_cancel_request).await?;
    update_on_cancel_items(&mut transaction, order.id, on_cancel_request, order).await?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to update order on cancel")?;

    Ok(())
}

fn get_on_cancel_bulk_item_data(
    commerce_id: Uuid,
    request: &ONDCOnCancelRequest,
    order: &Commerce,
) -> BulkCancelItemData {
    let mut commerce_ids = vec![];
    let mut refunded_tax_values = vec![];
    let mut item_ids = vec![];
    let mut refunded_discount_amounts = vec![];
    let mut refunded_gross_totals = vec![];
    let item_breakup_mapping =
        get_quote_item_breakup_mapping_for_refund(&request.message.order.quote.breakup);
    let item_mapping: HashMap<&str, &CommerceItem> = order
        .items
        .iter()
        .map(|product| (product.item_id.as_str(), product))
        .collect();
    for item in &request.message.order.items {
        commerce_ids.push(commerce_id);
        item_ids.push(item.id.clone());

        let refunded_discount_amount = item_breakup_mapping
            .get(&format!("{}_{}", ONDCTitleName::Discount, item.id,))
            .map(|e| BigDecimal::from_str(&e.price.value).unwrap_or(BigDecimal::from(0)))
            .unwrap_or(BigDecimal::from(0));
        refunded_discount_amounts.push(refunded_discount_amount);

        let refunded_tax_value = item_breakup_mapping
            .get(&format!("{}_{}", ONDCTitleName::Tax, item.id))
            .map(|e| BigDecimal::from_str(&e.price.value).unwrap_or(BigDecimal::from(0)))
            .unwrap_or(BigDecimal::from(0));
        refunded_tax_values.push(refunded_tax_value);
        let item_name = item_mapping
            .get(item.id.as_str())
            .map(|e| &e.item_name)
            .unwrap_or(&item.id);

        let refund_gross_total = item_breakup_mapping
            .get(&format!("{}_{}", item_name, item.id,))
            .map(|e| BigDecimal::from_str(&e.price.value).unwrap_or(BigDecimal::from(0)))
            .unwrap_or(BigDecimal::from(0));
        refunded_gross_totals.push(refund_gross_total);
    }

    BulkCancelItemData {
        commerce_ids,
        item_ids,
        refunded_discount_amounts,
        refunded_gross_totals,
        refunded_tax_values,
    }
}
#[tracing::instrument(name = "bulk_update_on_cancel_items", skip(transaction))]
async fn update_on_cancel_items(
    transaction: &mut Transaction<'_, Postgres>,
    commerce_id: Uuid,
    request: &ONDCOnCancelRequest,
    order: &Commerce,
) -> Result<(), anyhow::Error> {
    let data = get_on_cancel_bulk_item_data(commerce_id, request, order);
    let query = sqlx::query!(
        r#"
        UPDATE commerce_data_line
        SET
            refunded_tax_value = t.refunded_tax_value,
            refunded_discount_amount  = t.refunded_discount_amount,
            refunded_gross_total = t.refunded_gross_total
    
        FROM UNNEST($1::uuid[], $2::text[], $3::decimal[], $4::decimal[], $5::decimal[]) AS t(id, item_id,
             refunded_discount_amount, refunded_gross_total, refunded_tax_value)
        WHERE commerce_data_id = t.id and commerce_data_line.item_id=t.item_id;
        "#,
        &data.commerce_ids,
        &data.item_ids,
        &data.refunded_discount_amounts,
        &data.refunded_gross_totals,
        &data.refunded_tax_values,
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "save order on on_update", skip(pool))]
pub async fn initialize_order_on_update(
    pool: &PgPool,
    on_cancel_request: &ONDCOnUpdateRequest,
    order: &Commerce,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let _ = delete_payment_in_commerce(&mut transaction, on_cancel_request.context.transaction_id)
        .await?;

    initialize_payment_on_confirm(
        &mut transaction,
        order,
        &on_cancel_request.message.order.payments,
    )
    .await?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to update order on update")?;

    Ok(())
}
