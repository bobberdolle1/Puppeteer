use crate::{
    ai::ollama::OllamaClient,
    db::{AccountRepository, MessageRole, NewMessage},
    state::{AppState, UserbotHandle},
};
use anyhow::{Context, Result};
use rand::Rng;
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
        if let Err(e) = run_userbot_loop(state_clone.clone(), account_clone, client, shutdown_tx).await {
            tracing::error!("Userbot {} error: {}", account_id, e);
            
            // Notify owner about error
            if let Err(notify_err) = notify_owner(&state_clone, &format!("❌ Userbot {} error: {}", account_id, e)).await {
                tracing::error!("Failed to notify owner: {}", notify_err);
            }
        }
    });

    tracing::info!("Userbot {} started successfully", account_id);
    Ok(())
}

async fn run_userbot_loop(
    state: AppState,
    account: crate::db::models::Account,
    client: Arc<Mutex<TdClient>>,
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
                // Process updates from TDLib
                // In real implementation, we'd receive updates via TDLib's update mechanism
                // For now, this is a placeholder for the event loop
            }
        }
    }

    tracing::info!("Userbot {} event loop stopped", account.id);
    Ok(())
}

/// Handle incoming message with humanization
async fn handle_incoming_message(
    state: &AppState,
    account: &crate::db::models::Account,
    client: &Arc<Mutex<TdClient>>,
    message: &Message,
) -> Result<()> {
    // Extract message details using getters
    let chat_id = message.chat_id();
    let message_id = message.id();
    let message_date = message.date();
    
    // Get message text
    let text = match message.content() {
        MessageContent::MessageText(msg_text) => msg_text.text().text().to_string(),
        _ => return Ok(()), // Ignore non-text messages for now
    };

    // Check if message is too old
    let now = chrono::Utc::now().timestamp();
    let message_age = now - message_date as i64;
    if message_age > account.ignore_old_messages_sec {
        tracing::debug!("Ignoring old message ({}s old) in chat {}", message_age, chat_id);
        return Ok(());
    }

    // Check if chat is allowed
    if !account.is_chat_allowed(chat_id) {
        return Ok(());
    }

    // Determine if this is a private chat
    let is_private = chat_id > 0;

    // Decide whether to respond
    let should_respond = if is_private && account.always_respond_in_pm == 1 {
        true
    } else {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..100) < account.reply_probability
    };

    if !should_respond {
        tracing::debug!("Skipping message in chat {} (probability check)", chat_id);
        return Ok(());
    }

    // Calculate humanized delays
    let response_delay = calculate_response_delay(account, &text);
    let typing_duration = calculate_typing_duration(account, &text);

    // Wait before starting to "read" the message
    tokio::time::sleep(tokio::time::Duration::from_secs(response_delay as u64)).await;

    // Send typing indicator
    let client_lock = client.lock().await;
    let send_action = SendChatAction::builder()
        .chat_id(chat_id)
        .action(ChatAction::Typing(ChatActionTyping::builder().build()))
        .build();
    
    if let Err(e) = client_lock.send_chat_action(&send_action).await {
        tracing::warn!("Failed to send typing indicator: {}", e);
    }
    drop(client_lock);

    // "Type" the response
    tokio::time::sleep(tokio::time::Duration::from_secs(typing_duration as u64)).await;

    // Generate AI response
    let response_text = match generate_ai_response(state, account, chat_id, &text).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to generate AI response: {}", e);
            // Notify owner, but don't send error to chat
            notify_owner(state, &format!("⚠️ Userbot {} failed to generate response: {}", account.id, e)).await?;
            return Ok(());
        }
    };

    // Decide whether to use reply or regular message
    let use_reply = if is_private {
        false // Never use reply in private chats
    } else {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..100) < account.use_reply_probability
    };

    // Send the message
    let client_lock = client.lock().await;
    
    let input_message = InputMessageContent::InputMessageText(
        InputMessageText::builder()
            .text(FormattedText::builder().text(response_text.clone()).build())
            .build()
    );

    let mut send_message_builder = SendMessage::builder();
    send_message_builder
        .chat_id(chat_id)
        .input_message_content(input_message);

    // Add reply if needed
    if use_reply {
        send_message_builder.reply_to_message_id(message_id);
    }

    let send_message = send_message_builder.build();

    if let Err(e) = client_lock.send_message(&send_message).await {
        tracing::error!("Failed to send message: {}", e);
        notify_owner(state, &format!("❌ Userbot {} failed to send message: {}", account.id, e)).await?;
        return Ok(());
    }

    drop(client_lock);

    // Save to message history
    let new_message = NewMessage {
        account_id: account.id,
        chat_id,
        role: MessageRole::Assistant,
        content: response_text,
    };
    
    if let Err(e) = AccountRepository::add_message(&state.db_pool, new_message).await {
        tracing::warn!("Failed to save message to history: {}", e);
    }

    tracing::info!("Userbot {} responded in chat {}", account.id, chat_id);
    Ok(())
}

