use crate::config::Config;
use crate::llm::client::LlmClient;
use crate::security::{SecurityConfig, SecurityTracker};
use crate::voice::VoiceClient;
use crate::web::search::WebSearchClient;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use teloxide::prelude::*;
use tokio::sync::{Mutex, Semaphore};

pub type DialogueState = Arc<Mutex<HashMap<ChatId, Vec<Message>>>>;
pub type AdminCache = Arc<Mutex<HashMap<ChatId, Vec<UserId>>>>;
pub type RateLimiter = Arc<Mutex<HashMap<ChatId, Instant>>>;
pub type WizardStates = Arc<Mutex<HashMap<ChatId, WizardState>>>;
pub type PendingMessages = Arc<Mutex<HashMap<(ChatId, Option<teloxide::types::ThreadId>), PendingBatch>>>;
pub type UserRateLimit = Arc<Mutex<HashMap<u64, Vec<Instant>>>>;

/// Pending message batch for debounce
#[derive(Clone, Debug)]
pub struct PendingBatch {
    pub messages: Vec<String>,
    pub last_message_time: Instant,
    pub user_id: Option<u64>,
    pub user_name: String,
}

/// Wizard state for multi-step interactions
#[derive(Clone, Debug)]
pub enum WizardState {
    /// Creating persona - waiting for name
    CreatingPersonaName,
    /// Creating persona - waiting for display name (name stored)
    CreatingPersonaDisplayName { name: String },
    /// Creating persona - waiting for triggers (name, display_name stored)
    CreatingPersonaTriggers { name: String, display_name: Option<String> },
    /// Creating persona - waiting for prompt (name, display_name, triggers stored)
    CreatingPersonaPrompt { name: String, display_name: Option<String>, triggers: Option<String> },
    /// Updating persona - waiting for ID
    UpdatingPersonaId,
    /// Updating persona - waiting for new name
    UpdatingPersonaName { id: i64 },
    /// Updating persona - waiting for new display name
    UpdatingPersonaDisplayName { id: i64, name: String },
    /// Updating persona - waiting for new triggers
    UpdatingPersonaTriggers { id: i64, name: String, display_name: Option<String> },
    /// Updating persona - waiting for new prompt
    UpdatingPersonaPrompt { id: i64, name: String, display_name: Option<String>, triggers: Option<String> },
    /// Deleting persona - waiting for confirmation
    DeletingPersonaConfirm { id: i64 },
    /// Setting keyword triggers
    SettingKeywords,
    /// Importing persona from JSON
    ImportingPersona,
    /// Broadcasting message to all chats
    Broadcasting,
}

/// Queue statistics for monitoring
#[derive(Clone, Debug, Default)]
pub struct QueueStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub queue_timeouts: u64,
    pub avg_response_time_ms: u64,
}

/// Bot info from Telegram API
#[derive(Clone, Debug)]
pub struct BotInfo {
    pub id: u64,
    pub username: String,
    pub first_name: String,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub llm_client: LlmClient,
    pub web_search: WebSearchClient,
    pub voice_client: VoiceClient,
    pub dialogues: DialogueState,
    pub db_pool: SqlitePool,
    pub admin_cache: AdminCache,
    pub rate_limiter: RateLimiter,
    pub wizard_states: WizardStates,
    pub llm_semaphore: Arc<Semaphore>,
    pub queue_stats: Arc<Mutex<QueueStats>>,
    pub keyword_triggers: Arc<Mutex<HashMap<ChatId, Vec<String>>>>,
    pub security_tracker: Arc<SecurityTracker>,
    pub paused: Arc<AtomicBool>,
    pub bot_info: Arc<Mutex<Option<BotInfo>>>,
    pub pending_messages: PendingMessages,
    pub user_rate_limits: UserRateLimit,
}

