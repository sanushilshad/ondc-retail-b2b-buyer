use crate::routes::ondc::buyer::schemas::{ONDCOnSelectRequest, SellerProductInfo};
use crate::routes::ondc::buyer::utils::{
    get_ondc_seller_mapping_key, get_ondc_seller_product_info_mapping,
};
use crate::routes::ondc::{LookupData, ONDCActionType};
use crate::routes::order::schemas::{CommerceStatusType, OrderType};
use crate::routes::product::schemas::PaymentType;
use crate::routes::user::schemas::{BusinessAccount, DataSource, UserAccount};
use crate::schemas::RequestMetaData;
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::Utc;
use serde_json::Value;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;
use validator::HasLen;

use super::schemas::OrderSelectRequest;

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

#[tracing::instrument(name = "save rfq", skip(transaction))]
pub async fn save_rfq_order(
    transaction: &mut Transaction<'_, Postgres>,
    select_request: &OrderSelectRequest,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    bpp_detail: &LookupData,
    domain_uri: &str,
    provider_name: &str,
) -> Result<Uuid, anyhow::Error> {
    let order_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO buyer_commerce_data (id, external_urn, record_type, record_status, 
        domain_category_code, buyer_id, seller_id, seller_name, buyer_name, source, created_at, created_by, bpp_id, bpp_uri, tsp_id, is_import, quote_ttl, payment_types)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        ON CONFLICT (external_urn) 
        DO UPDATE SET 
        domain_category_code = EXCLUDED.domain_category_code,
        seller_id = EXCLUDED.seller_id
        "#,
        order_id,
        &select_request.transaction_id,
        &select_request.order_type as &OrderType,
        CommerceStatusType::QuoteRequested as CommerceStatusType,
        &select_request.domain_category_code.to_string(),
        &business_account.id,
        &select_request.provider_id,
        &provider_name,
        &business_account.company_name,
        DataSource::PlaceOrder as DataSource,
        Utc::now(),
        &user_account.id,
        &select_request.bpp_id,
        bpp_detail.subscriber_url,
        domain_uri,
        &select_request.is_import,
        &select_request.ttl,
        &select_request.payment_types as &[PaymentType]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;
    Ok(order_id)
}

#[tracing::instrument(name = "save rfq fulfillment", skip(_transaction))]
pub async fn save_rfq_fulfillment(
    _transaction: &mut Transaction<'_, Postgres>,
    select_request: &OrderSelectRequest,
) -> Result<(), anyhow::Error> {
    Ok(())
}

