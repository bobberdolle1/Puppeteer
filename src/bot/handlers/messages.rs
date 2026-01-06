use crate::db;
use crate::state::{AppState, DialogueState};
use teloxide::prelude::*;
use teloxide::types::ParseMode;

const MAX_CONTEXT_MESSAGES: usize = 20; // Maximum context size to prevent memory issues, but can be overridden by chat settings
const MAX_RAG_CHUNKS: u32 = 3;
const DEFAULT_PERSONA_PROMPT: &str = "You are a helpful AI assistant.";

pub async fn handle_message(bot: Bot, msg: Message, state: AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    if text.starts_with('/') {
        // Handle commands
        return crate::bot::handlers::commands::handle_command(bot, msg, state).await;
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
    let persona_prompt = db::get_active_persona(&state.db_pool)
        .await
        .map(|p_opt| p_opt.map_or_else(|| DEFAULT_PERSONA_PROMPT.to_string(), |p| p.prompt))
        .unwrap_or_else(|e| {
            log::error!("Failed to get active persona: {}", e);
            DEFAULT_PERSONA_PROMPT.to_string()
        });

    // --- Get Chat Settings ---
    let chat_settings = db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await
        .unwrap_or_else(|e| {
            log::error!("Failed to get chat settings: {}", e);
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

    // Check if auto-reply is enabled
    if !chat_settings.auto_reply_enabled {
        return Ok(());
    }

    // Check reply mode (mention/command vs all messages)
    // In private chats, always reply
    // If someone replies to bot's message, always reply
    // If someone mentions bot by name, always reply
    let is_private = msg.chat.is_private();
    let is_reply_to_bot = msg.reply_to_message().map(|reply| {
        reply.from.as_ref().map(|u| u.is_bot).unwrap_or(false)
    }).unwrap_or(false);
    
    // Check if bot is mentioned by name or username
    let bot_name = state.get_bot_name().await;
    let bot_username = state.get_bot_username().await;
    let text_lower = text.to_lowercase();
    let bot_name_lower = bot_name.to_lowercase();
    
    let is_mentioned_by_name = text_lower.contains(&bot_name_lower) ||
        bot_username.as_ref().map(|u| text.contains(&format!("@{}", u))).unwrap_or(false);
    
    let should_reply = if is_private || is_reply_to_bot || is_mentioned_by_name || chat_settings.reply_mode == "all_messages" {
        true
    } else {
        // For "mention_only" mode, check if bot is mentioned
        let bot_info = bot.get_me().await;
        if let Ok(me) = bot_info {
            let username = me.user.username.as_deref().unwrap_or("");
            text.contains(&format!("@{}", username))
                || text.contains(&format!("/{}", username))
        } else {
            false // If we can't get bot info, default to not replying in mention-only mode
        }
    };

    if !should_reply {
        // Still save the message for context, but don't reply
        save_and_embed_message(&state, &msg).await;
        return Ok(());
    }

    // Check cooldown
    if check_cooldown(&state, chat_id).await {
        return Ok(());
    }

    // --- RAG & Context ---
    let long_term_memories = if chat_settings.rag_enabled {
        retrieve_memories(&state, chat_id, text).await
    } else {
        vec![] // Empty vector if RAG is disabled
    };

    // Use context depth from chat settings
    let short_term_history = get_and_update_history_with_depth(state.dialogues.clone(), &msg, chat_settings.context_depth as usize).await;
    
    // Get bot name for prompt
    let bot_name = state.get_bot_name().await;
    let prompt = build_prompt(persona_prompt, long_term_memories, short_term_history, &bot_name);

    log::debug!("Prompt for chat {}: {}", chat_id, prompt);

    // Get thread_id for forum topics support
    let thread_id = msg.thread_id;

    // --- Show typing indicator ---
    let mut typing_action = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing);
    if let Some(tid) = thread_id {
        typing_action = typing_action.message_thread_id(tid);
    }
    let _ = typing_action.await;

    // --- Generate Response ---
    let start_time = std::time::Instant::now();
    match state.llm_client.generate(&state.config.ollama_chat_model, &prompt, state.config.temperature, state.config.max_tokens).await {
        Ok(response_text) => {
            let response_time = start_time.elapsed().as_millis();

            // Apply human-like behavior rules
            let processed_response = apply_human_behavior_rules(response_text, &state.config.bot_name);

            log::info!("LLM response for chat {}: {} (took {}ms)", chat_id, processed_response, response_time);
            
            // Try to send with MarkdownV2, fallback to plain text if parsing fails
            let escaped = escape_markdown_v2(&processed_response);
            
            // Build send message request with thread_id support
            let mut send_req = bot.send_message(chat_id, &escaped)
                .parse_mode(ParseMode::MarkdownV2);
            if let Some(tid) = thread_id {
                send_req = send_req.message_thread_id(tid);
            }
            
            let send_result = send_req.await;
            
            let sent_msg = match send_result {
                Ok(msg) => Some(msg),
                Err(_) => {
                    // Markdown parsing failed, try plain text
                    log::debug!("Markdown parsing failed, sending as plain text");
                    let mut plain_req = bot.send_message(chat_id, &processed_response);
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
            log::error!("Failed to get response from LLM after {}ms: {}", response_time, e);
            let mut err_req = bot.send_message(chat_id, "Не удалось сгенерировать ответ.");
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
                    log::error!("Failed to retrieve RAG chunks: {}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            log::error!("Failed to generate embeddings for RAG: {}", e);
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
                        log::error!("Failed to save embedding: {}", e);
                    }
                }
            }
        });
    }
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
    // Add bot name to system prompt
    let mut prompt = format!("System: Your name is {}. {}\n\n", bot_name, persona_prompt);

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
