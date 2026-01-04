use crate::db;
use crate::state::{AppState, DialogueState};
use teloxide::prelude::*;

const MAX_CONTEXT_MESSAGES: usize = 10;
const MAX_RAG_CHUNKS: u32 = 3;
const DEFAULT_LLM_MODEL: &str = "llama3:latest";
const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text";
const DEFAULT_PERSONA_PROMPT: &str = "You are a helpful AI assistant.";

pub async fn handle_message(bot: Bot, msg: Message, state: AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    if text.starts_with('/') {
        return Ok(())
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

    // --- RAG & Context ---
    let long_term_memories = retrieve_memories(&state, chat_id, text).await;
    let short_term_history = get_and_update_history(state.dialogues.clone(), &msg).await;
    let prompt = build_prompt(persona_prompt, long_term_memories, short_term_history);
    
    log::debug!("Prompt for chat {}: {}", chat_id, prompt);

    // --- Generate Response ---
    match state.llm_client.generate(DEFAULT_LLM_MODEL, &prompt).await {
        Ok(response_text) => {
            log::info!("LLM response for chat {}: {}", chat_id, response_text);
            if let Ok(sent_msg) = bot.send_message(chat_id, &response_text).await {
                save_and_embed_message(&state, &sent_msg).await;
                add_message_to_history(state.dialogues.clone(), &sent_msg).await;
            }
        }
        Err(e) => {
            log::error!("Failed to get response from LLM: {}", e);
            bot.send_message(chat_id, "Не удалось сгенерировать ответ.").await?;
        }
    }

    Ok(())
}

async fn retrieve_memories(state: &AppState, chat_id: ChatId, text: &str) -> Vec<String> {
    match state.llm_client.generate_embeddings(DEFAULT_EMBEDDING_MODEL, text).await {
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
                if let Ok(embedding) = state.llm_client.generate_embeddings(DEFAULT_EMBEDDING_MODEL, &text).await {
                    if let Err(e) = db::save_embedding(&state.db_pool, db_id, &text, &embedding).await {
                        log::error!("Failed to save embedding: {}", e);
                    }
                }
            }
        });
    }
}

async fn get_and_update_history(dialogues: DialogueState, new_msg: &Message) -> Vec<Message> {
    let mut dialogues = dialogues.lock().await;
    let history = dialogues.entry(new_msg.chat.id).or_default();
    history.push(new_msg.clone());
    if history.len() > MAX_CONTEXT_MESSAGES {
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
        let sender_name = msg.from().map(|u| u.first_name.clone()).unwrap_or_else(|| "Bot".to_string());
        let text = msg.text().unwrap_or("");
        prompt.push_str(&format!("{}: {}\n", sender_name, text));
    }
    prompt.push_str("Bot: ");
    prompt
}
