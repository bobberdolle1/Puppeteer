use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a Telegram MTProto account (userbot)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub phone_number: String,
    #[serde(skip)]
    pub session_data: Vec<u8>,
    pub system_prompt: String,
    pub is_active: bool,
    pub reply_probability: i64,
    pub allowed_chats: String,
    pub min_response_delay_sec: i64,
    pub max_response_delay_sec: i64,
    pub typing_speed_cpm: i64,
    pub use_reply_probability: i64,
    pub ignore_old_messages_sec: i64,
    pub always_respond_in_pm: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// Parse allowed_chats JSON into a Vec of chat IDs
    pub fn get_allowed_chats(&self) -> Vec<i64> {
        serde_json::from_str(&self.allowed_chats).unwrap_or_default()
    }

    /// Check if a chat is allowed (empty list = all chats allowed)
    pub fn is_chat_allowed(&self, chat_id: i64) -> bool {
        let allowed = self.get_allowed_chats();
        allowed.is_empty() || allowed.contains(&chat_id)
    }
}

/// Represents a message in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageHistory {
    pub id: i64,
    pub account_id: i64,
    pub chat_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Role of a message in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }
}

impl From<MessageRole> for String {
    fn from(role: MessageRole) -> Self {
        role.as_str().to_string()
    }
}

/// Data for creating a new account
#[derive(Debug, Clone)]
pub struct NewAccount {
    pub phone_number: String,
    pub session_data: Vec<u8>,
    pub system_prompt: String,
}

/// Data for creating a new message history entry
#[derive(Debug, Clone)]
pub struct NewMessage {
    pub account_id: i64,
    pub chat_id: i64,
    pub role: MessageRole,
    pub content: String,
}

/// Bot group for coordinated actions
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BotGroup {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Bot group member
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BotGroupMember {
    pub id: i64,
    pub group_id: i64,
    pub account_id: i64,
    pub created_at: DateTime<Utc>,
}

/// Spam campaign for coordinated attacks
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpamCampaign {
    pub id: i64,
    pub name: String,
    pub group_id: Option<i64>,
    pub target_type: String,
    pub target_id: i64,
    pub message_text: Option<String>,
    pub media_path: Option<String>,
    pub media_type: Option<String>,
    pub repeat_count: i64,
    pub delay_between_ms: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Data for creating a new bot group
#[derive(Debug, Clone)]
pub struct NewBotGroup {
    pub name: String,
    pub description: Option<String>,
}

/// Data for creating a new spam campaign
#[derive(Debug, Clone)]
pub struct NewSpamCampaign {
    pub name: String,
    pub group_id: Option<i64>,
    pub target_type: String,
    pub target_id: i64,
    pub message_text: Option<String>,
    pub media_path: Option<String>,
    pub media_type: Option<String>,
    pub repeat_count: i64,
    pub delay_between_ms: i64,
}
