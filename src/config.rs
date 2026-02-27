use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Telegram Bot API token for the admin bot
    pub bot_token: String,
    
    /// List of Telegram user IDs allowed to control the admin bot
    pub owner_ids: Vec<i64>,
    
    /// SQLite database URL
    pub database_url: String,
    
    /// Ollama API endpoint
    pub ollama_url: String,
    
    /// Telegram API ID (for MTProto)
    pub telegram_api_id: i32,
    
    /// Telegram API Hash (for MTProto)
    pub telegram_api_hash: String,
    
    /// Default Ollama model to use
    pub ollama_model: String,
    
    /// Whisper API endpoint (optional, for voice transcription)
    pub whisper_url: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let bot_token = env::var("TELOXIDE_TOKEN")
            .context("TELOXIDE_TOKEN must be set")?;

        let owner_ids_str = env::var("OWNER_IDS")
            .context("OWNER_IDS must be set (comma-separated list of user IDs)")?;
        
        let owner_ids: Vec<i64> = owner_ids_str
            .split(',')
            .map(|s| s.trim().parse::<i64>())
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse OWNER_IDS")?;

        if owner_ids.is_empty() {
            anyhow::bail!("OWNER_IDS must contain at least one user ID");
        }

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/puppeteer.db".to_string());

        let ollama_url = env::var("OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let telegram_api_id = env::var("TELEGRAM_API_ID")
            .context("TELEGRAM_API_ID must be set")?
            .parse::<i32>()
            .context("TELEGRAM_API_ID must be a valid integer")?;

        let telegram_api_hash = env::var("TELEGRAM_API_HASH")
            .context("TELEGRAM_API_HASH must be set")?;

        let ollama_model = env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3.2".to_string());

        let whisper_url = env::var("WHISPER_URL").ok();

        Ok(Config {
            bot_token,
            owner_ids,
            database_url,
            ollama_url,
            telegram_api_id,
            telegram_api_hash,
            ollama_model,
            whisper_url,
        })
    }

    /// Check if a user ID is an owner
    pub fn is_owner(&self, user_id: i64) -> bool {
        self.owner_ids.contains(&user_id)
    }
}