/// Calculate response delay based on message length and account settings
fn calculate_response_delay(account: &crate::db::models::Account, text: &str) -> i64 {
    let mut rng = rand::thread_rng();
    let base_delay = rng.gen_range(account.min_response_delay_sec..=account.max_response_delay_sec);
    
    // Add extra delay for longer messages (simulating reading time)
    let reading_delay = (text.len() / 100) as i64; // ~1 sec per 100 chars
    
    base_delay + reading_delay
}

/// Calculate typing duration based on message length and typing speed
fn calculate_typing_duration(account: &crate::db::models::Account, response: &str) -> i64 {
    // typing_speed_cpm = characters per minute
    let chars = response.len() as i64;
    let minutes = chars as f64 / account.typing_speed_cpm as f64;
    let seconds = (minutes * 60.0) as i64;
    
    // Add some randomness (±20%)
    let mut rng = rand::thread_rng();
    let variance = (seconds as f64 * 0.2) as i64;
    let final_duration = seconds + rng.gen_range(-variance..=variance);
    
    // Clamp between 1 and 30 seconds
    final_duration.max(1).min(30)
}

/// Generate AI response using Ollama
async fn generate_ai_response(
    state: &AppState,
    account: &crate::db::models::Account,
    chat_id: i64,
    user_message: &str,
) -> Result<String> {
    // Get recent message history
    let history = AccountRepository::get_recent_messages(&state.db_pool, account.id, chat_id, 10).await?;
    
    // Build conversation context
    let mut messages = vec![];
    
    // Add system prompt
    messages.push(crate::ai::ollama::OllamaMessage {
        role: "system".to_string(),
        content: account.system_prompt.clone(),
    });
    
    // Add history
    for msg in history {
        messages.push(crate::ai::ollama::OllamaMessage {
            role: msg.role,
            content: msg.content,
        });
    }
    
    // Add current user message
    messages.push(crate::ai::ollama::OllamaMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });
    
    // Generate response
    let ollama_client = crate::ai::ollama::OllamaClient::new(state.config.ollama_url.clone());
    let request = crate::ai::ollama::OllamaChatRequest {
        model: state.config.ollama_model.clone(),
        messages,
        stream: true,
    };
    
    let response = ollama_client.chat(request).await?;
    
    Ok(response)
}

/// Notify owner about system events (errors, warnings, etc.)
async fn notify_owner(state: &AppState, message: &str) -> Result<()> {
    use teloxide::prelude::*;
    use teloxide::types::ChatId;
    
    let bot = Bot::new(&state.config.bot_token);
    
    for owner_id in &state.config.owner_ids {
        if let Err(e) = bot.send_message(ChatId(*owner_id), message).await {
            tracing::error!("Failed to notify owner {}: {}", owner_id, e);
        }
    }
    
    Ok(())
}