impl AppState {
    pub fn new(config: Config, db_pool: SqlitePool) -> Self {
        let config_arc = Arc::new(config);
        let max_concurrent_llm = config_arc.max_concurrent_llm_requests.unwrap_or(3);
        
        // Security config from environment or defaults
        let security_config = SecurityConfig {
            strike_threshold: 30,
            max_strikes: 3,
            block_duration: std::time::Duration::from_secs(300), // 5 min
            strike_window: std::time::Duration::from_secs(3600),  // 1 hour
        };
        
        Self {
            config: config_arc.clone(),
            llm_client: LlmClient::new(config_arc.ollama_url.clone()),
            web_search: WebSearchClient::new(),
            voice_client: VoiceClient::new(config_arc.whisper_url.clone()),
            dialogues: Arc::new(Mutex::new(HashMap::new())),
            db_pool,
            admin_cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
            wizard_states: Arc::new(Mutex::new(HashMap::new())),
            llm_semaphore: Arc::new(Semaphore::new(max_concurrent_llm)),
            queue_stats: Arc::new(Mutex::new(QueueStats::default())),
            keyword_triggers: Arc::new(Mutex::new(HashMap::new())),
            security_tracker: Arc::new(SecurityTracker::new(security_config)),
            paused: Arc::new(AtomicBool::new(false)),
            bot_info: Arc::new(Mutex::new(None)),
            pending_messages: Arc::new(Mutex::new(HashMap::new())),
            user_rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check user rate limit (max 5 responses per minute)
    pub async fn check_user_rate_limit(&self, user_id: u64) -> bool {
        let mut limits = self.user_rate_limits.lock().await;
        let now = Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);
        
        let timestamps = limits.entry(user_id).or_insert_with(Vec::new);
        
        // Remove old timestamps
        timestamps.retain(|t| *t > one_minute_ago);
        
        // Check if over limit (5 per minute)
        if timestamps.len() >= 5 {
            return false; // Rate limited
        }
        
        // Add new timestamp
        timestamps.push(now);
        true // Allowed
    }

    /// Set bot info from Telegram API
    pub async fn set_bot_info(&self, info: BotInfo) {
        let mut bot_info = self.bot_info.lock().await;
        *bot_info = Some(info);
    }

    /// Get bot's first name (display name)
    pub async fn get_bot_name(&self) -> String {
        let bot_info = self.bot_info.lock().await;
        bot_info.as_ref()
            .map(|i| i.first_name.clone())
            .unwrap_or_else(|| self.config.bot_name.clone())
    }

    /// Get bot's username
    pub async fn get_bot_username(&self) -> Option<String> {
        let bot_info = self.bot_info.lock().await;
        bot_info.as_ref().map(|i| i.username.clone())
    }

    /// Check if bot info is loaded
    pub async fn has_bot_info(&self) -> bool {
        let bot_info = self.bot_info.lock().await;
        bot_info.is_some()
    }

    /// Check if bot is paused
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Set bot paused state
    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
    }

    /// Get wizard state for a chat
    pub async fn get_wizard_state(&self, chat_id: ChatId) -> Option<WizardState> {
        let states = self.wizard_states.lock().await;
        states.get(&chat_id).cloned()
    }

    /// Set wizard state for a chat
    pub async fn set_wizard_state(&self, chat_id: ChatId, state: WizardState) {
        let mut states = self.wizard_states.lock().await;
        states.insert(chat_id, state);
    }

    /// Clear wizard state for a chat
    pub async fn clear_wizard_state(&self, chat_id: ChatId) {
        let mut states = self.wizard_states.lock().await;
        states.remove(&chat_id);
    }

    /// Update queue statistics
    pub async fn update_queue_stats(&self, success: bool, response_time_ms: u64) {
        let mut stats = self.queue_stats.lock().await;
        stats.total_requests += 1;
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        // Rolling average
        stats.avg_response_time_ms = (stats.avg_response_time_ms * (stats.total_requests - 1) + response_time_ms) / stats.total_requests;
    }
}
