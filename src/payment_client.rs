use bigdecimal::BigDecimal;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use uuid::Uuid;

use crate::schemas::{CurrencyType, GenericResponse};
#[derive(Debug)]
pub struct PaymentClient {
    http_client: Client,
    base_url: String,
    authorization_token: SecretString,
}

#[derive(Debug, Serialize)]
struct PaymentOrderCreateRequest {
    id: String,
    order_no: Uuid,
    amount: BigDecimal,
    currency_type: CurrencyType,
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

    fn _generate_order_create_request(
        _order_no: Uuid,
        _amount: BigDecimal,
        _currency_type: CurrencyType,
    ) -> PaymentOrderCreateRequest {
        PaymentOrderCreateRequest {
            id: todo!(),
            order_no: _order_no,
            amount: _amount,
            currency_type: _currency_type,
        }
    }

    async fn _create_order(
        &self,
        request_body: PaymentOrderCreateRequest,
    ) -> Result<(), anyhow::Error> {
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
        let response_body: GenericResponse<()> = response
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
}
