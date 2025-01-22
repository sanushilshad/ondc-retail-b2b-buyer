use elasticsearch::auth::Credentials;
use elasticsearch::cert::CertificateValidation;
use elasticsearch::http::transport::SingleNodeConnectionPool;
use elasticsearch::http::transport::TransportBuilder;
use elasticsearch::Elasticsearch;
use elasticsearch::IndexParts;
use reqwest::Url;
use serde_json::json;
#[derive(Debug)]
pub struct ElasticSearchClient {
    client: Elasticsearch,
}

impl ElasticSearchClient {
    #[tracing::instrument]
    pub fn new(base_url: String) -> Self {
        let url = Url::parse(&base_url).expect("Something went wrong while parsing url");
        tracing::info!("Establishing connection to the ElasticSearch server.");
        let conn_pool = SingleNodeConnectionPool::new(url);
        // let credentials = Credentials::new(
        //     "key".to_owned(),
        //     "ckZZWlJwUUJlaVlfNkdGbkRtSXE6RzFXYnVqRzdSWU81UGpzbTZQbUhlQQ==".to_owned(),
        // );
        let credentials =
            Credentials::Basic("elastic".to_owned(), "ltWBNC0ZZ=wsBg2fQsOm".to_owned());
        let transport = TransportBuilder::new(conn_pool)
            .auth(credentials)
            .cert_validation(CertificateValidation::None)
            .build()
            .expect("Something went wrong while setting ElasticSearch");

        let client = Elasticsearch::new(transport);
        Self { client }
    }

    pub async fn send(&self) {
        let response = self
            .client
            .index(IndexParts::IndexId("tweets", "2"))
            .body(json!({
                "id": 1,
                "user": "sanu",
                "post_date": "2009-11-15T00:00:00Z",
                "message": "Trying out Elasticsearch, so far so good?"
            }))
            .send()
            .await
            // .map_err(|e| eprint!("apple{}", e.to_string()))
            .expect("Something went wrong while setting ElasticSearch");

        // let successful = response.status_code().is_success();
        eprint!("{:?}", response);
    }
}
