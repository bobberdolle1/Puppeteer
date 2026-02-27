use crate::{
    db::{AccountRepository, MessageRepository, MessageRole, NewMessage},
    AppState,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Ollama API client
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Call Ollama chat API
    pub async fn chat(&self, request: OllamaChatRequest) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error {}: {}", status, error_text);
        }

        // Ollama streams responses by default, we need to collect all chunks
        let text = response.text().await.context("Failed to read response")?;

        // Parse the last line (Ollama sends newline-delimited JSON)
        let mut final_response = String::new();
        for line in text.lines() {
            if let Ok(chunk) = serde_json::from_str::<OllamaChatResponse>(line) {
                if let Some(content) = chunk.message.content {
                    final_response.push_str(&content);
                }
            }
        }

        if final_response.is_empty() {
            anyhow::bail!("Empty response from Ollama");
        }

        Ok(final_response)
    }
}

#[derive(Debug, Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessageResponse,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageResponse {
    role: String,
    content: Option<String>,
}

/// Generate a response using Ollama with conversation context
pub async fn generate_response(
    state: &AppState,
    account_id: i64,
    chat_id: i64,
    incoming_text: &str,
) -> Result<String> {
    // 1. Save the incoming user message
    let user_message = NewMessage {
        account_id,
        chat_id,
        role: MessageRole::User,
        content: incoming_text.to_string(),
    };

    MessageRepository::create(&state.db_pool, user_message)
        .await
        .context("Failed to save user message")?;

    tracing::debug!(
        "Saved user message for account {} in chat {}",
        account_id,
        chat_id
    );

    // 2. Fetch the account to get the system prompt
    let account = AccountRepository::get_by_id(&state.db_pool, account_id)
        .await?
        .context("Account not found")?;

    // 3. Fetch recent conversation history (last 10 messages)
    let history = MessageRepository::get_recent_messages(&state.db_pool, account_id, chat_id, 10)
        .await
        .context("Failed to fetch message history")?;

    // 4. Build the messages array for Ollama
    let mut messages = Vec::new();

    // Add system prompt
    messages.push(OllamaMessage {
        role: "system".to_string(),
        content: account.system_prompt.clone(),
    });

    // Add conversation history
    for msg in history {
        messages.push(OllamaMessage {
            role: msg.role,
            content: msg.content,
        });
    }

    tracing::debug!(
        "Built context with {} messages for account {}",
        messages.len(),
        account_id
    );

    // 5. Call Ollama API
    let ollama_client = OllamaClient::new(state.config.ollama_url.clone());

    let request = OllamaChatRequest {
        model: state.config.ollama_model.clone(),
        messages,
        stream: true,
    };

    let response_text = ollama_client
        .chat(request)
        .await
        .context("Failed to generate response from Ollama")?;

    tracing::debug!(
        "Generated response ({} chars) for account {}",
        response_text.len(),
        account_id
    );

    // 6. Save the assistant's response
    let assistant_message = NewMessage {
        account_id,
        chat_id,
        role: MessageRole::Assistant,
        content: response_text.clone(),
    };

    MessageRepository::create(&state.db_pool, assistant_message)
        .await
        .context("Failed to save assistant message")?;

    Ok(response_text)
}
