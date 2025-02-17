use anyhow::anyhow;
use elasticsearch::auth::Credentials;
use elasticsearch::cert::CertificateValidation;
use elasticsearch::http::request::JsonBody;
use elasticsearch::http::transport::SingleNodeConnectionPool;
use elasticsearch::http::transport::TransportBuilder;
use elasticsearch::BulkParts;
use elasticsearch::Elasticsearch;
use elasticsearch::IndexParts;
use reqwest::Url;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde_json::Value;
#[derive(Debug)]
pub struct ElasticSearchClient {
    client: Elasticsearch,
    env: String,
}

impl ElasticSearchClient {
    #[tracing::instrument]
    pub fn new(base_url: String, username: String, password: SecretString, env: String) -> Self {
        let url = Url::parse(&base_url).expect("Something went wrong while parsing url");
        tracing::info!("Establishing connection to the ElasticSearch server.");
        let conn_pool = SingleNodeConnectionPool::new(url);
        // let credentials = Credentials::new(
        //     "key".to_owned(),
        //     "ckZZWlJwUUJlaVlfNkdGbkRtSXE6RzFXYnVqRzdSWU81UGpzbTZQbUhlQQ==".to_owned(),
        // );
        let credentials = Credentials::Basic(username, password.expose_secret().to_owned());
        let transport = TransportBuilder::new(conn_pool)
            .auth(credentials)
            .cert_validation(CertificateValidation::None)
            .build()
            .expect("Something went wrong while setting ElasticSearch");

        let client = Elasticsearch::new(transport);
        Self { client, env }
    }
    pub fn get_index(&self, index: &str) -> String {
        format!("{}_{}", self.env, index)
    }
    pub async fn send(&self, index: &str, data: Vec<JsonBody<Value>>) -> Result<(), anyhow::Error> {
        let response = self
            .client
            .bulk(BulkParts::Index(&index))
            .body(data)
            .send()
            .await?;
        if response.status_code() != 200 {
            let response_body = response.json::<serde_json::Value>().await?;
            tracing::info!("{:?}", response_body);
            return Err(anyhow!(response_body));
        } else {
            tracing::info!("{:?}", response);
        }

        Ok(())
    }
}
