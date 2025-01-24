use bigdecimal::BigDecimal;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::{
    routes::order::schemas::PaymentStatus,
    schemas::{CurrencyType, GenericResponse},
};
#[derive(Debug)]
pub struct PaymentClient {
    http_client: Client,
    base_url: String,
    authorization_token: SecretString,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOrderCreateRequest<'a> {
    pub id: &'a str,
    pub order_no: Uuid,
    pub amount: &'a BigDecimal,
    pub currency_type: &'a CurrencyType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentServiceOrderStatus {
    Created,
    Attempted,
    Paid,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentServiceStatusType {
    Created,
    Authorized,
    Captured,
    Refunded,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentServicePaymentData {
    pub id: String,
    pub entity: String,
    pub amount: BigDecimal,
    pub currency: CurrencyType,
    pub status: PaymentServiceStatusType,
    pub order_id: String,
    pub method: String,
    pub captured: bool,
    pub created_at: i64,
}

impl PaymentServicePaymentData {
    pub fn payment_status(&self) -> PaymentStatus {
        match self.status {
            PaymentServiceStatusType::Created => PaymentStatus::NotPaid,
            PaymentServiceStatusType::Authorized => PaymentStatus::Paid,
            PaymentServiceStatusType::Captured => PaymentStatus::Paid,
            PaymentServiceStatusType::Refunded => PaymentStatus::Refunded,
            PaymentServiceStatusType::Failed => PaymentStatus::NotPaid,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentServiceOrderData {
    pub id: String,
    pub entity: String,
    pub amount: BigDecimal,
    pub currency: CurrencyType,
    pub amount_paid: BigDecimal,
    pub amount_due: BigDecimal,
    pub created_at: u64,
    pub status: PaymentServiceOrderStatus,
}

impl PaymentClient {
    #[tracing::instrument]
    pub fn new(
        base_url: String,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        tracing::info!("Establishing connection to the payment server.");
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            authorization_token,
        }
    }

    fn get_auth_token(&self) -> String {
        format!("Bearer {}", self.authorization_token.expose_secret())
    }

    pub fn generate_order_create_request<'a>(
        &self,
        order_no: Uuid,
        amount: &'a BigDecimal,
        id: &'a str,
        currency_type: &'a CurrencyType,
    ) -> PaymentOrderCreateRequest<'a> {
        PaymentOrderCreateRequest {
            id,
            order_no,
            amount,
            currency_type,
        }
    }

    pub async fn create_order<'a>(
        &self,
        request_body: PaymentOrderCreateRequest<'a>,
    ) -> Result<PaymentServiceOrderData, anyhow::Error> {
        let url = format!("{}/order/create", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.get_auth_token())
            .header("x-request-id", "internal")
            .header("x-device-id", "internal")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let response_body: GenericResponse<PaymentServiceOrderData> = response
            .json()
            .await
            .map_err(|err| anyhow::anyhow!(format!("Failed to parse response: {}", err)))?;
        if status.is_success() {
            response_body
                .data
                .ok_or_else(|| anyhow::anyhow!("Payment Order not found".to_string()))
        } else {
            Err(anyhow::anyhow!(response_body.customer_message))
        }
    }
    pub async fn fetch_payments_by_order_id(
        &self,
        payment_order_id: &str,
        payment_service_id: &str,
    ) -> Result<Option<Vec<PaymentServicePaymentData>>, anyhow::Error> {
        let url = format!(
            "{}/order/payment/fetch/{}/{}",
            self.base_url, payment_service_id, payment_order_id
        );
        println!("{}", url);
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.get_auth_token())
            .header("x-request-id", "internal")
            .header("x-device-id", "internal")
            .send()
            .await?;

        let status = response.status();
        let response_body: GenericResponse<Vec<PaymentServicePaymentData>> = response
            .json()
            .await
            .map_err(|err| anyhow::anyhow!(format!("Failed to parse response: {}", err)))?;
        if status.is_success() {
            Ok(response_body.data)
        } else {
            Err(anyhow::anyhow!(response_body.customer_message))
        }
    }

    pub fn determine_final_payment_status(
        &self,
        payment_order: Option<&Vec<PaymentServicePaymentData>>,
    ) -> PaymentStatus {
        payment_order.map_or(PaymentStatus::NotPaid, |payment_objs| {
            if payment_objs
                .iter()
                .any(|e| e.payment_status() == PaymentStatus::Paid)
            {
                PaymentStatus::Paid
            } else if payment_objs
                .iter()
                .any(|e| e.payment_status() == PaymentStatus::Refunded)
            {
                PaymentStatus::Refunded
            } else {
                PaymentStatus::NotPaid
            }
        })
    }
}
