#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        routes::order::{
            schemas::{OrderListFilter, OrderListRequest},
            utils::{
                delete_order, get_commerce_data, get_commerce_data_line, get_commerce_fulfillments,
                get_commerce_payments, get_order_list,
            },
        },
        tests::tests::get_test_pool,
    };

    #[tokio::test]
    async fn test_order_list_sql() {
        let pool = get_test_pool().await;
        // without date
        let order_obj = OrderListRequest {
            transaction_id: Some(vec![Uuid::new_v4()]),
            start_date: None,
            end_date: None,
            offset: 0,
            limit: 1,
        };
        let filter = OrderListFilter::new(order_obj, Some(Uuid::new_v4()), Uuid::new_v4());
        let order = get_order_list(&pool, filter).await;
        assert!(order.is_ok());
        // with date
        let order_obj = OrderListRequest {
            transaction_id: Some(vec![Uuid::new_v4()]),
            start_date: Some(Utc::now()),
            end_date: Some(Utc::now()),
            offset: 0,
            limit: 1,
        };
        let filter = OrderListFilter::new(order_obj, Some(Uuid::new_v4()), Uuid::new_v4());
        let order = get_order_list(&pool, filter).await;
        assert!(order.is_ok());
    }

    #[tokio::test]
    async fn test_order_fetch_sql() {
        let pool = get_test_pool().await;
        // let order_obj = fetch_order_by_id(&pool, Uuid::new_v4()).await;
        let transaction_id = Uuid::new_v4();
        let order_data = get_commerce_data(&pool, transaction_id).await;
        let line_data = get_commerce_data_line(&pool, transaction_id).await;

        let payment_data = get_commerce_payments(&pool, transaction_id).await;

        let fulfillment_data = get_commerce_fulfillments(&pool, transaction_id).await;
        assert!(order_data.is_ok());
        assert!(line_data.is_ok());
        assert!(payment_data.is_ok());
        assert!(fulfillment_data.is_ok());
    }

    #[tokio::test]
    async fn test_delete_order_sql() {
        let pool = get_test_pool().await;
        let mut transaction = pool.begin().await.unwrap();
        let _ = delete_order(&mut transaction, Uuid::new_v4()).await;
        transaction.commit().await.unwrap();
    }
}
