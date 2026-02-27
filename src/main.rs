use anyhow::Result;
use puppeteer::{bot, db::AccountRepository, userbot, AppState, Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,puppeteer=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Puppeteer...");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded. Owners: {:?}", config.owner_ids);

    // Initialize database
    let db_pool = puppeteer::db::init_db(&config.database_url).await?;
    tracing::info!("Database initialized");

    // Create application state
    let state = AppState::new(config, db_pool);

    // Load and spawn existing active accounts from database
    tracing::info!("Loading active accounts from database...");
    let active_accounts = AccountRepository::list_active(&state.db_pool).await?;
    
    for account in active_accounts {
        tracing::info!("Spawning userbot for account {} ({})", account.id, account.phone_number);
        if let Err(e) = userbot::spawn_userbot(state.clone(), account.id).await {
            tracing::error!("Failed to spawn userbot {}: {}", account.id, e);
        }
    }

    tracing::info!("Puppeteer is ready! Starting admin bot...");

    // Start admin bot (this will block until shutdown)
    bot::run_admin_bot(state.clone()).await?;

    // Graceful shutdown
    tracing::info!("Shutting down all userbots...");
    state.shutdown_all_userbots().await?;

    tracing::info!("Puppeteer stopped");
    Ok(())
}
