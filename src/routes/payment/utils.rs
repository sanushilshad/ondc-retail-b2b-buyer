use super::errors::PaymentOrderError;
use super::models::CommercePaymentMetaModel;
use super::schemas::{CommercePaymentMetaData, PaymentOrderData};

use crate::payment_client::PaymentClient;
use crate::routes::order::schemas::{
    CommerceStatusType, MinimalCommerceData, OrderType, PaymentCollectedBy, PaymentStatus,
};
use crate::routes::order::utils::update_order_update_field;
use crate::routes::product::schemas::PaymentType;
use crate::schemas::WebSocketParam;
use crate::user_client::{BusinessAccount, SettingKey, UserAccount, UserClient};
use anyhow::{anyhow, Context};
use chrono::Utc;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;
pub fn validate_order_for_payment(order: &MinimalCommerceData) -> Result<(), anyhow::Error> {
    match order.record_type {
        OrderType::PurchaseOrder if order.record_status != CommerceStatusType::Created => {
            Err(anyhow!("Order is not created"))
        }
        OrderType::SaleOrder if order.record_status != CommerceStatusType::Initialized => {
            Err(anyhow!("Order is not initialized"))
        }
        _ => Ok(()),
    }
}

#[tracing::instrument(name = "fetch locked commerce payments", skip(transaction))]
pub async fn get_commerce_payments_with_lock(
    transaction: &mut Transaction<'_, Postgres>,
    order_id: Uuid,
) -> Result<CommercePaymentMetaModel, anyhow::Error> {
    let record = sqlx::query_as!(
        CommercePaymentMetaModel,
        r#"
        SELECT 
            id,
            collected_by as "collected_by?: PaymentCollectedBy",
            payment_type as "payment_type!: PaymentType", 
            payment_status as "payment_status!: PaymentStatus",
            payment_order_id
        FROM commerce_payment_data 
        WHERE commerce_data_id = $1
        AND collected_by != $2
        "#,
        order_id,
        PaymentCollectedBy::Buyer as PaymentCollectedBy
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to execute query while fetching commerce data payment with lock: {:?}",
            e
        );
        anyhow::Error::new(e).context(
            "A database failure occurred while fetching commerce data payment from database",
        )
    })?;

    Ok(record)
}

fn validate_payment_order_creation(payment: &CommercePaymentMetaData) -> Result<(), anyhow::Error> {
    match (
        &payment.payment_status,
        &payment.payment_type,
        &payment.collected_by,
    ) {
        (_, _, None) => return Err(anyhow!("Payment type is not defined")),
        (Some(PaymentStatus::Paid), PaymentType::PrePaid, Some(PaymentCollectedBy::Bap)) => {
            return Err(anyhow!("Payment is already completed"));
        }
        (Some(PaymentStatus::Pending), PaymentType::PrePaid, Some(PaymentCollectedBy::Bap)) => {
            return Err(anyhow!("Payment is waiting for successful status"));
        }
        (_, PaymentType::PrePaid, Some(PaymentCollectedBy::Bpp)) => {
            return Err(anyhow!("Payment is to be completed by seller"));
        }
        (_, _, _) => {}
    }

    Ok(())
}

