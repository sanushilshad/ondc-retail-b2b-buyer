use crate::routes::ondc::utils::serialize_timestamp_without_nanos;
use crate::{errors::GenericError, schemas::GenericResponse};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ChatParticipant {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCreateRequest {
    pub buyer: ChatParticipant,
    pub seller: ChatParticipant,
    pub chat_id: Uuid,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChatData {
    pub seller_link: String,
    pub buyer_link: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageType {
    Text,
    Header,
    Divider,
}

#[derive(Debug, Serialize)]
pub struct SendMessageDataDescription {
    pub text: String,
    pub r#type: ChatMessageType,
}

#[derive(Debug, Serialize)]
pub struct SendMessageData {
    pub actions: Vec<String>,
    pub title: String,
    pub description: Vec<SendMessageDataDescription>,
}

#[derive(Debug, Serialize)]
pub struct SendChatRequest {
    pub sender: ChatParticipant,
    #[serde(serialize_with = "serialize_timestamp_without_nanos")]
    pub timestamp: DateTime<Utc>,
    pub chat_id: Uuid,
    pub data: SendMessageData,
}

#[derive(Debug)]
pub struct ChatClient {
    http_client: Client,
    base_url: String,
    authorization_token: SecretString,
}

impl ChatClient {
    #[tracing::instrument]
    pub fn new(
        base_url: String,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        tracing::info!("Establishing connection to the chat server.");
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

    pub fn get_chat_participant(&self, id: &str, name: &str) -> ChatParticipant {
        ChatParticipant {
            id: id.to_owned(),
            name: name.to_owned(),
        }
    }

    pub async fn get_chat_link(
        &self,
        chat_id: Uuid,
        seller: ChatParticipant,
        buyer: ChatParticipant,
    ) -> Result<ChatData, anyhow::Error> {
        let url = format!("{}/create", self.base_url);
        let request_body = ChatCreateRequest {
            buyer,
            seller,
            chat_id,
        };
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.get_auth_token())
            .header("x-request-id", "internal")
            .header("x-device-id", "internal")
            .json(&request_body)
            .send()
            .await?;
        // .map_err(|err| anyhow!("Request error: {}", err))?;

        let status = response.status();
        let response_body: GenericResponse<ChatData> = response
            .json()
            .await
            .map_err(|err| anyhow::anyhow!(format!("Failed to parse response: {}", err)))?;
        if status.is_success() {
            response_body
                .data
                .ok_or_else(|| anyhow::anyhow!("Chat not found".to_string()))
        } else {
            Err(anyhow::anyhow!(response_body.customer_message))
        }
    }

    pub fn get_send_message_data(
        &self,
        title: &str,
        descriptions: Vec<SendMessageDataDescription>,
    ) -> SendMessageData {
        SendMessageData {
            actions: vec![],
            title: title.to_owned(),
            description: descriptions,
        }
    }
    pub async fn send_chat_data(
        &self,
        chat_id: Uuid,
        sender: ChatParticipant,
        data: SendMessageData,
    ) -> Result<(), anyhow::Error> {
        let url = format!("{}/send/message", self.base_url);
        let request_body = SendChatRequest {
            sender,
            chat_id,
            timestamp: Utc::now(),
            data,
        };
        println!("{:?}", &request_body.data);
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.get_auth_token())
            .header("x-request-id", "internal")
            .header("x-device-id", "internal")
            .json(&request_body)
            .send()
            .await
            .map_err(|err| GenericError::UnexpectedError(anyhow!("Request error: {}", err)))?;

        let status = response.status();
        let response_body: GenericResponse<ChatData> = response
            .json()
            .await
            .map_err(|err| anyhow::anyhow!(format!("Failed to parse response: {}", err)))?;
        if status.is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(response_body.customer_message))
        }
    }
}
