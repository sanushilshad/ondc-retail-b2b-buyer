use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::schemas::WebSocketParam;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketActionType {
    Search,
    Select,
    Init,
    Confirm,
    Status,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WSRequest {
    #[schema(value_type = String)]
    pub user_id: Uuid,
    #[schema(value_type = String)]
    pub business_id: Option<Uuid>,
    pub device_id: Option<String>,
    pub action_type: WebSocketActionType,
    pub data: Value,
}

#[derive(Debug)]
pub struct WebSocketClient {
    http_client: Client,
    base_url: String,
    authorization_token: SecretString,
}

impl WebSocketClient {
    pub fn new(
        base_url: String,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
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

    pub async fn send_msg(
        &self,
        params: WebSocketParam,
        action_type: WebSocketActionType,
        data: Value,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/send", self.base_url);
        let request_body = WSRequest {
            user_id: params.user_id,
            business_id: Some(params.business_id),
            device_id: params.device_id,
            action_type,
            data,
        };
        self.http_client
            .post(&url)
            .header("Authorization", self.get_auth_token())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
