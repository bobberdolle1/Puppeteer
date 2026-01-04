use persona_forge::config::Config;
use persona_forge::state::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting PersonaForge bot...");

    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("Failed to load configuration: {}", e);
            return;
        }
    };

    let db_pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            log::info!("Database connection pool created.");
            pool
        }
        Err(e) => {
            log::error!("Failed to create database connection pool: {}", e);
            return;
        }
    };

    if let Err(e) = sqlx::migrate!("./migrations").run(&db_pool).await {
        log::error!("Failed to run database migrations: {}", e);
        return;
    }
    log::info!("Database migrations ran successfully.");

    let bot = Bot::new(config.teloxide_token.clone());
    let app_state = AppState::new(config, db_pool);

    let handler = Update::filter_message().endpoint(persona_forge::bot::handlers::messages::handle_message);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("Bot has shut down.");
}