use persona_forge::config::Config;
use persona_forge::state::AppState;
use persona_forge::bot::handlers::callbacks::handle_callback_query;
use persona_forge::webapp::start_webapp_server;
use sqlx::sqlite::SqlitePoolOptions;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    
    log::info!("╔════════════════════════════════════════╗");
    log::info!("║       🤖 PersonaForge Starting...      ║");
    log::info!("╚════════════════════════════════════════╝");

    let config = match Config::from_env() {
        Ok(cfg) => {
            log::info!("✅ Config loaded");
            log::info!("   ├─ Bot: {}", cfg.bot_name);
            log::info!("   ├─ Owner: {}", cfg.owner_id);
            log::info!("   ├─ LLM: {}", cfg.ollama_chat_model);
            log::info!("   ├─ Vision: {}", if cfg.vision_enabled { "✓" } else { "✗" });
            log::info!("   ├─ Voice: {}", if cfg.voice_enabled { "✓" } else { "✗" });
            log::info!("   └─ Web Search: {}", if cfg.web_search_enabled { "✓" } else { "✗" });
            cfg
        }
        Err(e) => {
            log::error!("❌ Failed to load config: {}", e);
            return;
        }
    };

    let db_pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            log::info!("✅ Database connected: {}", config.database_url);
            pool
        }
        Err(e) => {
            log::error!("❌ Database connection failed: {}", e);
            return;
        }
    };

    if let Err(e) = sqlx::migrate!("./migrations").run(&db_pool).await {
        log::error!("❌ Migrations failed: {}", e);
        return;
    }
    log::info!("✅ Migrations applied");

    // Sync env config to runtime_config (env takes precedence)
    let _ = persona_forge::db::set_config(&db_pool, "ollama_chat_model", &config.ollama_chat_model).await;
    let _ = persona_forge::db::set_config(&db_pool, "ollama_embedding_model", &config.ollama_embedding_model).await;
    let _ = persona_forge::db::set_config(&db_pool, "ollama_vision_model", &config.ollama_vision_model).await;
    let _ = persona_forge::db::set_config(&db_pool, "temperature", &config.temperature.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "max_tokens", &config.max_tokens.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "vision_enabled", &config.vision_enabled.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "voice_enabled", &config.voice_enabled.to_string()).await;
    let _ = persona_forge::db::set_config(&db_pool, "web_search_enabled", &config.web_search_enabled.to_string()).await;
    log::info!("✅ Runtime config synced from env");

    let webapp_port = config.webapp_port;
    let bot = Bot::new(config.teloxide_token.clone());
    let app_state = AppState::new(config, db_pool);

    // Get bot info from Telegram API (with retry)
    for attempt in 1..=3 {
        match bot.get_me().await {
            Ok(me) => {
                let bot_info = persona_forge::state::BotInfo {
                    id: me.id.0,
                    username: me.username.clone().unwrap_or_default(),
                    first_name: me.first_name.clone(),
                };
                log::info!("✅ Bot info: {} (@{})", bot_info.first_name, bot_info.username);
                app_state.set_bot_info(bot_info).await;
                break;
            }
            Err(e) => {
                if attempt < 3 {
                    log::warn!("⚠️ Failed to get bot info (attempt {}): {}, retrying...", attempt, e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                } else {
                    log::warn!("⚠️ Failed to get bot info after 3 attempts: {}", e);
                }
            }
        }
    }

    // Start webapp server in background
    let webapp_state = app_state.clone();
    tokio::spawn(async move {
        start_webapp_server(webapp_state, webapp_port).await;
    });
    log::info!("✅ WebApp started on port {}", webapp_port);

    log::info!("╔════════════════════════════════════════╗");
    log::info!("║         🚀 Bot is now running!         ║");
    log::info!("╚════════════════════════════════════════╝");

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(persona_forge::bot::handlers::messages::handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback_query));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("👋 Bot has shut down.");
}