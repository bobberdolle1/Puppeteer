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
