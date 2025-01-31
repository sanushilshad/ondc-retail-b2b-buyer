use std::time::Duration;

use actix_web::web::Data;
use anyhow::anyhow;
use futures::StreamExt;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    consumer::{CommitMode, Consumer, StreamConsumer},
    error::KafkaError,
    producer::FutureProducer,
    ClientConfig, Message,
};
use sqlx::PgPool;

use crate::{
    configuration::get_configuration,
    routes::ondc::{utils::process_on_search, KafkaSearchData},
    utils::pascal_to_snake_case,
    websocket_client::WebSocketClient,
};

// #[derive(Debug)]
// pub enum TopicType {
//     Search,
// }

// impl std::fmt::Display for TopicType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
//     }
// }

#[derive(Debug)]
pub enum KafkaGroupName {
    Search,
}

impl std::fmt::Display for KafkaGroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

pub struct KafkaClient {
    servers: String,
    pub search_topic_name: String,
    pub producer: FutureProducer,
}

impl KafkaClient {
    pub fn create_producer(servers: &str) -> FutureProducer {
        ClientConfig::new()
            .set("bootstrap.servers", servers)
            .set("message.timeout.ms", "5000")
            .create::<FutureProducer>()
            .expect("Kafka Producer creation error")
    }
    pub fn new(servers: String, search_topic_name: String) -> Self {
        let producer = Self::create_producer(&servers);
        Self {
            servers,
            search_topic_name,
            producer,
        }
    }
    pub async fn create_topic(&self, topic_name: &str) -> Result<(), anyhow::Error> {
        let admin_client: AdminClient<_> = ClientConfig::new()
            .set("bootstrap.servers", &self.servers)
            .create()
            .expect("Failed to create Kafka AdminClient");

        let new_topic = NewTopic::new(topic_name, 9, TopicReplication::Fixed(3));

        let options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(5)));
        match admin_client.create_topics(&[new_topic], &options).await {
            Ok(results) => {
                for result in results {
                    match result {
                        Ok(_) => println!("Topic '{}' created successfully", &topic_name),
                        Err((topic_name, err)) => {
                            return Err(anyhow!(
                                "Failed to create topic '{}': {:?}",
                                topic_name,
                                err
                            ));
                        }
                    };
                }
                Ok(())
            }
            Err(err) => Err(anyhow!("Error during topic creation: {:?}", err)),
        }
    }

    pub async fn kafka_client_search_consumer(
        &self,
        websocket_client: Data<WebSocketClient>,
        pool: Data<PgPool>,
    ) -> Result<(), KafkaError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.servers)
            .set("group.id", KafkaGroupName::Search.to_string()) // Use WebSocket key as group.id
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .create()?;

        consumer.subscribe(&[&self.search_topic_name])?;

        tokio::spawn(async move {
            let mut message_stream = consumer.stream();

            while let Some(result) = message_stream.next().await {
                match result {
                    Ok(msg) => {
                        if let Some(payload) = msg.payload() {
                            if let Ok(message_data) =
                                serde_json::from_slice::<KafkaSearchData>(payload)
                            {
                                match process_on_search(
                                    &pool,
                                    message_data.ondc_on_search,
                                    message_data.search_obj,
                                    &websocket_client,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        if let Err(e) =
                                            consumer.commit_message(&msg, CommitMode::Async)
                                        {
                                            eprintln!("Failed to commit message: {:?}", e);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error in process_on_search: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

pub async fn create_kafka_topic_command() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let kafka_client = configuration.kafka.client();
    let _ = kafka_client
        .create_topic(&kafka_client.search_topic_name)
        .await;
}
