use persona_forge::config::Config;
use persona_forge::logging;
use persona_forge::state::AppState;
use persona_forge::bot::handlers::callbacks::handle_callback_query;
use persona_forge::webapp::start_webapp_server;
use sqlx::sqlite::SqlitePoolOptions;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    // Initialize our beautiful logging system
    logging::init();
    
    // Print startup banner
    logging::print_banner();

    // Load configuration
    let config = match Config::from_env() {
        Ok(cfg) => {
            logging::print_config(
                &cfg.bot_name,
                cfg.owner_id,
                &cfg.ollama_chat_model,
                cfg.vision_enabled,
                cfg.voice_enabled,
                cfg.web_search_enabled,
            );
            cfg
        }
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            return;
        }
    };

    // Connect to database
    let db_pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            logging::print_db_connected(&config.database_url);
            pool
        }
        Err(e) => {
            tracing::error!("Database connection failed: {}", e);
            return;
        }
    };

    // Run migrations
    if let Err(e) = sqlx::migrate!("./migrations").run(&db_pool).await {
        tracing::error!("Migrations failed: {}", e);
        return;
    }
    tracing::info!("Migrations applied successfully");

    // Sync env config to runtime_config
    let _ = persona_forge::db::set_config(&db_pool, "ollama_chat_model", &config.ollama_chat_model).await;
    let _ = persona_forge::db::set_config(&db_pool, "ollama_embedding_model", &config.ollama_embedding_model).await;
    let _ = persona_forge::db::set_config(&db_pool, "ollama_vision_model", &config.ollama_vision_model).await;
    let _ = persona_forge::db::set_config(&db_pool, "temperature", &config.temperature.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "max_tokens", &config.max_tokens.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "vision_enabled", &config.vision_enabled.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "voice_enabled", &config.voice_enabled.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "web_search_enabled", &config.web_search_enabled.to_string()).await;
    tracing::debug!("Runtime config synced from environment");

    let webapp_port = config.webapp_port;
    let bot = Bot::new(config.teloxide_token.clone());
    let app_state = AppState::new(config, db_pool);

    // Get bot info from Telegram API (with retry)
    for attempt in 1..=3 {
        match bot.get_me().await {
            Ok(me) => {
                let username = me.username.clone().unwrap_or_default();
                let first_name = me.first_name.clone();
                
                let bot_info = persona_forge::state::BotInfo {
                    id: me.id.0,
                    username: username.clone(),
                    first_name: first_name.clone(),
                };
                
                logging::print_bot_info(&first_name, &username);
                app_state.set_bot_info(bot_info).await;
                break;
            }
            Err(e) => {
                if attempt < 3 {
                    tracing::warn!("Failed to get bot info (attempt {}): {}, retrying...", attempt, e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                } else {
                    tracing::warn!("Failed to get bot info after 3 attempts: {}", e);
                }
            }
        }
    }

    // Start webapp server in background
    let webapp_state = app_state.clone();
    tokio::spawn(async move {
        start_webapp_server(webapp_state, webapp_port).await;
    });
    logging::print_webapp_started(webapp_port);

    // Print ready message
    logging::print_ready();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(persona_forge::bot::handlers::messages::handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback_query));

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build();

    // Run dispatcher
    dispatcher.dispatch().await;

    // Print shutdown stats
    logging::print_shutdown();
}
