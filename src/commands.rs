use crate::routes::product;
use crate::utils::generate_user_token;
use crate::{elastic_search_client, kafka_client, migration};
#[tracing::instrument(name = "Run custom command")]
pub async fn run_custom_commands(args: Vec<String>) -> Result<(), anyhow::Error> {
    if args.len() < 2 {
        eprintln!("Invalid command. Please provide a valid command.");
        return Ok(());
    }
    let command = args[1].as_str();

    match command {
        "migrate" => {
            migration::run_migrations().await;
        }
        "sqlx_migrate" => {
            migration::migrate_using_sqlx().await;
        }
        "generate_service_token" => {
            // let arg = args.get(2).unwrap_or(&TopicType::Search.to_string());
            generate_user_token().await;
        }
        "generate_kafka_topic" => {
            // let arg = args.get(2).unwrap_or(&TopicType::Search.to_string());
            kafka_client::create_kafka_topic_command().await;
        }
        "generate_elastic_search_indices" => {
            elastic_search_client::generate_indices().await;
        }
        "generate_item_cache" => {
            product::utils::generate_cache().await?;
        }
        "regenerate_item_cache" => {
            product::utils::regenerate_cache_to_es().await?;
        }
        _ => {
            eprintln!("Unknown command: {}. Please use a valid command.", command);
        }
    }

    Ok(())
}