#[tracing::instrument(name = "save rfq items", skip(transaction))]
pub async fn save_order_select_items(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: &Uuid,
    select_request: &OrderSelectRequest,
    product_map: &HashMap<String, SellerProductInfo>,
) -> Result<(), anyhow::Error> {
    let item_count = select_request.items.length();
    let line_id_list: Vec<Uuid> = (0..item_count).map(|_| Uuid::new_v4()).collect();
    let order_id_list: Vec<Uuid> = vec![*order_id; item_count as usize];
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
    for item in &select_request.items {
        let key = get_ondc_seller_mapping_key(
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
    }
    let query = sqlx::query!(
        r#"
        INSERT INTO buyer_commerce_data_line (id, commerce_data_id, item_id, item_name, item_code, item_image, 
            qty, location_ids, fulfillment_ids, tax_rate, mrp, unit_price)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::text[], $4::text[], $5::text[], $6::text[],
             $7::decimal[], $8::jsonb[], $9::jsonb[], $10::decimal[], $11::decimal[], $12::decimal[])
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
    id: &Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query(
        r#"
        DELETE FROM buyer_commerce_data
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
#[tracing::instrument(name = "save request for quote", skip(pool))]
pub async fn initialize_order_select(
    pool: &PgPool,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    select_request: &OrderSelectRequest,
    tsp_id: &str,
    bpp_detail: &LookupData,
) -> Result<(), anyhow::Error> {
    let item_code_list: Vec<&str> = select_request
        .items
        .iter()
        .map(|item| item.item_id.as_str()) // Assuming item_id is a String
        .collect();
    let seller_product_map = get_ondc_seller_product_info_mapping(
        pool,
        &bpp_detail.subscriber_id,
        &select_request.provider_id,
        &item_code_list,
    )
    .await?;
    let provider_name = seller_product_map
        .values()
        .next()
        .and_then(|obj| obj.provider_name.as_deref())
        .unwrap_or_default();
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    delete_order(&mut transaction, &select_request.transaction_id).await?;

    let order_id = save_rfq_order(
        &mut transaction,
        select_request,
        user_account,
        business_account,
        bpp_detail,
        tsp_id,
        provider_name,
    )
    .await?;

    save_order_select_items(
        &mut transaction,
        &order_id,
        select_request,
        &seller_product_map,
    )
    .await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a order")?;

    Ok(())
}

// #[tracing::instrument(name = "save order on select", skip(transaction))]
// pub async fn save_order_on_select(
//     transaction: &mut Transaction<'_, Postgres>,
//     on_select_req: &ONDCOnSelectRequest,
//     user_account: &UserAccount,
//     business_account: &BusinessAccount,
//     bpp_detail: &LookupData,
//     domain_uri: &str,
//     provider_name: &str,
// ) -> Result<Uuid, anyhow::Error> {
//     let order_id = Uuid::new_v4();
//     // let payment_type = on_select_req.message.order.payments.iter()
//     let order_type
//     let query = sqlx::query!(
//         r#"
//         INSERT INTO buyer_commerce_data (id, external_urn, record_type, record_status,
//         domain_category_code, buyer_id, seller_id, seller_name, buyer_name, source, created_at, created_by, bpp_id, bpp_uri, tsp_id, is_import, quote_ttl, payment_types)
//         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
//         ON CONFLICT (external_urn)
//         DO UPDATE SET
//         domain_category_code = EXCLUDED.domain_category_code,
//         seller_id = EXCLUDED.seller_id
//         "#,
//         order_id,
//         &on_select_req.context.transaction_id,
//         &order_type as &OrderType,
//         CommerceStatusType::QuoteAccepted as CommerceStatusType,
//         &on_select_req.context.domain.get_category_domain(),
//         &business_account.id,
//         &on_select_req.message.order.provider.id,
//         &provider_name,
//         &business_account.company_name,
//         DataSource::PlaceOrder as DataSource,
//         Utc::now(),
//         &user_account.id,
//         &bpp_detail.subscriber_id,
//         bpp_detail.subscriber_url,
//         domain_uri,
//         &select_request.is_import,
//         &on_select_req.context.ttl,
//         &select_request.payment_types as &[PaymentType]
//     );

//     transaction.execute(query).await.map_err(|e| {
//         tracing::error!("Failed to execute query: {:?}", e);
//         anyhow::Error::new(e)
//             .context("A database failure occurred while saving RFQ to database request")
//     })?;
//     Ok(order_id)
// }

#[tracing::instrument(name = "save request for quote", skip(pool))]
pub async fn initialize_order_on_select(
    pool: &PgPool,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    on_select_request: &ONDCOnSelectRequest,
    tsp_id: &str,
    bpp_detail: &LookupData,
) -> Result<(), anyhow::Error> {
    // let item_code_list: Vec<&str> = select_request
    //     .items
    //     .iter()
    //     .map(|item| item.item_id.as_str()) // Assuming item_id is a String
    //     .collect();
    // let seller_product_map = get_ondc_seller_product_info_mapping(
    //     &pool,
    //     &bpp_detail.subscriber_id,
    //     &select_request.provider_id,
    //     &item_code_list,
    // )
    // .await?;
    // let provider_name = seller_product_map
    //     .values()
    //     .next()
    //     .and_then(|obj| obj.provider_name.as_deref())
    //     .unwrap_or_default();

    // delete_order(&mut transaction, &select_request.transaction_id).await?;

    // let order_id = save_rfq_order(
    //     &mut transaction,
    //     select_request,
    //     user_account,
    //     business_account,
    //     bpp_detail,
    //     tsp_id,
    //     &provider_name,
    // )
    // .await?;

    // save_order_select_items(
    //     &mut transaction,
    //     &order_id,
    //     &select_request,
    //     &seller_product_map,
    // )
    // .await?;

    let transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a order")?;

    Ok(())
}
