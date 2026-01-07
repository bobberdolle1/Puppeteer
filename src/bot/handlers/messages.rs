use crate::db;
use crate::logging;
use crate::state::{AppState, DialogueState, PendingBatch, WizardState};
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReplyParameters};
use std::time::Instant;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

const MAX_CONTEXT_MESSAGES: usize = 20;
const MAX_RAG_CHUNKS: u32 = 3;
const DEFAULT_PERSONA_PROMPT: &str = "You are a helpful AI assistant.";
const DEBOUNCE_MS: u64 = 1500; // Wait 1.5 seconds for more messages

pub async fn handle_message(bot: Bot, msg: Message, state: AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let thread_id = msg.thread_id;
    
    // Check for GIF (animation), video_note (circle video), or voice message
    let media_description = if let Some(animation) = msg.animation() {
        if state.config.vision_enabled {
            tracing::debug!(target: "media", "GIF received in chat {}", chat_id);
            
            // Show typing indicator
            let mut typing = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing);
            if let Some(tid) = thread_id {
                typing = typing.message_thread_id(tid);
            }
            let _ = typing.await;
            
            match process_animation(
                &bot,
                &state,
                &animation.file.id.0,
                msg.caption(),
            ).await {
                Ok(desc) => Some(desc),
                Err(e) => {
                    logging::log_error("GIF processing", &e);
                    None
                }
            }
        } else {
            None
        }
    } else if let Some(video_note) = msg.video_note() {
        // Video notes can be processed with voice OR vision (or both)
        if state.config.vision_enabled || state.config.voice_enabled {
            tracing::debug!(target: "media", "Video note received in chat {}", chat_id);
            
            // Show typing indicator
            let mut typing = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing);
            if let Some(tid) = thread_id {
                typing = typing.message_thread_id(tid);
            }
            let _ = typing.await;
            
            match process_video_note(
                &bot,
                &state,
                &video_note.file.id.0,
            ).await {
                Ok(desc) => Some(desc),
                Err(e) => {
                    logging::log_error("Video note processing", &e);
                    None
                }
            }
        } else {
            None
        }
    } else if let Some(voice) = msg.voice() {
        // Voice messages - transcribe with Whisper
        if state.config.voice_enabled {
            tracing::debug!(target: "media", "Voice message received in chat {}", chat_id);
            
            // Show typing indicator
            let mut typing = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing);
            if let Some(tid) = thread_id {
                typing = typing.message_thread_id(tid);
            }
            let _ = typing.await;
            
            match process_voice_message(&bot, &state, &voice.file.id.0).await {
                Ok(transcript) => Some(format!("[Голосовое сообщение]: {}", transcript)),
                Err(e) => {
                    logging::log_error("Voice processing", &e);
                    None
                }
            }
        } else {
            None
        }
    } else if let Some(photos) = msg.photo() {
        // Photo messages - analyze with vision model
        if state.config.vision_enabled {
            tracing::debug!(target: "media", "Photo received in chat {}", chat_id);
            
            // Show typing indicator
            let mut typing = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing);
            if let Some(tid) = thread_id {
                typing = typing.message_thread_id(tid);
            }
            let _ = typing.await;
            
            // Get the largest photo (last in array)
            if let Some(photo) = photos.last() {
                match process_photo(&bot, &state, &photo.file.id.0, msg.caption()).await {
                    Ok(desc) => Some(desc),
                    Err(e) => {
                        logging::log_error("Photo processing", &e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Get text from message OR use media description as context
    let text = msg.text().unwrap_or_default();
    
    // Build effective message content: combine text with media description
    let effective_text = match (&media_description, text.is_empty()) {
        (Some(media_desc), true) => {
            // Only media, no text caption
            format!("[Пользователь отправил медиа]\n{}", media_desc)
        }
        (Some(media_desc), false) => {
            // Media with caption
            format!("[Пользователь отправил медиа с подписью: \"{}\"]\n{}", text, media_desc)
        }
        (None, _) => text.to_string(),
    };
    
    // Skip if no content to process
    if effective_text.is_empty() {
        return Ok(());
    }

    if text.starts_with('/') {
        // Handle commands
        return crate::bot::handlers::commands::handle_command(bot, msg, state).await;
    }

    // Check for active wizard state first
    if let Some(wizard_state) = state.get_wizard_state(chat_id).await {
        return handle_wizard_input(bot, msg, state, wizard_state).await;
    }

    // Check if bot is paused
    if state.is_paused() {
        return Ok(());
    }

    // Lazy load bot info if not available
    if !state.has_bot_info().await {
        if let Ok(me) = bot.get_me().await {
            let bot_info = crate::state::BotInfo {
                id: me.id.0,
                username: me.username.clone().unwrap_or_default(),
                first_name: me.first_name.clone(),
            };
            state.set_bot_info(bot_info).await;
        }
    }

    // --- Save incoming message and generate embedding ---
    save_and_embed_message(&state, &msg).await;

    // --- Get Active Persona ---
    let active_persona = db::get_active_persona(&state.db_pool)
        .await
        .unwrap_or_else(|e| {
            logging::log_error("Persona fetch", &e.to_string());
            None
        });
    
    let persona_prompt = active_persona.as_ref()
        .map(|p| p.prompt.clone())
        .unwrap_or_else(|| DEFAULT_PERSONA_PROMPT.to_string());
    
    // Get persona's display name (fallback to bot name from config)
    let persona_display_name = active_persona.as_ref()
        .and_then(|p| p.display_name.clone());
    
    // Get persona's triggers
    let persona_triggers: Option<Vec<String>> = active_persona.as_ref()
        .and_then(|p| p.triggers.as_ref())
        .map(|t| t.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect());

    // --- Get Chat Settings (for per-chat options like RAG, cooldown) ---
    let chat_settings = db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await
        .unwrap_or_else(|e| {
            tracing::warn!(target: "db", "Failed to get chat settings: {}", e);
            // Return default settings if there's an error
            db::ChatSettings {
                chat_id: chat_id.0,
                auto_reply_enabled: true,
                reply_mode: "mention_only".to_string(),
                cooldown_seconds: 5,
                context_depth: 10,
                rag_enabled: true,
            }
        });

    // Get GLOBAL reply mode from runtime_config (not per-chat)
    let global_reply_mode = db::get_config(&state.db_pool, "reply_mode").await
        .ok()
        .flatten()
        .unwrap_or_else(|| "mention_only".to_string());

    // Check if auto-reply is enabled (still per-chat)
    if !chat_settings.auto_reply_enabled {
        return Ok(());
    }

    // Check reply mode (mention/command vs all messages)
    // In private chats, always reply
    // If someone replies to bot's message, always reply
    // If someone mentions bot by name (or persona's display name), always reply
    // If message contains a keyword trigger (chat or persona), always reply
    // If message contains media (GIF/video_note), always reply
    let is_private = msg.chat.is_private();
    let is_reply_to_bot = msg.reply_to_message().map(|reply| {
        reply.from.as_ref().map(|u| u.is_bot).unwrap_or(false)
    }).unwrap_or(false);
    let has_media = media_description.is_some();
    
    // Check if bot is mentioned by name, persona display name, or username
    let bot_name = state.get_bot_name().await;
    let bot_username = state.get_bot_username().await;
    let text_lower = text.to_lowercase();
    let bot_name_lower = bot_name.to_lowercase();
    
    // Use persona's display name if set, otherwise use bot's default name
    let effective_name = persona_display_name.as_ref()
        .map(|n| n.to_lowercase())
        .unwrap_or_else(|| bot_name_lower.clone());
    
    let is_mentioned_by_name = text_lower.contains(&effective_name) ||
        text_lower.contains(&bot_name_lower) ||
        bot_username.as_ref().map(|u| text.contains(&format!("@{}", u))).unwrap_or(false);
    
    // Check if message contains any keyword trigger (chat-level or persona-level)
    let chat_triggers = state.keyword_triggers.lock().await.get(&chat_id).cloned();
    let is_triggered = {
        // Check chat-level triggers
        let chat_triggered = chat_triggers.as_ref().map(|kw| {
            kw.iter().any(|keyword| text_lower.contains(keyword))
        }).unwrap_or(false);
        
        // Check persona-level triggers
        let persona_triggered = persona_triggers.as_ref().map(|kw| {
            kw.iter().any(|keyword| text_lower.contains(keyword))
        }).unwrap_or(false);
        
        chat_triggered || persona_triggered
    };
    
    let should_reply = if is_private || is_reply_to_bot || is_mentioned_by_name || is_triggered {
        // Always reply: private chat, reply to bot, mention, or trigger
        tracing::info!(target: "auto_reply", "Chat {} will reply: private={}, reply_to_bot={}, mentioned={} (name='{}'), triggered={}", 
            chat_id, is_private, is_reply_to_bot, is_mentioned_by_name, effective_name, is_triggered);
        true
    } else if has_media && global_reply_mode == "all_messages" {
        // Media only triggers reply in "all_messages" mode
        true
    } else if global_reply_mode == "all_messages" {
        // In "all_messages" mode, reply with probability
        use rand::Rng;
        let probability = db::get_config_f64(&state.db_pool, "random_reply_probability", state.config.random_reply_probability).await;
        tracing::info!(target: "auto_reply", "Chat {} mode=all_messages, probability={}", chat_id, probability);
        if probability <= 0.0 {
            tracing::info!(target: "auto_reply", "Skipping: probability is 0");
            false
        } else if probability >= 1.0 {
            tracing::info!(target: "auto_reply", "Replying: probability is 100%");
            true
        } else {
            let roll = rand::rng().random::<f64>();
            let will_reply = roll < probability;
            tracing::info!(target: "auto_reply", "Roll: {:.3} < {:.3} = {}", roll, probability, will_reply);
            will_reply
        }
    } else {
        // For "mention_only" mode, check if bot is mentioned by @username
        tracing::info!(target: "auto_reply", "Chat {} mode=mention_only, checking @username", chat_id);
        let bot_info = bot.get_me().await;
        if let Ok(me) = bot_info {
            let username = me.user.username.as_deref().unwrap_or("");
            text.contains(&format!("@{}", username))
                || text.contains(&format!("/{}", username))
        } else {
            false
        }
    };

    if !should_reply {
        // Still save the message for context, but don't reply
        tracing::info!(target: "auto_reply", "Chat {} NOT replying (mode={}, prob check skipped)", chat_id, global_reply_mode);
        save_and_embed_message(&state, &msg).await;
        return Ok(());
    }

    // Check user rate limit (5 responses per minute)
    let user_id = msg.from.as_ref().map(|u| u.id.0).unwrap_or(0);
    if !state.check_user_rate_limit(user_id).await {
        tracing::debug!(target: "rate_limit", "User {} rate limited", user_id);
        return Ok(());
    }

    // Check cooldown
    if check_cooldown(&state, chat_id).await {
        return Ok(());
    }

    // --- Debounce: collect messages and wait for more ---
    let thread_id = msg.thread_id;
    let batch_key = (chat_id, thread_id);
    let user_name = msg.from.as_ref().map(|u| u.first_name.clone()).unwrap_or_else(|| "User".to_string());
    
    {
        let mut pending = state.pending_messages.lock().await;
        let batch = pending.entry(batch_key).or_insert_with(|| PendingBatch {
            messages: Vec::new(),
            last_message_time: Instant::now(),
            user_id: Some(user_id),
            user_name: user_name.clone(),
        });
        batch.messages.push(effective_text.clone());
        batch.last_message_time = Instant::now();
        
        // If this is not the first message in batch, just add and return
        // The first message handler will process all
        if batch.messages.len() > 1 {
            tracing::debug!(target: "batch", "Added to batch {:?}, total: {}", batch_key, batch.messages.len());
            return Ok(());
        }
    }
    
    // Wait for debounce period
    tokio::time::sleep(std::time::Duration::from_millis(DEBOUNCE_MS)).await;
    
    // Check if more messages arrived during debounce
    let combined_text = {
        let mut pending = state.pending_messages.lock().await;
        if let Some(batch) = pending.remove(&batch_key) {
            // Check if last message was recent (more messages might be coming)
            if batch.last_message_time.elapsed().as_millis() < (DEBOUNCE_MS / 2) as u128 {
                // Put it back and wait more
                pending.insert(batch_key, batch);
                drop(pending);
                tokio::time::sleep(std::time::Duration::from_millis(DEBOUNCE_MS)).await;
                
                // Try again
                let mut pending = state.pending_messages.lock().await;
                pending.remove(&batch_key).map(|b| b.messages.join("\n")).unwrap_or_else(|| effective_text.clone())
            } else {
                batch.messages.join("\n")
            }
        } else {
            effective_text.clone()
        }
    };

    // --- RAG & Context ---
    let long_term_memories = if chat_settings.rag_enabled {
        retrieve_memories(&state, chat_id, &combined_text).await
    } else {
        vec![] // Empty vector if RAG is disabled
    };

    // Use context depth from chat settings
    let short_term_history = get_and_update_history_with_depth(state.dialogues.clone(), &msg, chat_settings.context_depth as usize).await;
    
    // Get effective name for prompt (persona's display_name or bot's default name)
    let bot_name = state.get_bot_name().await;
    let effective_name = persona_display_name.as_ref()
        .unwrap_or(&bot_name);
    let prompt = build_prompt(persona_prompt, long_term_memories, short_term_history, effective_name);

    tracing::trace!(target: "llm", "Prompt for chat {}: {} chars", chat_id, prompt.len());

    // --- Show typing indicator ---
    let mut typing_action = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing);
    if let Some(tid) = thread_id {
        typing_action = typing_action.message_thread_id(tid);
    }
    let _ = typing_action.await;

    // --- Generate Response ---
    let user_name = msg.from.as_ref().map(|u| u.first_name.clone()).unwrap_or_else(|| "User".to_string());
    let text_preview = combined_text.chars().take(50).collect::<String>();
    logging::log_message_received(chat_id.0, &user_name, &text_preview, media_description.is_some());
    
    let start_time = std::time::Instant::now();
    match state.llm_client.generate(&state.config.ollama_chat_model, &prompt, state.config.temperature, state.config.max_tokens).await {
        Ok(response_text) => {
            let response_time = start_time.elapsed().as_millis();

            // Apply human-like behavior rules
            let processed_response = apply_human_behavior_rules(response_text, &state.config.bot_name);

            tracing::debug!(target: "messages", "Response for chat {} in {}ms", chat_id, response_time);
            
            // Try to send with MarkdownV2, fallback to plain text if parsing fails
            let escaped = escape_markdown_v2(&processed_response);
            
            // Build send message request with thread_id and reply support
            let mut send_req = bot.send_message(chat_id, &escaped)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_parameters(ReplyParameters::new(msg.id));
            if let Some(tid) = thread_id {
                send_req = send_req.message_thread_id(tid);
            }
            
            let send_result = send_req.await;
            
            let sent_msg = match send_result {
                Ok(msg) => Some(msg),
                Err(_) => {
                    // Markdown parsing failed, try plain text
                    tracing::debug!(target: "messages", "Markdown failed, using plain text");
                    let mut plain_req = bot.send_message(chat_id, &processed_response)
                        .reply_parameters(ReplyParameters::new(msg.id));
                    if let Some(tid) = thread_id {
                        plain_req = plain_req.message_thread_id(tid);
                    }
                    plain_req.await.ok()
                }
            };
            
            if let Some(sent_msg) = sent_msg {
                save_and_embed_message(&state, &sent_msg).await;
                add_message_to_history(state.dialogues.clone(), &sent_msg).await;
            }
        }
        Err(e) => {
            let response_time = start_time.elapsed().as_millis();
            logging::log_error("LLM generation", &format!("Failed after {}ms: {}", response_time, e));
            let mut err_req = bot.send_message(chat_id, "Не удалось сгенерировать ответ.")
                .reply_parameters(ReplyParameters::new(msg.id));
            if let Some(tid) = thread_id {
                err_req = err_req.message_thread_id(tid);
            }
            err_req.await?;
        }
    }

    Ok(())
}

async fn check_cooldown(state: &AppState, chat_id: ChatId) -> bool {
    let mut rate_limiter = state.rate_limiter.lock().await;
    if let Some(last_request) = rate_limiter.get(&chat_id) {
        let elapsed = last_request.elapsed().as_secs();
        let chat_settings = match db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await {
            Ok(settings) => settings,
            Err(_) => {
                // Default to 5 seconds if we can't get settings
                db::ChatSettings {
                    chat_id: chat_id.0,
                    auto_reply_enabled: true,
                    reply_mode: "mention_only".to_string(),
                    cooldown_seconds: 5,
                    context_depth: 10,
                    rag_enabled: true,
                }
            }
        };

        if elapsed < chat_settings.cooldown_seconds as u64 {
            return true; // Still in cooldown
        }
    }

    // Update the last request time
    rate_limiter.insert(chat_id, std::time::Instant::now());
    false // Not in cooldown
}

async fn retrieve_memories(state: &AppState, chat_id: ChatId, text: &str) -> Vec<String> {
    match state.llm_client.generate_embeddings(&state.config.ollama_embedding_model, text).await {
        Ok(embedding) => {
            match db::find_similar_chunks(&state.db_pool, chat_id.0, &embedding, MAX_RAG_CHUNKS).await {
                Ok(chunks) => chunks,
                Err(e) => {
                    tracing::warn!(target: "rag", "Failed to retrieve chunks: {}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            tracing::warn!(target: "rag", "Failed to generate embeddings: {}", e);
            vec![]
        }
    }
}

async fn save_and_embed_message(state: &AppState, msg: &Message) {
    if let Some(text) = msg.text() {
        let state = state.clone();
        let msg = msg.clone();
        let text = text.to_string();
        tokio::spawn(async move {
            if let Ok(db_id) = db::save_message(&state.db_pool, &msg).await {
                if let Ok(embedding) = state.llm_client.generate_embeddings(&state.config.ollama_embedding_model, &text).await {
                    if let Err(e) = db::save_embedding(&state.db_pool, db_id, &text, &embedding).await {
                        tracing::warn!(target: "db", "Failed to save embedding: {}", e);
                    }
                }
            }
        });
    }
}

/// Extract 3 frames from video/GIF (start, middle, end) using ffmpeg
async fn extract_frames_from_video(video_data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    use tokio::process::Command;
    
    // Create temp file for input video
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join(format!("pf_input_{}.mp4", std::process::id()));
    // Write video data to temp file
    tokio::fs::write(&input_path, video_data).await
        .map_err(|e| format!("Failed to write temp video: {}", e))?;
    
    // Get video duration using ffprobe
    let duration_output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            input_path.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("ffprobe failed: {}", e))?;
    
    let duration_str = String::from_utf8_lossy(&duration_output.stdout);
    let duration: f64 = duration_str.trim().parse().unwrap_or(1.0);
    
    // Calculate timestamps for 3 frames: start (0.1s), middle, end (duration - 0.1s)
    let timestamps = vec![
        0.1_f64.min(duration * 0.1),
        duration / 2.0,
        (duration - 0.1).max(duration * 0.9),
    ];
    
    let mut frames = Vec::new();
    
    for (i, ts) in timestamps.iter().enumerate() {
        let frame_path = temp_dir.join(format!("pf_frame_{}_{}.jpg", std::process::id(), i));
        
        // Extract single frame at timestamp
        let result = Command::new("ffmpeg")
            .args([
                "-y",
                "-ss", &format!("{:.3}", ts),
                "-i", input_path.to_str().unwrap(),
                "-vframes", "1",
                "-q:v", "2",
                "-vf", "scale='min(800,iw)':'-1'", // Resize to max 800px width
                frame_path.to_str().unwrap(),
            ])
            .output()
            .await;
        
        if let Ok(output) = result {
            if output.status.success() {
                if let Ok(frame_data) = tokio::fs::read(&frame_path).await {
                    frames.push(frame_data);
                }
            }
        }
        
        // Cleanup frame file
        let _ = tokio::fs::remove_file(&frame_path).await;
    }
    
    // Cleanup input file
    let _ = tokio::fs::remove_file(&input_path).await;
    
    if frames.is_empty() {
        return Err("Failed to extract any frames".to_string());
    }
    
    Ok(frames)
}

/// Download file from Telegram and return bytes
async fn download_telegram_file(bot: &Bot, file_id: &str) -> Result<Vec<u8>, String> {
    use teloxide::net::Download;
    use teloxide::types::FileId;
    
    let file = bot.get_file(FileId(file_id.to_string())).await
        .map_err(|e| format!("Failed to get file info: {}", e))?;
    
    let mut buffer = Vec::new();
    bot.download_file(&file.path, &mut buffer).await
        .map_err(|e| format!("Failed to download file: {}", e))?;
    
    Ok(buffer)
}

/// Process voice message - transcribe with Whisper
pub async fn process_voice_message(
    bot: &Bot,
    state: &AppState,
    file_id: &str,
) -> Result<String, String> {
    if !state.config.voice_enabled {
        return Err("Voice is disabled".to_string());
    }
    
    tracing::debug!(target: "voice", "Processing voice file: {}", &file_id[..8.min(file_id.len())]);
    
    // Download the voice file
    let audio_data = download_telegram_file(bot, file_id).await?;
    tracing::debug!(target: "voice", "Downloaded {} bytes", audio_data.len());
    
    // Transcribe with Whisper
    let start = std::time::Instant::now();
    let transcript = state.voice_client.transcribe(audio_data, "voice.ogg").await
        .map_err(|e| format!("Transcription failed: {}", e))?;
    
    if transcript.trim().is_empty() {
        return Err("Empty transcription".to_string());
    }
    
    logging::log_voice_transcription(start.elapsed().as_millis() as u64, &transcript);
    Ok(transcript)
}

/// Process animation (GIF) and generate description
pub async fn process_animation(
    bot: &Bot,
    state: &AppState,
    file_id: &str,
    caption: Option<&str>,
) -> Result<String, String> {
    if !state.config.vision_enabled {
        return Err("Vision is disabled".to_string());
    }
    
    tracing::debug!(target: "vision", "Processing GIF: {}", &file_id[..8.min(file_id.len())]);
    
    // Download the file
    let video_data = download_telegram_file(bot, file_id).await?;
    tracing::debug!(target: "vision", "Downloaded {} bytes", video_data.len());
    
    // Extract frames
    let frames = extract_frames_from_video(&video_data).await?;
    tracing::debug!(target: "vision", "Extracted {} frames", frames.len());
    
    // Convert frames to base64
    let images_base64: Vec<String> = frames.iter()
        .map(|f| BASE64.encode(f))
        .collect();
    
    // Build prompt for vision model
    let prompt = if let Some(cap) = caption {
        format!(
            "Это GIF-анимация из Telegram. Показаны 3 кадра: начало, середина и конец.\n\
            Подпись от пользователя: \"{}\"\n\n\
            Опиши что происходит на этой анимации, учитывая подпись. Будь кратким.",
            cap
        )
    } else {
        "Это GIF-анимация из Telegram. Показаны 3 кадра: начало, середина и конец.\n\n\
        Опиши что происходит на этой анимации. Будь кратким.".to_string()
    };
    
    // Call vision model
    let description = state.llm_client.generate_vision(
        &state.config.ollama_vision_model,
        &prompt,
        images_base64,
        state.config.temperature,
        state.config.max_tokens,
    ).await
    .map_err(|e| format!("Vision model error: {}", e))?;
    
    Ok(description)
}

/// Process photo and generate description
pub async fn process_photo(
    bot: &Bot,
    state: &AppState,
    file_id: &str,
    caption: Option<&str>,
) -> Result<String, String> {
    if !state.config.vision_enabled {
        return Err("Vision is disabled".to_string());
    }
    
    tracing::debug!(target: "vision", "Processing photo: {}", &file_id[..8.min(file_id.len())]);
    
    // Download the photo
    let photo_data = download_telegram_file(bot, file_id).await?;
    tracing::debug!(target: "vision", "Downloaded {} bytes", photo_data.len());
    
    // Convert to base64
    let image_base64 = BASE64.encode(&photo_data);
    
    // Build prompt for vision model
    let prompt = if let Some(cap) = caption {
        format!(
            "Пользователь отправил фото с подписью: \"{}\"\n\n\
            Опиши что на фото, учитывая подпись. Будь кратким.",
            cap
        )
    } else {
        "Пользователь отправил фото. Опиши что на нём. Будь кратким.".to_string()
    };
    
    // Call vision model
    let description = state.llm_client.generate_vision(
        &state.config.ollama_vision_model,
        &prompt,
        vec![image_base64],
        state.config.temperature,
        state.config.max_tokens,
    ).await
    .map_err(|e| format!("Vision model error: {}", e))?;
    
    Ok(format!("[Фото]: {}", description))
}

/// Extract audio from video file using ffmpeg
async fn extract_audio_from_video(video_data: &[u8]) -> Result<Vec<u8>, String> {
    use tokio::process::Command;
    
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join(format!("pf_video_{}.mp4", std::process::id()));
    let output_path = temp_dir.join(format!("pf_audio_{}.ogg", std::process::id()));
    
    // Write video data to temp file
    tokio::fs::write(&input_path, video_data).await
        .map_err(|e| format!("Failed to write temp video: {}", e))?;
    
    // Extract audio using ffmpeg
    let result = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input_path.to_str().unwrap(),
            "-vn",  // No video
            "-acodec", "libopus",
            "-b:a", "64k",
            output_path.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("ffmpeg failed: {}", e))?;
    
    // Cleanup input
    let _ = tokio::fs::remove_file(&input_path).await;
    
    if !result.status.success() {
        let _ = tokio::fs::remove_file(&output_path).await;
        return Err("Failed to extract audio".to_string());
    }
    
    // Read audio file
    let audio_data = tokio::fs::read(&output_path).await
        .map_err(|e| format!("Failed to read audio: {}", e))?;
    
    // Cleanup output
    let _ = tokio::fs::remove_file(&output_path).await;
    
    Ok(audio_data)
}

/// Process video_note (circle video) - extract frames + transcribe audio
pub async fn process_video_note(
    bot: &Bot,
    state: &AppState,
    file_id: &str,
) -> Result<String, String> {
    tracing::debug!(target: "media", "Processing video_note: {}", &file_id[..8.min(file_id.len())]);
    
    // Download the file
    let video_data = download_telegram_file(bot, file_id).await?;
    tracing::debug!(target: "media", "Downloaded {} bytes", video_data.len());
    
    let mut result_parts: Vec<String> = Vec::new();
    
    // Try to transcribe audio if voice is enabled
    if state.config.voice_enabled {
        match extract_audio_from_video(&video_data).await {
            Ok(audio_data) => {
                tracing::debug!(target: "voice", "Extracted audio ({} bytes)", audio_data.len());
                let start = std::time::Instant::now();
                match state.voice_client.transcribe(audio_data, "video_note.ogg").await {
                    Ok(transcript) if !transcript.trim().is_empty() => {
                        logging::log_voice_transcription(start.elapsed().as_millis() as u64, &transcript);
                        result_parts.push(format!("🎤 Сказано: {}", transcript.trim()));
                    }
                    Ok(_) => {
                        tracing::debug!(target: "voice", "Empty transcription");
                    }
                    Err(e) => {
                        tracing::debug!(target: "voice", "Transcription failed: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::debug!(target: "media", "Audio extraction failed: {}", e);
            }
        }
    }
    
    // Try to analyze video frames if vision is enabled
    if state.config.vision_enabled {
        match extract_frames_from_video(&video_data).await {
            Ok(frames) => {
                tracing::debug!(target: "vision", "Extracted {} frames", frames.len());
                
                let images_base64: Vec<String> = frames.iter()
                    .map(|f| BASE64.encode(f))
                    .collect();
                
                let prompt = "Это видеосообщение (кружок) из Telegram. Показаны 3 кадра: начало, середина и конец.\n\n\
                    Кратко опиши что видно на видео.";
                
                match state.llm_client.generate_vision(
                    &state.config.ollama_vision_model,
                    prompt,
                    images_base64,
                    state.config.temperature,
                    state.config.max_tokens,
                ).await {
                    Ok(description) => {
                        result_parts.push(format!("👁 Видно: {}", description.trim()));
                    }
                    Err(e) => {
                        tracing::debug!(target: "vision", "Vision analysis failed: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::debug!(target: "media", "Frame extraction failed: {}", e);
            }
        }
    }
    
    if result_parts.is_empty() {
        return Err("Could not process video_note (vision and voice disabled or failed)".to_string());
    }
    
    Ok(result_parts.join("\n\n"))
}

async fn get_and_update_history_with_depth(dialogues: DialogueState, new_msg: &Message, depth: usize) -> Vec<Message> {
    let mut dialogues = dialogues.lock().await;
    let history = dialogues.entry(new_msg.chat.id).or_default();
    history.push(new_msg.clone());
    if history.len() > depth {
        history.remove(0);
    }
    history.clone()
}

async fn add_message_to_history(dialogues: DialogueState, new_msg: &Message) {
    let mut dialogues = dialogues.lock().await;
    let history = dialogues.entry(new_msg.chat.id).or_default();
    history.push(new_msg.clone());
    if history.len() > MAX_CONTEXT_MESSAGES {
        history.remove(0);
    }
}

fn build_prompt(
    persona_prompt: String,
    long_term_memories: Vec<String>,
    short_term_history: Vec<Message>,
    bot_name: &str,
) -> String {
    // Build system prompt with bot name integration
    // The bot name from config is the "real" name that the persona should use
    let mut prompt = format!(
        "System: Тебя зовут {name}. Это твоё имя — используй его когда представляешься или когда спрашивают как тебя зовут. \
        Ты откликаешься на имя \"{name}\" и его вариации. \
        Когда к тебе обращаются по имени, отвечай как будто это твоё настоящее имя.\n\n\
        {prompt}\n\n",
        name = bot_name,
        prompt = persona_prompt
    );

    if !long_term_memories.is_empty() {
        prompt.push_str("### Relevant Past Memories (for context):\n");
        for memory in long_term_memories {
            prompt.push_str(&format!("- {}\n", memory.trim()));
        }
        prompt.push_str("\n### Current Conversation:\n");
    }

    for msg in short_term_history {
        let sender_name = msg.from.as_ref().map(|u| {
            if u.is_bot {
                bot_name.to_string()
            } else {
                u.first_name.clone()
            }
        }).unwrap_or_else(|| bot_name.to_string());
        let text = msg.text().unwrap_or("");
        prompt.push_str(&format!("{}: {}\n", sender_name, text));
    }
    prompt.push_str(&format!("{}: ", bot_name));
    prompt
}

fn apply_human_behavior_rules(response: String, bot_name: &str) -> String {
    let mut processed_response = response;

    // Rule 1: Avoid revealing that it's a bot
    processed_response = processed_response.replace("I am a bot", "I am an AI assistant");
    processed_response = processed_response.replace("I'm a bot", "I'm an AI assistant");
    processed_response = processed_response.replace("as a bot", "as an AI assistant");
    processed_response = processed_response.replace("as an AI assistant", "as an AI"); // More natural
    processed_response = processed_response.replace(&format!("I am {}", bot_name), "Hi, I'm an AI");
    processed_response = processed_response.replace(&format!("I'm {}", bot_name), "Hi, I'm an AI");

    // Rule 2: Reduce repetitive phrases
    processed_response = processed_response.replace("Sure, ", "");
    processed_response = processed_response.replace("Okay, ", "");
    processed_response = processed_response.replace("Certainly, ", "");
    processed_response = processed_response.replace("Absolutely, ", "");

    // Rule 3: Handle personal data carefully
    processed_response = processed_response.replace("personal information", "private details");
    processed_response = processed_response.replace("personal data", "private details");

    // Rule 4: Add more natural phrasing
    if processed_response.starts_with("Yes, ") {
        processed_response = processed_response.replacen("Yes, ", "", 1);
    }
    if processed_response.starts_with("No, ") {
        processed_response = processed_response.replacen("No, ", "", 1);
    }

    // Rule 5: Avoid mentioning the bot's name unnecessarily
    processed_response = processed_response.replace(&format!(" {}", bot_name), " I");

    processed_response
}


/// Escape special characters for Telegram MarkdownV2
/// Preserves intentional formatting like *bold*, _italic_, `code`, ```code blocks```
fn escape_markdown_v2(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    
    while i < len {
        let c = chars[i];
        
        // Check for code blocks ```
        if c == '`' && i + 2 < len && chars[i + 1] == '`' && chars[i + 2] == '`' {
            // Find closing ```
            if let Some(end) = find_closing_code_block(&chars, i + 3) {
                // Copy code block as-is (code blocks don't need escaping inside)
                for j in i..=end {
                    result.push(chars[j]);
                }
                i = end + 1;
                continue;
            }
        }
        
        // Check for inline code `
        if c == '`' {
            if let Some(end) = find_closing_char(&chars, i + 1, '`') {
                // Copy inline code as-is
                for j in i..=end {
                    result.push(chars[j]);
                }
                i = end + 1;
                continue;
            }
        }
        
        // Check for bold **text** or *text*
        if c == '*' {
            let double = i + 1 < len && chars[i + 1] == '*';
            let closing_char = if double { "**" } else { "*" };
            let start = if double { i + 2 } else { i + 1 };
            
            if let Some(end) = find_closing_pattern(&chars, start, closing_char) {
                // Keep formatting markers, escape content
                result.push('*');
                if double {
                    result.push('*');
                }
                for j in start..end {
                    if should_escape_in_formatted(chars[j]) {
                        result.push('\\');
                    }
                    result.push(chars[j]);
                }
                result.push('*');
                if double {
                    result.push('*');
                    i = end + 2;
                } else {
                    i = end + 1;
                }
                continue;
            }
        }
        
        // Check for italic _text_
        if c == '_' {
            if let Some(end) = find_closing_char(&chars, i + 1, '_') {
                result.push('_');
                for j in (i + 1)..end {
                    if should_escape_in_formatted(chars[j]) {
                        result.push('\\');
                    }
                    result.push(chars[j]);
                }
                result.push('_');
                i = end + 1;
                continue;
            }
        }
        
        // Escape special characters outside of formatting
        if should_escape_outside(c) {
            result.push('\\');
        }
        result.push(c);
        i += 1;
    }
    
    result
}

fn find_closing_code_block(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i + 2 < len {
        if chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
            return Some(i + 2);
        }
        i += 1;
    }
    None
}

fn find_closing_char(chars: &[char], start: usize, closing: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == closing {
            return Some(i);
        }
        if chars[i] == '\n' {
            return None; // Don't cross newlines for inline formatting
        }
    }
    None
}

fn find_closing_pattern(chars: &[char], start: usize, pattern: &str) -> Option<usize> {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let plen = pattern_chars.len();
    
    for i in start..(chars.len() - plen + 1) {
        let mut matches = true;
        for j in 0..plen {
            if chars[i + j] != pattern_chars[j] {
                matches = false;
                break;
            }
        }
        if matches {
            return Some(i);
        }
    }
    None
}

fn should_escape_in_formatted(c: char) -> bool {
    // Inside formatted text, escape these
    matches!(c, '\\' | '`' | '[' | ']' | '(' | ')' | '~' | '>' | '#' | '+' | '-' | '=' | '|' | '{' | '}' | '.' | '!')
}

fn should_escape_outside(c: char) -> bool {
    // Outside formatted text, escape all special chars
    matches!(c, '\\' | '[' | ']' | '(' | ')' | '~' | '>' | '#' | '+' | '-' | '=' | '|' | '{' | '}' | '.' | '!')
}


// === WIZARD HANDLERS ===

async fn handle_wizard_input(bot: Bot, msg: Message, state: AppState, wizard_state: WizardState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default().trim();

    match wizard_state {
        WizardState::CreatingPersonaName => {
            if text.is_empty() {
                bot.send_message(chat_id, "❌ Название не может быть пустым. Попробуйте ещё раз:").await?;
                return Ok(());
            }
            // Move to display name step
            state.set_wizard_state(chat_id, WizardState::CreatingPersonaDisplayName { name: text.to_string() }).await;
            
            let bot_name = state.get_bot_name().await;
            bot.send_message(chat_id, format!(
                "✅ Название: <b>{}</b>\n\n\
                Теперь укажите <b>имя</b>, на которое персона будет откликаться.\n\n\
                💡 Текущее имя бота: <code>{}</code>\n\
                Отправьте <code>-</code> чтобы использовать имя бота по умолчанию.\n\n\
                /cancel для отмены",
                text, bot_name
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
        
        WizardState::CreatingPersonaDisplayName { name } => {
            let display_name = if text == "-" || text.is_empty() {
                None // Use default bot name
            } else {
                Some(text.to_string())
            };
            
            // Move to triggers step
            state.set_wizard_state(chat_id, WizardState::CreatingPersonaTriggers { name, display_name: display_name.clone() }).await;
            
            let display_info = display_name.as_ref()
                .map(|n| format!("<code>{}</code>", n))
                .unwrap_or_else(|| "имя бота по умолчанию".to_string());
            
            bot.send_message(chat_id, format!(
                "✅ Имя персоны: {}\n\n\
                Теперь укажите <b>триггеры</b> — ключевые слова через запятую, на которые персона будет реагировать.\n\n\
                💡 Пример: <code>помоги, подскажи, эй</code>\n\
                Отправьте <code>-</code> чтобы пропустить (только имя и @упоминание).\n\n\
                /cancel для отмены",
                display_info
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
        
        WizardState::CreatingPersonaTriggers { name, display_name } => {
            let triggers = if text == "-" || text.is_empty() {
                None
            } else {
                let keywords: Vec<String> = text
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                if keywords.is_empty() { None } else { Some(keywords.join(",")) }
            };
            
            // Move to prompt step
            state.set_wizard_state(chat_id, WizardState::CreatingPersonaPrompt { name, display_name, triggers: triggers.clone() }).await;
            
            let triggers_info = triggers.as_ref()
                .map(|t| format!("<code>{}</code>", t))
                .unwrap_or_else(|| "не заданы".to_string());
            
            bot.send_message(chat_id, format!(
                "✅ Триггеры: {}\n\n\
                Теперь введите <b>системный промпт</b> для персоны.\n\n\
                💡 Опишите характер, стиль общения, правила поведения.\n\n\
                /cancel для отмены",
                triggers_info
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
        
        WizardState::CreatingPersonaPrompt { name, display_name, triggers } => {
            if text.is_empty() {
                bot.send_message(chat_id, "❌ Промпт не может быть пустым. Попробуйте ещё раз:").await?;
                return Ok(());
            }
            
            // Create persona with all fields
            match db::create_persona_full(
                &state.db_pool, 
                &name, 
                text,
                display_name.as_deref(),
                triggers.as_deref(),
            ).await {
                Ok(id) => {
                    state.clear_wizard_state(chat_id).await;
                    
                    let display_info = display_name.as_ref()
                        .map(|n| format!("Имя: {}", n))
                        .unwrap_or_else(|| "Имя: по умолчанию".to_string());
                    let triggers_info = triggers.as_ref()
                        .map(|t| format!("Триггеры: {}", t))
                        .unwrap_or_else(|| "Триггеры: не заданы".to_string());
                    
                    bot.send_message(chat_id, format!(
                        "✅ Персона <b>{}</b> создана!\n\n\
                        📋 ID: {}\n\
                        👤 {}\n\
                        🎯 {}\n\n\
                        Используйте /activate_persona {} или меню для активации.",
                        name, id, display_info, triggers_info, id
                    ))
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
                Err(e) => {
                    tracing::warn!(target: "db", "Failed to create persona: {}", e);
                    state.clear_wizard_state(chat_id).await;
                    bot.send_message(chat_id, "❌ Ошибка при создании персоны.").await?;
                }
            }
        }
        
        WizardState::UpdatingPersonaName { id } => {
            if text.is_empty() {
                bot.send_message(chat_id, "❌ Название не может быть пустым. Попробуйте ещё раз:").await?;
                return Ok(());
            }
            
            // Get current persona to show current values
            let current = db::get_persona_by_id(&state.db_pool, id).await.ok().flatten();
            let current_display = current.as_ref()
                .and_then(|p| p.display_name.as_ref())
                .map(|n| format!("<code>{}</code>", n))
                .unwrap_or_else(|| "по умолчанию".to_string());
            
            state.set_wizard_state(chat_id, WizardState::UpdatingPersonaDisplayName { id, name: text.to_string() }).await;
            
            let bot_name = state.get_bot_name().await;
            bot.send_message(chat_id, format!(
                "✅ Новое название: <b>{}</b>\n\n\
                Теперь укажите <b>имя</b>, на которое персона будет откликаться.\n\n\
                💡 Текущее: {}\n\
                💡 Имя бота: <code>{}</code>\n\
                Отправьте <code>-</code> чтобы использовать имя бота по умолчанию.\n\n\
                /cancel для отмены",
                text, current_display, bot_name
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
        
        WizardState::UpdatingPersonaDisplayName { id, name } => {
            let display_name = if text == "-" || text.is_empty() {
                None
            } else {
                Some(text.to_string())
            };
            
            // Get current triggers to show
            let current = db::get_persona_by_id(&state.db_pool, id).await.ok().flatten();
            let current_triggers = current.as_ref()
                .and_then(|p| p.triggers.as_ref())
                .map(|t| format!("<code>{}</code>", t))
                .unwrap_or_else(|| "не заданы".to_string());
            
            state.set_wizard_state(chat_id, WizardState::UpdatingPersonaTriggers { id, name, display_name: display_name.clone() }).await;
            
            let display_info = display_name.as_ref()
                .map(|n| format!("<code>{}</code>", n))
                .unwrap_or_else(|| "имя бота по умолчанию".to_string());
            
            bot.send_message(chat_id, format!(
                "✅ Имя персоны: {}\n\n\
                Теперь укажите <b>триггеры</b> — ключевые слова через запятую.\n\n\
                💡 Текущие: {}\n\
                Отправьте <code>-</code> чтобы убрать триггеры.\n\n\
                /cancel для отмены",
                display_info, current_triggers
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
        
        WizardState::UpdatingPersonaTriggers { id, name, display_name } => {
            let triggers = if text == "-" || text.is_empty() {
                None
            } else {
                let keywords: Vec<String> = text
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                if keywords.is_empty() { None } else { Some(keywords.join(",")) }
            };
            
            state.set_wizard_state(chat_id, WizardState::UpdatingPersonaPrompt { id, name, display_name, triggers: triggers.clone() }).await;
            
            let triggers_info = triggers.as_ref()
                .map(|t| format!("<code>{}</code>", t))
                .unwrap_or_else(|| "не заданы".to_string());
            
            bot.send_message(chat_id, format!(
                "✅ Триггеры: {}\n\n\
                Теперь введите новый <b>системный промпт</b>.\n\n\
                /cancel для отмены",
                triggers_info
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
        
        WizardState::UpdatingPersonaPrompt { id, name, display_name, triggers } => {
            if text.is_empty() {
                bot.send_message(chat_id, "❌ Промпт не может быть пустым. Попробуйте ещё раз:").await?;
                return Ok(());
            }
            
            match db::update_persona_full(&state.db_pool, id, &name, text, display_name.as_deref(), triggers.as_deref()).await {
                Ok(()) => {
                    state.clear_wizard_state(chat_id).await;
                    
                    let display_info = display_name.as_ref()
                        .map(|n| format!("Имя: {}", n))
                        .unwrap_or_else(|| "Имя: по умолчанию".to_string());
                    let triggers_info = triggers.as_ref()
                        .map(|t| format!("Триггеры: {}", t))
                        .unwrap_or_else(|| "Триггеры: не заданы".to_string());
                    
                    bot.send_message(chat_id, format!(
                        "✅ Персона <b>{}</b> обновлена!\n\n\
                        📋 ID: {}\n\
                        👤 {}\n\
                        🎯 {}",
                        name, id, display_info, triggers_info
                    ))
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
                Err(e) => {
                    tracing::warn!(target: "db", "Failed to update persona: {}", e);
                    state.clear_wizard_state(chat_id).await;
                    bot.send_message(chat_id, "❌ Ошибка при обновлении персоны.").await?;
                }
            }
        }
        
        WizardState::SettingKeywords => {
            let keywords: Vec<String> = text
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
            
            if keywords.is_empty() {
                bot.send_message(chat_id, "❌ Введите хотя бы одно ключевое слово через запятую:").await?;
                return Ok(());
            }
            
            state.keyword_triggers.lock().await.insert(chat_id, keywords.clone());
            state.clear_wizard_state(chat_id).await;
            bot.send_message(chat_id, format!("✅ Триггеры установлены: {}", keywords.join(", "))).await?;
        }
        
        WizardState::ImportingPersona => {
            // Try to import from text or check for document
            if let Some(doc) = msg.document() {
                // Handle document import
                let file = bot.get_file(doc.file.id.clone()).await?;
                let mut buffer = Vec::new();
                use teloxide::net::Download;
                bot.download_file(&file.path, &mut buffer).await?;
                let json = String::from_utf8_lossy(&buffer);
                
                match db::import_personas(&state.db_pool, &json).await {
                    Ok(ids) if !ids.is_empty() => {
                        state.clear_wizard_state(chat_id).await;
                        bot.send_message(chat_id, format!("✅ Импортировано {} персон: {:?}", ids.len(), ids)).await?;
                    }
                    Ok(_) => {
                        match db::import_persona(&state.db_pool, &json).await {
                            Ok(id) => {
                                state.clear_wizard_state(chat_id).await;
                                bot.send_message(chat_id, format!("✅ Персона импортирована с ID: {}", id)).await?;
                            }
                            Err(e) => {
                                bot.send_message(chat_id, format!("❌ Ошибка импорта: {}", e)).await?;
                            }
                        }
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("❌ Ошибка импорта: {}", e)).await?;
                    }
                }
            } else if !text.is_empty() {
                // Try to parse as JSON
                match db::import_persona(&state.db_pool, text).await {
                    Ok(id) => {
                        state.clear_wizard_state(chat_id).await;
                        bot.send_message(chat_id, format!("✅ Персона импортирована с ID: {}", id)).await?;
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("❌ Ошибка импорта: {}\n\nПопробуйте ещё раз или /cancel", e)).await?;
                    }
                }
            } else {
                bot.send_message(chat_id, "❌ Отправьте JSON-файл или текст в формате JSON.").await?;
            }
        }
        
        WizardState::Broadcasting => {
            if text.is_empty() {
                bot.send_message(chat_id, "❌ Сообщение не может быть пустым. Попробуйте ещё раз:").await?;
                return Ok(());
            }
            
            let chats = db::get_all_chat_ids(&state.db_pool).await.unwrap_or_default();
            if chats.is_empty() {
                state.clear_wizard_state(chat_id).await;
                bot.send_message(chat_id, "❌ Нет чатов для рассылки.").await?;
                return Ok(());
            }
            
            let (mut ok, mut err) = (0, 0);
            for target in &chats {
                match bot.send_message(ChatId(*target), text).await {
                    Ok(_) => ok += 1,
                    Err(_) => err += 1,
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            
            state.clear_wizard_state(chat_id).await;
            bot.send_message(chat_id, format!("📢 Рассылка завершена: ✅{} ❌{}", ok, err)).await?;
        }
        
        // Handle other wizard states that don't need text input
        _ => {
            state.clear_wizard_state(chat_id).await;
        }
    }
    
    Ok(())
}
