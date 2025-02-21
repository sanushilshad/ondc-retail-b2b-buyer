use std::collections::HashMap;

use crate::configuration::get_configuration;
use anyhow::anyhow;
use elasticsearch::auth::Credentials;
use elasticsearch::cert::CertificateValidation;
use elasticsearch::http::request::JsonBody;
use elasticsearch::http::transport::SingleNodeConnectionPool;
use elasticsearch::http::transport::TransportBuilder;
use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::indices::IndicesExistsParts;
use elasticsearch::BulkParts;
use elasticsearch::Elasticsearch;
use lazy_static::lazy_static;
// use once_cell::sync::Lazy;
use reqwest::Url;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum ElasticSearchIndex {
    ItemServicabilityHyperLocal,
    ItemServicabilityCountry,
    ItemServicabilityInterCity,
    ItemServicabilityGeoJson,
    NetworkParticipant,
}

impl ToString for ElasticSearchIndex {
    fn to_string(&self) -> String {
        match self {
            ElasticSearchIndex::ItemServicabilityHyperLocal => {
                "b2b_retail_item_servicability_hyper_local".to_string()
            }
            ElasticSearchIndex::ItemServicabilityCountry => {
                "b2b_retail_item_servicability_country".to_string()
            }
            ElasticSearchIndex::ItemServicabilityInterCity => {
                "b2b_retail_item_servicability_inter_city".to_string()
            }
            ElasticSearchIndex::ItemServicabilityGeoJson => {
                "b2b_retail_item_servicability_geo_json".to_string()
            }
            ElasticSearchIndex::NetworkParticipant => {
                "b2b_retail_seller_network_participant".to_string()
            }
        }
    }
}

lazy_static! {
    static ref INDICES: HashMap<ElasticSearchIndex, serde_json::Value> = {
        let mut map = HashMap::new();
        map.insert(
            ElasticSearchIndex::ItemServicabilityHyperLocal,
            json!({
              "mappings": {
                "dynamic": false,
                "properties": {
                  "created_on": {
                    "type": "date"
                  },
                  "domain_code": {
                     "type": "keyword"
                  },
                  "id": {
                     "type": "keyword"
                  },
                  "location": {
                     "type": "geo_point"
                  },
                  "location_cache_id": {
                     "type": "keyword"
                  },
                  "network_participant_cache_id": {
                     "type": "keyword"
                  },
                  "provider_cache_id": {
                     "type": "keyword"
                  },
                  "radius": {
                     "type": "float"
                  }
                }
              }
            }
            ),
        );
        map.insert(
            ElasticSearchIndex::ItemServicabilityCountry,
            json!({
              "mappings": {
                "dynamic": false,
                "properties": {
                  "country_code": {
                    "type": "keyword"
                  },
                  "created_on": {
                    "type": "date"
                  },
                  "domain_code": {
                    "type": "keyword"
                  },
                  "id": {
                    "type": "keyword"
                  },
                  "location_cache_id": {
                    "type": "keyword"
                  },
                  "network_participant_cache_id": {
                    "type": "keyword"
                  },
                  "provider_cache_id": {
                    "type": "keyword"
                  }
                }
              }
            }),
        );
        map.insert(
            ElasticSearchIndex::ItemServicabilityInterCity,
            json!({
              "mappings": {
                "dynamic": false,
                "properties": {
                  "created_on": {
                    "type": "date"
                  },
                  "domain_code": {
                    "type": "keyword"
                  },
                  "id": {
                    "type": "keyword"
                  },
                  "location_cache_id": {
                    "type": "keyword"
                  },
                  "network_participant_cache_id": {
                    "type": "keyword"
                  },
                  "pincode": {
                    "type": "keyword"
                  },
                  "provider_cache_id": {
                    "type": "keyword"
                  }
                }
              }
            }),
        );
        map.insert(
            ElasticSearchIndex::ItemServicabilityGeoJson,
            json!({
              "mappings": {
                "dynamic": false,
                "properties": {
                  "created_on": {
                    "type": "date"
                  },
                  "domain_code": {
                    "type": "keyword"
                  },
                  "id": {
                    "type": "keyword"
                  },
                  "location_cache_id": {
                    "type": "keyword"
                  },
                  "network_participant_cache_id": {
                    "type": "keyword"
                  },
                  "coordinates": {
                    "type": "geo_shape"
                  },
                  "provider_cache_id": {
                    "type": "keyword"
                  }
                }
              }
            }),
        );
        map.insert(
            ElasticSearchIndex::NetworkParticipant,
            json!({
              "mappings": {
                "dynamic": false,
                "properties": {
                  "id": { "type": "keyword" },
                  "subscriber_id": { "type": "keyword" },
                  "name": {
                    "type": "text",
                    "fields": {
                      "keyword": { "type": "keyword" }
                    }
                  },
                  "short_desc": {
                    "type": "text",
                    "fields": {
                      "keyword": { "type": "keyword" }
                    }
                  },
                  "long_desc": {
                    "type": "text",
                    "fields": {
                      "keyword": { "type": "keyword" }
                    }
                  },
                  "images": { "type": "text" },
                  "created_on": { "type": "date" }
                }
              }
            }),
        );
        map
    };
}

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

    async fn generate_indices(&self) -> Result<(), anyhow::Error> {
        for (index_name, mapping_json) in INDICES.iter() {
            let full_index_name = self.get_index(&index_name.to_string()); // Add environment prefix

            let exists_response = self
                .client
                .indices()
                .exists(IndicesExistsParts::Index(&[&full_index_name]))
                .send()
                .await?;

            if exists_response.status_code() != 200 {
                let create_response = self
                    .client
                    .indices()
                    .create(IndicesCreateParts::Index(&full_index_name))
                    .body(mapping_json)
                    .send()
                    .await?;

                if create_response.status_code() != 200 {
                    let body: Value = create_response.json().await?;
                    tracing::error!("Response body: {:?}", body);
                    return Err(anyhow!("Failed to create index"));
                } else {
                    tracing::info!("Index '{}' created with mapping", full_index_name);
                }
            } else {
                tracing::info!("Index '{}' already exists", full_index_name);
            }
        }
        Ok(())
    }
}

pub async fn generate_indices() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let es_client = configuration.elastic_search.client();
    let _ = es_client
        .generate_indices()
        .await
        .map_err(|e| tracing::info!("{}", e.to_string()));
}
