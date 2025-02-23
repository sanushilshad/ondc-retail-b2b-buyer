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
    ProviderServicabilityHyperLocal,
    ProviderServicabilityCountry,
    ProviderServicabilityInterCity,
    ProviderServicabilityGeoJson,
    NetworkParticipant,
    ProviderLocation,
    Provider,
    ProviderItemVariant,
    ProviderItem,
}

impl ToString for ElasticSearchIndex {
    fn to_string(&self) -> String {
        match self {
            ElasticSearchIndex::ProviderServicabilityHyperLocal => {
                "b2b_retail_provider_servicability_hyper_local".to_string()
            }
            ElasticSearchIndex::ProviderServicabilityCountry => {
                "b2b_retail_provider_servicability_country".to_string()
            }
            ElasticSearchIndex::ProviderServicabilityInterCity => {
                "b2b_retail_provider_servicability_inter_city".to_string()
            }
            ElasticSearchIndex::ProviderServicabilityGeoJson => {
                "b2b_retail_provider_servicability_geo_json".to_string()
            }
            ElasticSearchIndex::NetworkParticipant => {
                "b2b_retail_seller_network_participant".to_string()
            }

            ElasticSearchIndex::ProviderLocation => {
                "b2b_retail_seller_provider_location".to_string()
            }

            ElasticSearchIndex::Provider => "b2b_retail_seller_provider".to_string(),

            ElasticSearchIndex::ProviderItemVariant => "b2b_retail_seller_item_variant".to_string(),
            ElasticSearchIndex::ProviderItem => "b2b_retail_seller_item".to_string(),
        }
    }
}

lazy_static! {
    static ref INDICES: HashMap<ElasticSearchIndex, serde_json::Value> = {
        let mut map = HashMap::new();
        map.insert(
            ElasticSearchIndex::ProviderServicabilityHyperLocal,
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
            ElasticSearchIndex::ProviderServicabilityCountry,
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
            ElasticSearchIndex::ProviderServicabilityInterCity,
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
            ElasticSearchIndex::ProviderServicabilityGeoJson,
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
        map.insert(
            ElasticSearchIndex::ProviderLocation,
            json!({
              "mappings": {
                "dynamic": false,
                "properties": {
                  "id": { "type": "keyword" },
                  "provider_cache_id": { "type": "keyword" },
                  "location_id": { "type": "keyword" },
                  "latitude": { "type": "float" },
                  "longitude": { "type": "float" },
                  "address": { "type": "keyword" },
                  "city_code": { "type": "keyword" },
                  "city_name": { "type": "keyword" },
                  "state_code": { "type": "keyword" },
                  "state_name": { "type": "keyword" },
                  "country_code": { "type": "keyword" },
                  "country_name": { "type": "keyword" },
                  "area_code": { "type": "keyword" },
                  "created_on": { "type": "date" },
                  "updated_on": { "type": "date" }
                }
              }
            }),
        );

        map.insert(
            ElasticSearchIndex::Provider,
            json!({
                "mappings": {
                    "dynamic": false,
                    "properties": {
                        "id": { "type": "keyword" },
                        "provider_id": { "type": "keyword" },
                        "network_participant_cache_id": { "type": "keyword" },
                        "name": { "type": "text" },
                        "code": { "type": "text" },
                        "short_desc": { "type": "text" },
                        "long_desc": { "type": "text" },
                        "images": { "type": "text" },
                        "rating": { "type": "float" },
                        "ttl": { "type": "text" },
                        "credentials": { "type": "object" },
                        "contact": { "type": "object" },
                        "terms": { "type": "object" },
                        "identifications": { "type": "object" },
                        "created_on": { "type": "date"},
                        "updated_on": { "type": "date"}
                    }
                }
            }),
        );
        map.insert(
            ElasticSearchIndex::ProviderItemVariant,
            json!(
              {
                "mappings": {
                  "dynamic": false,
                  "properties": {
                    "id": { "type": "keyword" },
                    "provider_cache_id": { "type": "keyword" },
                    "variant_id": { "type": "keyword" },
                    "variant_name": { "type": "keyword" },
                    "attributes": { "type": "object" },
                    "created_on": { "type": "date" },
                    "updated_on": { "type": "date" }
                  }
                }
              }
            ),
        );

        map.insert(
            ElasticSearchIndex::ProviderItem,
            json!(
              {
              "mappings": {
                "dynamic": false,
                "properties": {
                  "cancellation_terms": {
                    "type": "object"
                  },
                  "categories": {
                    "type": "nested",
                    "properties": {
                      "code": {
                        "type": "text",
                        "fields": {
                          "keyword": {
                            "type": "keyword"
                          }
                        }
                      },
                      "name": {
                        "type": "text",
                        "fields": {
                          "keyword": {
                            "type": "keyword"
                          }
                        }
                      }
                    }
                  },
                  "country_code": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "country_of_origin": {
                    "type": "text"
                  },
                  "created_on": {
                    "type": "date"
                  },
                  "creator": {
                    "type": "object"
                  },
                  "currency": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "domain_code": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "fulfillment_options": {
                    "type": "keyword"
                  },
                  "id": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "images": {
                    "type": "keyword"
                  },
                  "item_code": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "item_id": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "item_name": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "location_ids": {
                    "type": "keyword"
                  },
                  "long_desc": {
                    "type": "text"
                  },
                  "matched": {
                    "type": "boolean"
                  },
                  "maximum_price": {
                    "type": "float"
                  },
                  "payment_options": {
                    "type": "keyword"
                  },
                  "price_slabs": {
                    "type": "nested",
                    "properties": {
                      "max": {
                        "type": "float"
                      },
                      "min": {
                        "type": "float"
                      },
                      "price_with_tax": {
                        "type": "float"
                      },
                      "price_without_tax": {
                        "type": "float"
                      }
                    }
                  },
                  "price_with_tax": {
                    "type": "float"
                  },
                  "price_without_tax": {
                    "type": "float"
                  },
                  "provider_cache_id": {
                    "type": "text",
                    "fields": {
                      "keyword": {
                        "type": "keyword"
                      }
                    }
                  },
                  "qty": {
                    "type": "object"
                  },
                  "recommended": {
                    "type": "boolean"
                  },
                  "replacement_terms": {
                    "type": "object"
                  },
                  "return_terms": {
                    "type": "object"
                  },
                  "short_desc": {
                    "type": "text"
                  },
                  "tax_rate": {
                    "type": "text"
                  },
                  "time_to_ship": {
                    "type": "text"
                  }
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