pub async fn get_payment_order_id(
    pool: &PgPool,
    payment_client: &PaymentClient,
    user_client: &UserClient,
    order: &MinimalCommerceData,
    business_account: &BusinessAccount,
    user_account: &UserAccount,
) -> Result<PaymentOrderData, PaymentOrderError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")
        .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;

    let payment = get_commerce_payments_with_lock(&mut transaction, order.id)
        .await
        .map(|op| op.into_schema())
        .map_err(|e| {
            PaymentOrderError::DatabaseError("Failed to fetch order list".to_string(), e)
        })?;

    let setting_data = user_client
        .fetch_setting(
            user_account.id,
            business_account.id,
            vec![SettingKey::PaymentServiceId],
        )
        .await
        .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;

    let payment_service_id = setting_data
        .get_setting(SettingKey::PaymentServiceId)
        .ok_or_else(|| anyhow!("Payment Service id  is not configured"))?;
    if let Some(payment_order_id) = payment.payment_order_id {
        let payment_order = payment_client
            .fetch_payments_by_order_id(&payment_order_id, &payment_service_id)
            .await
            .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;
        let payment_status = payment_client.determine_final_payment_status(payment_order.as_ref());

        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store an order")
            .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;

        return Ok(PaymentOrderData {
            order_id: payment_order_id,
            status: payment_status,
        });
    }

    validate_payment_order_creation(&payment)
        .map_err(|e| PaymentOrderError::ValidationError(e.to_string()))?;
    let webhook = format!(
        "https://{}/payment/notification",
        &business_account.subscriber_id
    );
    let data = payment_client.generate_order_create_request(
        order.external_urn,
        &order.grand_total,
        &payment_service_id,
        &order.currency_code,
        &webhook,
    );
    let payment_order = payment_client
        .create_order(data)
        .await
        .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;
    update_order_update_field(
        &mut transaction,
        order.external_urn,
        &user_account.id.to_string(),
    )
    .await
    .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;
    update_payment_order_id(&mut transaction, payment.id, &payment_order.id)
        .await
        .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")
        .map_err(|e| PaymentOrderError::UnexpectedCustomError(e.to_string()))?;

    Ok(PaymentOrderData {
        order_id: payment_order.id,
        status: PaymentStatus::NotPaid,
    })
}

#[tracing::instrument(name = "update_payment_order_id", skip(transaction))]
async fn update_payment_order_id(
    transaction: &mut Transaction<'_, Postgres>,
    payment_table_id: Uuid,
    payment_order_id: &str,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE commerce_payment_data SET payment_order_id=$1 WHERE id=$2
        "#,
        payment_order_id,
        payment_table_id,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while updating payment_order_id  to database")
    })?;
    Ok(())
}

#[tracing::instrument(name = "update_order_and_payment_status", skip(transaction), fields())]
pub async fn update_payment_status(
    transaction: &mut Transaction<'_, Postgres>,
    transaction_id: Uuid,
    updated_by: &str,
    payment_status: PaymentStatus,
    payment_id: &str,
    payment_order_id: &str,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        WITH updated_order AS (
            UPDATE commerce_data
            SET updated_on = $1, updated_by = $2
            WHERE external_urn = $3
            RETURNING id
        )
        UPDATE commerce_payment_data
        SET payment_id = $4, payment_status = $5
        FROM updated_order
        WHERE commerce_payment_data.payment_order_id = $6
        AND commerce_payment_data.commerce_data_id = updated_order.id;
        "#,
        Utc::now(),
        updated_by,
        transaction_id,
        payment_id,
        payment_status as PaymentStatus,
        payment_order_id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute combined query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while updating order and payment status")
    })?;

    Ok(())
}

// #[tracing::instrument(name = "update paymentstatus", skip(transaction))]
// pub async fn update_payment_status(
//     transaction: &mut Transaction<'_, Postgres>,
//     payment_status: PaymentStatus,
//     payment_id: &str,
//     payment_order_id: &str,
//     transaction_id: Uuid,
// ) -> Result<(), anyhow::Error> {
//     // Execute the query within the transaction
//     let query = sqlx::query!(
//         r#"
//         UPDATE commerce_payment_data
//         SET payment_id = $1, payment_status = $2
//         FROM commerce_data
//         WHERE commerce_payment_data.payment_order_id = $3
//         AND commerce_payment_data.commerce_data_id = commerce_data.id
//         AND commerce_data.external_urn = $4
//         "#,
//         payment_id,
//         payment_status as PaymentStatus,
//         payment_order_id,
//         transaction_id,
//     );
//     transaction.execute(query).await.map_err(|e| {
//         tracing::error!("Failed to execute query in transaction: {:?}", e);
//         anyhow::Error::new(e).context(
//             "A database failure occurred while updating commerce payment data in transaction",
//         )
//     })?;

//     Ok(())
// }

pub fn get_payment_ws_params(order: &MinimalCommerceData) -> WebSocketParam {
    WebSocketParam {
        user_id: None,
        business_id: order.buyer_id,
        device_id: None,
    }
}
