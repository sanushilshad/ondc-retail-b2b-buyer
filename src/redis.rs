use redis::{aio::Connection as connection, RedisResult};
use secrecy::ExposeSecret;

use crate::configuration::RedisSettings;

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub async fn new(redis_obj: RedisSettings) -> Result<Self, redis::RedisError> {
        let redis_str = redis_obj.get_string().expose_secret().to_string();
        let client = redis::Client::open(redis_str)?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> RedisResult<connection> {
        self.client.get_async_connection().await
    }
}
