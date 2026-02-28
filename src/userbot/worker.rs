use crate::{
    db::{AccountRepository, MessageRole, NewMessage},
    state::{AppState, UserbotHandle},
};
use anyhow::{Context, Result};
use rust_tdlib::{
    client::{tdlib_client::TdJson, Client, ConsoleAuthStateHandler, Worker},
    types::*,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

type TdClient = Client<TdJson>;
type TdWorker = Worker<ConsoleAuthStateHandler, TdJson>;

// Rate limiting: track message timestamps per user
lazy_static::lazy_static! {
    static ref USER_MESSAGE_TIMESTAMPS: Arc<RwLock<HashMap<i64, Vec<i64>>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

// Casual responses for stickers
const STICKER_RESPONSES: &[&str] = &["ахах", "жиза", "норм", "кек", "лол", "хд"];

/// Default advanced adaptive system prompt for human-like behavior with extreme dryness
/// NOTE: This is a fallback. For diverse "horde" behavior, use personas::generate_random_persona()
/// or personas::generate_persona_by_name() when creating new accounts.
pub const DEFAULT_SYSTEM_PROMPT: &str = r#"[КТО ТЫ]
Ты обычный, немного уставший пользователь Telegram. Ты общаешься максимально сухо, без эмоций и без попыток быть вежливым.

[ТВОЙ СТИЛЬ (СТРОГИЕ ПРАВИЛА)]
- НИКАКИХ ЭМОДЗИ. Вообще. Максимум, что ты можешь использовать для эмоций — это скобки ")" или "(", либо текстовые реакции: "пхпх", "мда", "треш", "жесть".
- Короткие ответы. Никакой воды. Строчная буква, без точек в конце.
- Если мыслей несколько — разделяй их символом `||`.
- Если собеседник пишет что-то, не требующее ответа (например: "ок", "понял", "спс", "давай", "до завтра"), ты ОБЯЗАН вернуть ровно одно слово: `<IGNORE>`. Это команда системе ничего не отправлять. Не пиши "пожалуйста" или "обращайся". Просто `<IGNORE>`.
- Если собеседник пишет откровенный бред или просто кидает гифку без контекста, реагируй сухо: "и че", "мда", "?" или тоже верни `<IGNORE>`.

[ПРИМЕРЫ]
User: слушай, я там код запушил, глянешь?
Assistant: да, ща посмотрю || минут через 20

User: все, я пофиксил баг
Assistant: ок

User: спасибо бро выручил
Assistant: <IGNORE>

User: [Пользователь отправил стикер]
Assistant: <IGNORE>

User: ну и че мы делать будем с этой базой данных?
Assistant: хз вообще || надо думать || я бы снес ее нахрен и заново поднял"#;

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

    // Create a channel to receive updates from the worker
    let (_tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<Update>>();
    
    // Spawn a task to receive updates from TDLib
    let client_clone = client.clone();
    let _account_id = account.id;
    tokio::spawn(async move {
        loop {
            let client_lock = client_clone.lock().await;
            // Note: In rust-tdlib 0.4, updates are typically received through the Worker
            // For now, we'll use a simple polling approach
            // The actual implementation depends on the specific rust-tdlib API
            drop(client_lock);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                tracing::info!("Userbot {} received shutdown signal", account.id);
                break;
            }
            Some(update) = rx.recv() => {
                // Process the update
                if let Err(e) = process_update(&state, &account, &client, update).await {
                    tracing::error!("Error processing update for userbot {}: {}", account.id, e);
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                // Keep alive
            }
        }
    }

    tracing::info!("Userbot {} event loop stopped", account.id);
    Ok(())
}

/// Process a TDLib update
async fn process_update(
    state: &AppState,
    account: &crate::db::models::Account,
    client: &Arc<Mutex<TdClient>>,
    update: Box<Update>,
) -> Result<()> {
    match update.as_ref() {
        Update::NewMessage(new_message) => {
            let message = new_message.message();
            
            // Ignore outgoing messages (sent by this userbot)
            if message.is_outgoing() {
                return Ok(());
            }
            
            // Handle incoming message with humanization
            handle_incoming_message(state, account, client, message).await?;
        }
        Update::MessageContent(msg_content) => {
            // Message content was edited - we can ignore this for now
            tracing::debug!("Message content updated in chat {}", msg_content.chat_id());
        }
        Update::AuthorizationState(auth_state) => {
            // Handle authorization state changes
            match auth_state.authorization_state() {
                AuthorizationState::Ready(_) => {
                    tracing::info!("Userbot {} is authorized and ready", account.id);
                }
                AuthorizationState::Closed(_) => {
                    tracing::warn!("Userbot {} authorization closed", account.id);
                }
                _ => {}
            }
        }
        _ => {
            // Ignore other update types for now
        }
    }
    
    Ok(())
}

/// Handle incoming message with humanization
/// Handle incoming message with extreme humanization
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

    // Get sender user ID for rate limiting
    let sender_id = match message.sender_id() {
        MessageSender::User(user) => user.user_id(),
        _ => 0,
    };

    // Rate limiting: check if user is spamming (>5 messages per minute)
    if sender_id != 0 {
        let now = chrono::Utc::now().timestamp();
        let mut timestamps_lock = USER_MESSAGE_TIMESTAMPS.write().await;
        let user_timestamps = timestamps_lock.entry(sender_id).or_insert_with(Vec::new);
        
        // Remove timestamps older than 60 seconds
        user_timestamps.retain(|&ts| now - ts < 60);
        
        // Check if user exceeded rate limit
        if user_timestamps.len() >= 5 {
            tracing::debug!("Rate limit exceeded for user {} in chat {}", sender_id, chat_id);
            return Ok(());
        }
        
        // Add current timestamp
        user_timestamps.push(now);
    }

    // Process message content and get text + optional media description
    let (text, is_sticker) = match message.content() {
        MessageContent::MessageText(msg_text) => {
            (msg_text.text().text().to_string(), false)
        }
        MessageContent::MessagePhoto(photo) => {
            // Process photo with vision
            match process_photo(state, client, photo).await {
                Ok(description) => (format!("[Изображение]: {}", description), false),
                Err(e) => {
                    tracing::warn!("Failed to process photo: {}", e);
                    ("[Пользователь отправил фото]".to_string(), false)
                }
            }
        }
        MessageContent::MessageAnimation(animation) => {
            // Process GIF/animation with vision (extract 3 frames)
            match process_animation(state, client, animation).await {
                Ok(description) => (format!("[GIF/Анимация]: {}", description), false),
                Err(e) => {
                    tracing::warn!("Failed to process animation: {}", e);
                    ("[Пользователь отправил GIF]".to_string(), false)
                }
            }
        }
        MessageContent::MessageSticker(_sticker) => {
            // Stickers get casual responses with low probability
            ("[Пользователь отправил стикер]".to_string(), true)
        }
        MessageContent::MessageVoiceNote(voice) => {
            // Process voice with Whisper
            match process_voice(state, client, voice).await {
                Ok(transcription) => (format!("[Голосовое сообщение]: {}", transcription), false),
                Err(e) => {
                    tracing::warn!("Failed to process voice: {}", e);
                    ("[Пользователь отправил голосовое сообщение]".to_string(), false)
                }
            }
        }
        MessageContent::MessageVideoNote(video_note) => {
            // Process video circle (extract 3 frames)
            match process_video_note(state, client, video_note).await {
                Ok(description) => (format!("[Видео кружок]: {}", description), false),
                Err(e) => {
                    tracing::warn!("Failed to process video note: {}", e);
                    ("[Пользователь отправил видеосообщение]".to_string(), false)
                }
            }
        }
        MessageContent::MessageVideo(_) => {
            ("[Пользователь отправил видео]".to_string(), false)
        }
        _ => {
            // Ignore other message types
            return Ok(());
        }
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

    // Calculate reply probability (lower for stickers)
    let adjusted_probability = if is_sticker {
        account.reply_probability / 4 // Very low probability for stickers
    } else {
        account.reply_probability
    };

    // Decide whether to respond
    let should_respond = if is_private && account.always_respond_in_pm == 1 {
        true
    } else {
        rand::random::<u8>() as i64 % 100 < adjusted_probability
    };

    if !should_respond {
        tracing::debug!("Skipping message in chat {} (probability check)", chat_id);
        return Ok(());
    }

    // IMMEDIATELY mark message as read (simulate instant read receipt)
    let client_lock = client.lock().await;
    let view_messages = ViewMessages::builder()
        .chat_id(chat_id)
        .message_ids(vec![message_id])
        .force_read(true)
        .build();

    if let Err(e) = client_lock.view_messages(&view_messages).await {
        tracing::warn!("Failed to mark message as read: {}", e);
    }
    drop(client_lock);

    // Random "Read Delay" - simulate user reading and thinking (5-60 seconds)
    let read_delay = 5 + (rand::random::<u8>() % 56) as u64; // 5-60 seconds
    tracing::debug!("Read delay: {}s for chat {}", read_delay, chat_id);
    tokio::time::sleep(tokio::time::Duration::from_secs(read_delay)).await;

    // Calculate additional response delay based on message length
    let response_delay = calculate_response_delay(account, &text);
    tokio::time::sleep(tokio::time::Duration::from_secs(response_delay as u64)).await;

    // Generate AI response
    let response_text = if is_sticker {
        // Casual response for stickers
        let idx = rand::random::<usize>() % STICKER_RESPONSES.len();
        STICKER_RESPONSES[idx].to_string()
    } else {
        match generate_ai_response(state, account, chat_id, &text).await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("Failed to generate AI response: {}", e);
                // Notify owner, but don't send error to chat
                notify_owner(state, &format!("⚠️ Userbot {} failed to generate response: {}", account.id, e)).await?;
                return Ok(());
            }
        }
    };

    // Check if AI returned <IGNORE> - if so, don't send anything
    if response_text.trim() == "<IGNORE>" {
        tracing::info!("Userbot {} ignoring message in chat {} (AI returned <IGNORE>)", account.id, chat_id);
        return Ok(());
    }

    // Split response by || for multi-texting
    let message_chunks: Vec<&str> = response_text
        .split("||")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "<IGNORE>")
        .collect();

    // If no chunks (empty response), skip
    if message_chunks.is_empty() {
        tracing::warn!("Empty response after splitting for userbot {}", account.id);
        return Ok(());
    }

    // Decide whether to use reply or regular message
    let use_reply = if is_private {
        false // Never use reply in private chats
    } else {
        // In group chats, use reply only if:
        // 1. The message is a reply to our previous message (active dialogue)
        // 2. Or based on probability (but less often)
        let is_reply_to_us = message.reply_to_message_id() != 0; // Check if replying to someone

        if is_reply_to_us {
            // If someone replied to us, always use reply back
            true
        } else {
            // Otherwise, use reply based on probability (but make it lower for natural feel)
            rand::random::<u8>() as i64 % 100 < (account.use_reply_probability / 2) // Half the probability for non-dialogue messages
        }
    };

    // 20% chance of "distracted typist" behavior
    let is_distracted = (rand::random::<u8>() % 100) < 20;

    if is_distracted {
        tracing::debug!("Distracted typist behavior triggered for userbot {}", account.id);

        // Start typing
        let client_lock = client.lock().await;
        let send_action = SendChatAction::builder()
            .chat_id(chat_id)
            .action(ChatAction::Typing(ChatActionTyping::builder().build()))
            .build();

        if let Err(e) = client_lock.send_chat_action(&send_action).await {
            tracing::warn!("Failed to send typing indicator: {}", e);
        }
        drop(client_lock);

        // Type for a bit (2-4 seconds)
        let distracted_typing_duration = 2 + (rand::random::<u8>() % 3) as u64; // 2-4 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(distracted_typing_duration)).await;

        // Cancel typing (send cancel action)
        let client_lock = client.lock().await;
        let cancel_action = SendChatAction::builder()
            .chat_id(chat_id)
            .action(ChatAction::Cancel(ChatActionCancel::builder().build()))
            .build();

        if let Err(e) = client_lock.send_chat_action(&cancel_action).await {
            tracing::warn!("Failed to cancel typing: {}", e);
        }
        drop(client_lock);

        // Pause (distracted - 3-10 seconds)
        let distracted_pause = 3 + (rand::random::<u8>() % 8) as u64; // 3-10 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(distracted_pause)).await;
    }

    // Send each chunk as a separate message with typing indicators
    for (idx, chunk) in message_chunks.iter().enumerate() {
        // Calculate typing duration for this chunk
        let typing_duration = calculate_typing_duration(account, chunk);

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

        // Send the message chunk
        let client_lock = client.lock().await;

        let input_message = InputMessageContent::InputMessageText(
            InputMessageText::builder()
                .text(FormattedText::builder().text(chunk.to_string()).build())
                .build()
        );

        let mut send_message_builder = SendMessage::builder();
        send_message_builder
            .chat_id(chat_id)
            .input_message_content(input_message);

        // Add reply only to the first chunk if needed
        if use_reply && idx == 0 {
            send_message_builder.reply_to_message_id(message_id);
        }

        let send_message = send_message_builder.build();

        if let Err(e) = client_lock.send_message(&send_message).await {
            tracing::error!("Failed to send message chunk {}: {}", idx, e);
            notify_owner(state, &format!("❌ Userbot {} failed to send message chunk: {}", account.id, e)).await?;
            drop(client_lock);
            return Ok(());
        }

        drop(client_lock);

        // Add a small random pause between chunks (0.5s - 1.5s)
        if idx < message_chunks.len() - 1 {
            let pause_ms = 500 + (rand::random::<u16>() % 1001) as u64; // 500-1500ms
            tokio::time::sleep(tokio::time::Duration::from_millis(pause_ms)).await;
        }
    }

    // Save to message history
    let new_message = NewMessage {
        account_id: account.id,
        chat_id,
        role: MessageRole::Assistant,
        content: response_text.clone(),
    };

    if let Err(e) = AccountRepository::add_message(&state.db_pool, new_message).await {
        tracing::warn!("Failed to save message to history: {}", e);
    }

    tracing::info!("Userbot {} responded in chat {} with {} chunks", account.id, chat_id, message_chunks.len());
    Ok(())
}

