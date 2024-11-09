use redis::{aio::MultiplexedConnection, RedisResult};
use secrecy::ExposeSecret;

use crate::configuration::RedisSetting;

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    #[tracing::instrument(name = "Initialize Redis")]
    pub async fn new(redis_obj: &RedisSetting) -> Result<Self, redis::RedisError> {
        let redis_str = redis_obj.get_string().expose_secret().to_string();
        let client = redis::Client::open(redis_str)?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> RedisResult<MultiplexedConnection> {
        self.client.get_multiplexed_tokio_connection().await
    }
}