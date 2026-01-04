use crate::config::Config;
use crate::llm::client::LlmClient;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use teloxide::prelude::*;
use tokio::sync::Mutex;

pub type DialogueState = Arc<Mutex<HashMap<ChatId, Vec<Message>>>>;
pub type AdminCache = Arc<Mutex<HashMap<ChatId, Vec<UserId>>>>;
pub type RateLimiter = Arc<Mutex<HashMap<ChatId, Instant>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub llm_client: LlmClient,
    pub dialogues: DialogueState,
    pub db_pool: SqlitePool,
    pub admin_cache: AdminCache,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(config: Config, db_pool: SqlitePool) -> Self {
        let config_arc = Arc::new(config);
        Self {
            config: config_arc.clone(),
            llm_client: LlmClient::new(config_arc.ollama_url.clone()),
            dialogues: Arc::new(Mutex::new(HashMap::new())),
            db_pool,
            admin_cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
