use crate::{
    db::AccountRepository,
    state::{AppState, UserbotHandle},
};
use anyhow::{Context, Result};
use rust_tdlib::{
    client::{tdlib_client::TdJson, Client, ConsoleAuthStateHandler, Worker},
    types::*,
};
use std::sync::Arc;
use tokio::sync::Mutex;

type TdClient = Client<TdJson>;
type TdWorker = Worker<ConsoleAuthStateHandler, TdJson>;

pub async fn spawn_userbot(state: AppState, account_id: i64) -> Result<()> {
    if state.is_userbot_running(account_id).await {
        tracing::warn!("Userbot {} is already running", account_id);
        return Ok(());
    }

    let account = AccountRepository::get_by_id(&state.db_pool, account_id)
        .await?
        .context("Account not found")?;

    tracing::info!("Starting userbot for account {}: {}", account_id, account.phone_number);

    let mut worker: TdWorker = Worker::builder().build()?;
    worker.start();

    let tdlib_params = TdlibParameters::builder()
        .api_id(state.config.telegram_api_id)
        .api_hash(state.config.telegram_api_hash.clone())
        .database_directory(format!("./data/tdlib/{}", account.phone_number))
        .use_message_database(true)
        .use_secret_chats(false)
        .system_language_code("en".to_string())
        .device_model("Desktop".to_string())
        .application_version("1.0.0".to_string())
        .build();

    let client: TdClient = Client::builder()
        .with_tdlib_parameters(tdlib_params)
        .build()?;

    let client = worker.bind_client(client).await?;
    let client = Arc::new(Mutex::new(client));

    let shutdown_tx = Arc::new(tokio::sync::Notify::new());

    let handle = UserbotHandle {
        client: client.clone(),
        account_id,
        phone_number: account.phone_number.clone(),
        shutdown_tx: shutdown_tx.clone(),
    };

    state.add_userbot(handle).await;

    let state_clone = state.clone();
    let account_clone = account.clone();
    tokio::spawn(async move {
        if let Err(e) = run_userbot_loop(state_clone, account_clone, client, shutdown_tx).await {
            tracing::error!("Userbot {} error: {}", account_id, e);
        }
    });

    tracing::info!("Userbot {} started successfully", account_id);
    Ok(())
}

async fn run_userbot_loop(
    _state: AppState,
    account: crate::db::models::Account,
    _client: Arc<Mutex<TdClient>>,
    shutdown: Arc<tokio::sync::Notify>,
) -> Result<()> {
    tracing::info!("Userbot {} event loop started", account.id);

    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                tracing::info!("Userbot {} received shutdown signal", account.id);
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Placeholder for update processing
            }
        }
    }

    tracing::info!("Userbot {} event loop stopped", account.id);
    Ok(())
}