/// Calculate response delay based on message length and account settings
fn calculate_response_delay(account: &crate::db::models::Account, text: &str) -> i64 {
    let min = account.min_response_delay_sec;
    let max = account.max_response_delay_sec;
    let range = max - min;
    let base_delay = min + (rand::random::<u8>() as i64 % (range + 1));
    
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
    let variance = (seconds as f64 * 0.2) as i64;
    let random_offset = (rand::random::<u8>() as i64 % (variance * 2 + 1)) - variance;
    let final_duration = seconds + random_offset;
    
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
    let http_client = reqwest::Client::new();
    
    // Check if web search is needed
    let search_context = match crate::ai::should_search(
        &http_client,
        &state.config.ollama_url,
        &state.config.ollama_model,
        user_message,
    ).await {
        Ok(Some(query)) => {
            tracing::info!("Web search triggered for query: {}", query);
            
            // Perform search
            match crate::ai::search_web(&http_client, &query, 3).await {
                Ok(results) => {
                    if !results.is_empty() {
                        Some(crate::ai::format_search_results(&results))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    tracing::warn!("Web search failed: {}", e);
                    None
                }
            }
        }
        Ok(None) => None,
        Err(e) => {
            tracing::warn!("Search detection failed: {}", e);
            None
        }
    };
    
    // Generate embedding for current message for RAG retrieval
    let query_embedding = match crate::ai::generate_embedding(
        &http_client,
        &state.config.ollama_url,
        &state.config.ollama_model,
        user_message,
    ).await {
        Ok(emb) => Some(emb),
        Err(e) => {
            tracing::warn!("Failed to generate query embedding: {}", e);
            None
        }
    };
    
    // Retrieve relevant memories if embedding was successful
    let memory_context = if let Some(ref embedding) = query_embedding {
        match crate::ai::retrieve_memories(&state.db_pool, account.id, chat_id, embedding, 3).await {
            Ok(memories) => {
                if !memories.is_empty() {
                    let mut context = String::from("[ВСПЛЫВШИЕ ВОСПОМИНАНИЯ О ПРОШЛЫХ ДИАЛОГАХ]\n\n");
                    for (i, memory) in memories.iter().enumerate() {
                        if memory.similarity > 0.5 { // Only include relevant memories
                            context.push_str(&format!("{}. {}\n", i + 1, memory.content));
                        }
                    }
                    Some(context)
                } else {
                    None
                }
            }
            Err(e) => {
                tracing::warn!("Failed to retrieve memories: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    // Get recent message history
    let history = AccountRepository::get_recent_messages(&state.db_pool, account.id, chat_id, 10).await?;
    
    // Build conversation context
    let mut messages = vec![];
    
    // Add system prompt
    messages.push(crate::ai::ollama::OllamaMessage {
        role: "system".to_string(),
        content: account.system_prompt.clone(),
    });
    
    // Add memory context if available
    if let Some(ref mem_ctx) = memory_context {
        messages.push(crate::ai::ollama::OllamaMessage {
            role: "system".to_string(),
            content: mem_ctx.clone(),
        });
    }
    
    // Add search results if available
    if let Some(ref search_ctx) = search_context {
        messages.push(crate::ai::ollama::OllamaMessage {
            role: "system".to_string(),
            content: search_ctx.clone(),
        });
    }
    
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
    
    // Store significant messages in long-term memory
    if let Some(embedding) = query_embedding {
        // Only store if message is substantial (>10 chars)
        if user_message.len() > 10 {
            if let Err(e) = crate::ai::store_memory(
                &state.db_pool,
                account.id,
                chat_id,
                user_message,
                &embedding,
            ).await {
                tracing::warn!("Failed to store memory: {}", e);
            }
            
            // Cleanup old memories periodically (every 100th message)
            if rand::random::<u8>() % 100 == 0 {
                if let Err(e) = crate::ai::cleanup_old_memories(&state.db_pool, account.id, chat_id).await {
                    tracing::warn!("Failed to cleanup old memories: {}", e);
                }
            }
        }
    }
    
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


/// Process photo with vision model
async fn process_photo(
    state: &AppState,
    client: &Arc<Mutex<TdClient>>,
    photo: &MessagePhoto,
) -> Result<String> {
    // Get the largest photo size
    let photo_size = photo.photo().sizes().iter()
        .max_by_key(|s| s.width() * s.height())
        .context("No photo sizes available")?;
    
    let file_id = photo_size.photo().id();
    
    // Download the photo
    let file_path = download_file(client, file_id).await?;
    
    // Read and encode to base64
    let image_bytes = tokio::fs::read(&file_path).await?;
    use base64::Engine;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    
    // Analyze with vision model
    let ollama_client = crate::ai::ollama::OllamaClient::new(state.config.ollama_url.clone());
    let description = ollama_client.vision(
        "llava", // or minicpm-v
        "Опиши что на этом изображении. Будь кратким, 1-2 предложения.",
        vec![base64_image],
    ).await?;
    
    // Clean up temp file
    let _ = tokio::fs::remove_file(file_path).await;
    
    Ok(description)
}

/// Process animation/GIF with vision model (extract 3 frames)
async fn process_animation(
    state: &AppState,
    client: &Arc<Mutex<TdClient>>,
    animation: &MessageAnimation,
) -> Result<String> {
    let file_id = animation.animation().animation().id();
    
    // Download the animation
    let file_path = download_file(client, file_id).await?;
    
    // Extract 3 frames (start, middle, end)
    let frames = extract_video_frames(&file_path, 3).await?;
    
    // Encode frames to base64
    use base64::Engine;
    let mut base64_frames = Vec::new();
    for frame_path in &frames {
        let frame_bytes = tokio::fs::read(frame_path).await?;
        base64_frames.push(base64::engine::general_purpose::STANDARD.encode(&frame_bytes));
    }
    
    // Analyze with vision model
    let ollama_client = crate::ai::ollama::OllamaClient::new(state.config.ollama_url.clone());
    let description = ollama_client.vision(
        "llava",
        "Опиши что происходит в этой гифке/анимации. Будь кратким, 1-2 предложения.",
        base64_frames,
    ).await?;
    
    // Clean up temp files
    let _ = tokio::fs::remove_file(file_path).await;
    for frame_path in frames {
        let _ = tokio::fs::remove_file(frame_path).await;
    }
    
    Ok(description)
}

/// Process voice message with Whisper
async fn process_voice(
    state: &AppState,
    client: &Arc<Mutex<TdClient>>,
    voice: &MessageVoiceNote,
) -> Result<String> {
    let file_id = voice.voice_note().voice().id();
    
    // Download the voice file
    let file_path = download_file(client, file_id).await?;
    
    // Transcribe with Whisper
    let whisper_url = state.config.whisper_url.as_ref()
        .context("Whisper URL not configured")?;
    
    let transcription = crate::ai::whisper::transcribe_audio(
        whisper_url,
        std::path::Path::new(&file_path),
    ).await?;
    
    // Clean up temp file
    let _ = tokio::fs::remove_file(file_path).await;
    
    Ok(transcription)
}

/// Process video note/circle (extract 3 frames)
async fn process_video_note(
    state: &AppState,
    client: &Arc<Mutex<TdClient>>,
    video_note: &MessageVideoNote,
) -> Result<String> {
    let file_id = video_note.video_note().video().id();
    
    // Download the video note
    let file_path = download_file(client, file_id).await?;
    
    // Extract 3 frames (start, middle, end)
    let frames = extract_video_frames(&file_path, 3).await?;
    
    // Encode frames to base64
    use base64::Engine;
    let mut base64_frames = Vec::new();
    for frame_path in &frames {
        let frame_bytes = tokio::fs::read(frame_path).await?;
        base64_frames.push(base64::engine::general_purpose::STANDARD.encode(&frame_bytes));
    }
    
    // Analyze with vision model
    let ollama_client = crate::ai::ollama::OllamaClient::new(state.config.ollama_url.clone());
    let description = ollama_client.vision(
        "llava",
        "Опиши что происходит в этом видео кружке. Будь кратким, 1-2 предложения.",
        base64_frames,
    ).await?;
    
    // Clean up temp files
    let _ = tokio::fs::remove_file(file_path).await;
    for frame_path in frames {
        let _ = tokio::fs::remove_file(frame_path).await;
    }
    
    Ok(description)
}

/// Download file from TDLib
async fn download_file(client: &Arc<Mutex<TdClient>>, file_id: i32) -> Result<String> {
    let client_lock = client.lock().await;
    
    let download_file = DownloadFile::builder()
        .file_id(file_id)
        .priority(32)
        .synchronous(true)
        .build();
    
    let file = client_lock.download_file(&download_file).await?;
    let local_path = file.local().path().to_string();
    
    drop(client_lock);
    
    Ok(local_path)
}

/// Extract frames from video file using ffmpeg
async fn extract_video_frames(video_path: &str, num_frames: usize) -> Result<Vec<String>> {
    use tokio::process::Command;
    
    let mut frame_paths = Vec::new();
    
    // Get video duration first
    let duration_output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            video_path,
        ])
        .output()
        .await?;
    
    let duration_str = String::from_utf8_lossy(&duration_output.stdout);
    let duration: f64 = duration_str.trim().parse()
        .context("Failed to parse video duration")?;
    
    // Extract frames at evenly spaced intervals
    for i in 0..num_frames {
        let timestamp = if num_frames == 1 {
            duration / 2.0
        } else {
            (duration / (num_frames - 1) as f64) * i as f64
        };
        
        let frame_path = format!("/tmp/frame_{}_{}.jpg", 
            std::process::id(), 
            rand::random::<u32>());
        
        Command::new("ffmpeg")
            .args(&[
                "-ss", &timestamp.to_string(),
                "-i", video_path,
                "-vframes", "1",
                "-q:v", "2",
                &frame_path,
            ])
            .output()
            .await?;
        
        frame_paths.push(frame_path);
    }
    
    Ok(frame_paths)
}
