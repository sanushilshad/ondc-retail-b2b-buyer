use rust_test::commands::migration;
#[tokio::main]
pub async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        if args[1] == "migrate" {
            migration::run_migrations().await;
        }

        if args[1] == "sqlx_migrate" {
            migration::migrate_using_sqlx().await;
        }
    } else {
        println!("Invalid command. Use Enter a valid command");
    }
}
