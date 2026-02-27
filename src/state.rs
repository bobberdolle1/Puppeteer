use crate::config::Config;
use anyhow::Result;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handle for a running userbot worker
#[derive(Clone)]
pub struct UserbotHandle {
    /// The rust-tdlib client
    pub client: Arc<tokio::sync::Mutex<rust_tdlib::client::Client<rust_tdlib::client::tdlib_client::TdJson>>>,
    /// Account ID in the database
    pub account_id: i64,
    /// Phone number
    pub phone_number: String,
    /// Cancellation token for graceful shutdown
    pub shutdown_tx: Arc<tokio::sync::Notify>,
}

/// Global application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Arc<Config>,
    
    /// Database connection pool
    pub db_pool: SqlitePool,
    
    /// Registry of active MTProto clients (userbots)
    /// Key: account_id, Value: UserbotHandle
    pub userbots: Arc<RwLock<HashMap<i64, UserbotHandle>>>,
}

impl AppState {
    /// Create a new application state
    pub fn new(config: Config, db_pool: SqlitePool) -> Self {
        Self {
            config: Arc::new(config),
            db_pool,
            userbots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a userbot to the active pool
    pub async fn add_userbot(&self, handle: UserbotHandle) {
        let account_id = handle.account_id;
        let mut userbots = self.userbots.write().await;
        userbots.insert(account_id, handle);
        tracing::info!("Added userbot {} to active pool", account_id);
    }

    /// Remove a userbot from the active pool
    pub async fn remove_userbot(&self, account_id: i64) -> Option<UserbotHandle> {
        let mut userbots = self.userbots.write().await;
        let handle = userbots.remove(&account_id);
        if handle.is_some() {
            tracing::info!("Removed userbot {} from active pool", account_id);
        }
        handle
    }

    /// Get a userbot handle by account ID
    pub async fn get_userbot(&self, account_id: i64) -> Option<UserbotHandle> {
        let userbots = self.userbots.read().await;
        userbots.get(&account_id).cloned()
    }

    /// Check if a userbot is running
    pub async fn is_userbot_running(&self, account_id: i64) -> bool {
        let userbots = self.userbots.read().await;
        userbots.contains_key(&account_id)
    }

    /// Get count of active userbots
    pub async fn active_userbot_count(&self) -> usize {
        let userbots = self.userbots.read().await;
        userbots.len()
    }

    /// List all active userbot account IDs
    pub async fn list_active_userbot_ids(&self) -> Vec<i64> {
        let userbots = self.userbots.read().await;
        userbots.keys().copied().collect()
    }

    /// Shutdown a specific userbot gracefully
    pub async fn shutdown_userbot(&self, account_id: i64) -> Result<()> {
        if let Some(handle) = self.remove_userbot(account_id).await {
            // Signal shutdown
            handle.shutdown_tx.notify_one();
            
            // TDLib client cleanup is automatic
            tracing::info!("Shutdown userbot {} successfully", account_id);
        }
        Ok(())
    }

    /// Shutdown all userbots gracefully
    pub async fn shutdown_all_userbots(&self) -> Result<()> {
        let account_ids = self.list_active_userbot_ids().await;
        
        for account_id in account_ids {
            if let Err(e) = self.shutdown_userbot(account_id).await {
                tracing::error!("Failed to shutdown userbot {}: {}", account_id, e);
            }
        }
        
        tracing::info!("All userbots shutdown");
        Ok(())
    }
}
