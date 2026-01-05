use crate::db;
use crate::state::{AppState, DialogueState};
use teloxide::prelude::*;

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
    let should_reply = if chat_settings.reply_mode == "all_messages" {
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
    let prompt = build_prompt(persona_prompt, long_term_memories, short_term_history);

    log::debug!("Prompt for chat {}: {}", chat_id, prompt);

    // --- Show typing indicator ---
    let _ = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing).await;

    // --- Generate Response ---
    let start_time = std::time::Instant::now();
    match state.llm_client.generate(&state.config.ollama_chat_model, &prompt, state.config.temperature, state.config.max_tokens).await {
        Ok(response_text) => {
            let response_time = start_time.elapsed().as_millis();

            // Apply human-like behavior rules
            let processed_response = apply_human_behavior_rules(response_text, &state.config.bot_name);

            log::info!("LLM response for chat {}: {} (took {}ms)", chat_id, processed_response, response_time);
            if let Ok(sent_msg) = bot.send_message(chat_id, &processed_response).await {
                save_and_embed_message(&state, &sent_msg).await;
                add_message_to_history(state.dialogues.clone(), &sent_msg).await;
            }
        }
        Err(e) => {
            let response_time = start_time.elapsed().as_millis();
            log::error!("Failed to get response from LLM after {}ms: {}", response_time, e);
            bot.send_message(chat_id, "Не удалось сгенерировать ответ.").await?;
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
) -> String {
    let mut prompt = format!("System: {}\n\n", persona_prompt);

    if !long_term_memories.is_empty() {
        prompt.push_str("### Relevant Past Memories (for context):\n");
        for memory in long_term_memories {
            prompt.push_str(&format!("- {}\n", memory.trim()));
        }
        prompt.push_str("\n### Current Conversation:\n");
    }

    for msg in short_term_history {
        let sender_name = msg.from.as_ref().map(|u| u.first_name.clone()).unwrap_or_else(|| "Bot".to_string());
        let text = msg.text().unwrap_or("");
        prompt.push_str(&format!("{}: {}\n", sender_name, text));
    }
    prompt.push_str("Bot: ");
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
