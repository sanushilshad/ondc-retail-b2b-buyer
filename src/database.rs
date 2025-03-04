use crate::configuration::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, Connection, Executor, PgConnection, PgPool};
use std::{fs, io};
pub fn get_connection_pool(configuration: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(
            configuration.acquire_timeout,
        ))
        .max_connections(configuration.max_connections)
        .min_connections(configuration.min_connections)
        .connect_lazy_with(configuration.with_db())
}

pub async fn configure_database_using_sqlx(config: &DatabaseConfig) -> PgPool {
    // Create database
    create_database(config).await;
    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    let test_connection_pool = PgPool::connect_with(config.test_with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&test_connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
#[tracing::instrument(name = "Confiure Database")]
pub async fn configure_database(config: &DatabaseConfig) -> PgPool {
    create_database(config).await;
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    let test_connection_pool = PgPool::connect_with(config.test_with_db())
        .await
        .expect("Failed to connect to Postgres.");

    let _ = execute_query("./migrations", &connection_pool).await;
    let _ = execute_query("./migrations", &test_connection_pool).await;
    connection_pool
}
#[tracing::instrument(name = "Create Database")]
pub async fn create_database(config: &DatabaseConfig) {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    let db_count: Result<Option<i64>, sqlx::Error> =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM pg_database WHERE datname = $1")
            .bind(&config.name)
            .fetch_optional(&mut connection)
            .await;

    match db_count {
        Ok(Some(count)) => {
            if count > 0 {
                tracing::info!("Database {} already exists.", &config.name);
            } else {
                connection
                    .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
                    .await
                    .expect("Failed to create database.");
                eprintln!("Database created.");
            }
        }
        Ok(_) => eprintln!("No rows found."),
        Err(err) => eprintln!("Error: {}", err),
    }

    let test_db_count: Result<Option<i64>, sqlx::Error> =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM pg_database WHERE datname = $1")
            .bind(&config.test_name)
            .fetch_optional(&mut connection)
            .await;

    match test_db_count {
        Ok(Some(count)) => {
            if count > 0 {
                eprintln!("Test database {} already exists.", &config.test_name);
            } else {
                connection
                    .execute(format!(r#"CREATE DATABASE "{}";"#, config.test_name).as_str())
                    .await
                    .expect("Failed to create test database.");
                eprintln!("Test database {} created.", &config.test_name);
            }
        }
        Ok(_) => eprintln!("No rows found for the test database check."),
        Err(err) => eprintln!("Error checking test database existence: {}", err),
    }
}

#[tracing::instrument(name = "Execute Queries")]
async fn execute_query(path: &str, pool: &PgPool) -> io::Result<()> {
    let migration_files = fs::read_dir(path)?;
    for migration_file in migration_files {
        let migration_file = migration_file?;
        let migration_path = migration_file.path();
        let migration_sql = fs::read_to_string(&migration_path)?;
        let statements: String = migration_sql.replace('\n', "");
        let new_statement: Vec<&str> = statements
            .split(';')
            .filter(|s| !s.trim().is_empty() & !s.starts_with("--"))
            .collect();
        for statement in new_statement {
            if let Err(err) = sqlx::query(statement).execute(pool).await {
                eprintln!("Error executing statement {:?}: {} ", statement, err);
            } else {
                eprintln!("Migration applied: {:?}", statement);
            }
        }

        eprintln!("Migration applied: {:?}", migration_path);
    }

    Ok(())
}
