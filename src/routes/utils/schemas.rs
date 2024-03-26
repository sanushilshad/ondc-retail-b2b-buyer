use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RedisAction {
    Get,
    Set,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RedisBasicRequest {
    pub action: RedisAction,
    pub key: String,
    pub value: Option<String>,
}
