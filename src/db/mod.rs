use bincode::{deserialize, serialize};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row, SqlitePool};
use teloxide::types::Message;

// --- Data Structures ---

#[derive(Debug, FromRow)]
pub struct DbMessage {
    pub id: i64,
    pub message_id: i64,
    pub chat_id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub text: Option<String>,
    pub sent_at: chrono::NaiveDateTime,
}

#[derive(Debug, FromRow, Clone)]
pub struct MemoryChunk {
    pub id: i64,
    pub message_id: i64,
    pub chunk_text: String,
    pub embedding: Option<Vec<u8>>,
    pub importance_score: Option<f64>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, FromRow, Clone)]
pub struct ChatSummary {
    pub id: i64,
    pub chat_id: i64,
    pub summary_text: String,
    pub messages_from: i64,
    pub messages_to: i64,
    pub message_count: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, FromRow, Clone)]
pub struct Persona {
    pub id: i64,
    pub name: String,
    pub prompt: String,
    pub is_active: bool,
}

/// Persona export format for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaExport {
    pub name: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub version: String,
}

impl From<Persona> for PersonaExport {
    fn from(p: Persona) -> Self {
        Self {
            name: p.name,
            prompt: p.prompt,
            description: None,
            version: "1.0".to_string(),
        }
    }
}

#[derive(Debug, FromRow, Clone)]
pub struct ChatSettings {
    pub chat_id: i64,
    pub auto_reply_enabled: bool,
    pub reply_mode: String,
    pub cooldown_seconds: i64,
    pub context_depth: i64,
    pub rag_enabled: bool,
}

// --- Public Functions: Personas ---

pub async fn get_all_personas(pool: &SqlitePool) -> Result<Vec<Persona>, sqlx::Error> {
    sqlx::query("SELECT id, name, prompt, is_active FROM personas ORDER BY name")
        .map(|row: SqliteRow| Persona {
            id: row.get("id"),
            name: row.get("name"),
            prompt: row.get("prompt"),
            is_active: row.get("is_active"),
        })
        .fetch_all(pool)
        .await
}

pub async fn get_active_persona(pool: &SqlitePool) -> Result<Option<Persona>, sqlx::Error> {
    sqlx::query("SELECT id, name, prompt, is_active FROM personas WHERE is_active = 1 LIMIT 1")
        .map(|row: SqliteRow| Persona {
            id: row.get("id"),
            name: row.get("name"),
            prompt: row.get("prompt"),
            is_active: row.get("is_active"),
        })
        .fetch_optional(pool)
        .await
}

pub async fn get_persona_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Persona>, sqlx::Error> {
    sqlx::query("SELECT id, name, prompt, is_active FROM personas WHERE id = ?")
        .bind(id)
        .map(|row: SqliteRow| Persona {
            id: row.get("id"),
            name: row.get("name"),
            prompt: row.get("prompt"),
            is_active: row.get("is_active"),
        })
        .fetch_optional(pool)
        .await
}

pub async fn set_active_persona(pool: &SqlitePool, persona_id: i64) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE personas SET is_active = 0")
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE personas SET is_active = 1 WHERE id = ?")
        .bind(persona_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub async fn create_persona(pool: &SqlitePool, name: &str, prompt: &str) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO personas (name, prompt, is_active)
        VALUES (?, ?, 0)
        "#,
    )
    .bind(name)
    .bind(prompt)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update_persona(pool: &SqlitePool, id: i64, name: &str, prompt: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE personas
        SET name = ?, prompt = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(prompt)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_persona(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // If we're deleting the active persona, deactivate it first
    sqlx::query("UPDATE personas SET is_active = 0 WHERE id = ? AND is_active = 1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM personas WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await
}

// --- Public Functions: Chat Settings ---

pub async fn get_or_create_chat_settings(
    pool: &SqlitePool,
    chat_id: i64,
) -> Result<ChatSettings, sqlx::Error> {
    let query = "SELECT chat_id, auto_reply_enabled, reply_mode, cooldown_seconds, context_depth, rag_enabled FROM chat_settings WHERE chat_id = ?";
    let existing: Option<ChatSettings> = sqlx::query(query)
        .bind(chat_id)
        .map(|row: SqliteRow| ChatSettings {
            chat_id: row.get("chat_id"),
            auto_reply_enabled: row.get("auto_reply_enabled"),
            reply_mode: row.get("reply_mode"),
            cooldown_seconds: row.get("cooldown_seconds"),
            context_depth: row.get("context_depth"),
            rag_enabled: row.get("rag_enabled"),
        })
        .fetch_optional(pool)
        .await?;

    if let Some(settings) = existing {
        Ok(settings)
    } else {
        let default_settings = ChatSettings {
            chat_id,
            auto_reply_enabled: true,
            reply_mode: "mention_only".to_string(),
            cooldown_seconds: 5,
            context_depth: 10,
            rag_enabled: true,
        };
        sqlx::query(
            r#"
            INSERT INTO chat_settings (chat_id, auto_reply_enabled, reply_mode, cooldown_seconds, context_depth, rag_enabled)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(chat_id)
        .bind(default_settings.auto_reply_enabled)
        .bind(&default_settings.reply_mode)
        .bind(default_settings.cooldown_seconds)
        .bind(default_settings.context_depth)
        .bind(default_settings.rag_enabled)
        .execute(pool)
        .await?;
        Ok(default_settings)
    }
}

pub async fn update_rag_settings(
    pool: &SqlitePool,
    chat_id: i64,
    rag_enabled: bool,
    context_depth: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chat_settings
        SET rag_enabled = ?, context_depth = ?, updated_at = CURRENT_TIMESTAMP
        WHERE chat_id = ?
        "#,
    )
    .bind(rag_enabled)
    .bind(context_depth)
    .bind(chat_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn toggle_rag_for_chat(
    pool: &SqlitePool,
    chat_id: i64,
    enabled: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chat_settings
        SET rag_enabled = ?, updated_at = CURRENT_TIMESTAMP
        WHERE chat_id = ?
        "#,
    )
    .bind(enabled)
    .bind(chat_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn toggle_auto_reply_for_chat(
    pool: &SqlitePool,
    chat_id: i64,
    enabled: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chat_settings
        SET auto_reply_enabled = ?, updated_at = CURRENT_TIMESTAMP
        WHERE chat_id = ?
        "#,
    )
    .bind(enabled)
    .bind(chat_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_reply_mode_for_chat(
    pool: &SqlitePool,
    chat_id: i64,
    reply_mode: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chat_settings
        SET reply_mode = ?, updated_at = CURRENT_TIMESTAMP
        WHERE chat_id = ?
        "#,
    )
    .bind(reply_mode)
    .bind(chat_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_cooldown_for_chat(
    pool: &SqlitePool,
    chat_id: i64,
    cooldown_seconds: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE chat_settings
        SET cooldown_seconds = ?, updated_at = CURRENT_TIMESTAMP
        WHERE chat_id = ?
        "#,
    )
    .bind(cooldown_seconds)
    .bind(chat_id)
    .execute(pool)
    .await?;

    Ok(())
}


// --- Public Functions: Messages & RAG ---

pub async fn save_message(pool: &SqlitePool, msg: &Message) -> Result<i64, sqlx::Error> {
    let user = msg.from.as_ref();
    let user_id = user.map(|u| u.id.0 as i64);
    let username = user.map(|u| u.full_name());
    let text = msg.text();
    let sent_at = chrono::DateTime::from_timestamp(msg.date.timestamp(), 0)
        .unwrap()
        .naive_utc();
    
    let message_id_i64 = msg.id.0 as i64;
    let chat_id_i64 = msg.chat.id.0;

    let inserted_id = sqlx::query(
        r#"
        INSERT INTO messages (message_id, chat_id, user_id, username, text, sent_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(message_id_i64)
    .bind(chat_id_i64)
    .bind(user_id)
    .bind(username)
    .bind(text)
    .bind(sent_at)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(inserted_id)
}

pub async fn save_embedding(
    pool: &SqlitePool,
    message_db_id: i64,
    chunk_text: &str,
    embedding: &[f64],
) -> Result<(), anyhow::Error> {
    let encoded_embedding = serialize(embedding)?;

    sqlx::query(
        r#"
        INSERT INTO memory_chunks (message_id, chunk_text, embedding)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(message_db_id)
    .bind(chunk_text)
    .bind(encoded_embedding)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_similar_chunks(
    pool: &SqlitePool,
    chat_id: i64,
    query_embedding: &[f64],
    limit: u32,
) -> Result<Vec<String>, sqlx::Error> {
    let chunks: Vec<MemoryChunk> = sqlx::query(
        r#"
        SELECT mc.id, mc.message_id, mc.chunk_text, mc.embedding
        FROM memory_chunks AS mc
        JOIN messages ON messages.id = mc.message_id
        WHERE messages.chat_id = ? AND mc.embedding IS NOT NULL
        "#,
    )
    .bind(chat_id)
    .map(|row: SqliteRow| MemoryChunk {
        id: row.get("id"),
        message_id: row.get("message_id"),
        chunk_text: row.get("chunk_text"),
        embedding: row.get("embedding"),
        importance_score: None,
        created_at: None,
    })
    .fetch_all(pool)
    .await?;

    let mut similarities: Vec<(f64, String)> = chunks
        .into_iter()
        .filter_map(|chunk| {
            if let Some(embedding_bytes) = chunk.embedding {
                match deserialize::<Vec<f64>>(&embedding_bytes) {
                    Ok(decoded_embedding) => {
                        let similarity = cosine_similarity(query_embedding, &decoded_embedding);
                        Some((similarity, chunk.chunk_text))
                    }
                    Err(e) => {
                        log::error!("Failed to deserialize embedding for chunk {}: {}", chunk.id, e);
                        None
                    }
                }
            } else {
                None
            }
        })
        .collect();

    similarities.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let top_chunks = similarities
        .into_iter()
        .take(limit as usize)
        .map(|(_, text)| text)
        .collect();

    Ok(top_chunks)
}

// --- Public Functions: Health Checks ---

pub async fn check_db_health(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// --- Public Functions: Broadcast ---

/// Get all unique chat IDs from messages table for broadcast
pub async fn get_all_chat_ids(pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
    let chat_ids: Vec<i64> = sqlx::query("SELECT DISTINCT chat_id FROM messages WHERE chat_id != 0")
        .map(|row: SqliteRow| row.get("chat_id"))
        .fetch_all(pool)
        .await?;
    
    Ok(chat_ids)
}

/// Get message count per chat for statistics
pub async fn get_chat_stats(pool: &SqlitePool) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    let stats: Vec<(i64, i64)> = sqlx::query(
        "SELECT chat_id, COUNT(*) as msg_count FROM messages GROUP BY chat_id ORDER BY msg_count DESC"
    )
    .map(|row: SqliteRow| (row.get("chat_id"), row.get("msg_count")))
    .fetch_all(pool)
    .await?;
    
    Ok(stats)
}

// --- Private Helpers ---

fn cosine_similarity(v1: &[f64], v2: &[f64]) -> f64 {
    let dot_product = v1.iter().zip(v2).map(|(a, b)| a * b).sum::<f64>();
    let norm_v1 = v1.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let norm_v2 = v2.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();

    if norm_v1 == 0.0 || norm_v2 == 0.0 {
        0.0
    } else {
        dot_product / (norm_v1 * norm_v2)
    }
}

// --- Time-Decay RAG Functions ---

/// Calculate time decay factor (exponential decay)
/// decay_rate: how fast memories fade (0.1 = slow, 1.0 = fast)
/// hours_old: age of the memory in hours
fn calculate_time_decay(hours_old: f64, decay_rate: f64) -> f64 {
    (-decay_rate * hours_old / 24.0).exp() // Decay per day
}

/// Find similar chunks with time-decay weighting
pub async fn find_similar_chunks_with_decay(
    pool: &SqlitePool,
    chat_id: i64,
    query_embedding: &[f64],
    limit: u32,
    decay_rate: f64,
) -> Result<Vec<String>, sqlx::Error> {
    let chunks: Vec<(MemoryChunk, NaiveDateTime)> = sqlx::query(
        r#"
        SELECT mc.id, mc.message_id, mc.chunk_text, mc.embedding, 
               mc.importance_score, mc.created_at, m.sent_at
        FROM memory_chunks AS mc
        JOIN messages m ON m.id = mc.message_id
        WHERE m.chat_id = ? AND mc.embedding IS NOT NULL
        "#,
    )
    .bind(chat_id)
    .map(|row: SqliteRow| {
        let chunk = MemoryChunk {
            id: row.get("id"),
            message_id: row.get("message_id"),
            chunk_text: row.get("chunk_text"),
            embedding: row.get("embedding"),
            importance_score: row.get::<Option<f64>, _>("importance_score"),
            created_at: row.get::<Option<NaiveDateTime>, _>("created_at"),
        };
        let sent_at: NaiveDateTime = row.get("sent_at");
        (chunk, sent_at)
    })
    .fetch_all(pool)
    .await?;

    let now = Utc::now().naive_utc();
    
    let mut scored_chunks: Vec<(f64, String)> = chunks
        .into_iter()
        .filter_map(|(chunk, sent_at)| {
            if let Some(embedding_bytes) = chunk.embedding {
                match deserialize::<Vec<f64>>(&embedding_bytes) {
                    Ok(decoded_embedding) => {
                        let similarity = cosine_similarity(query_embedding, &decoded_embedding);
                        
                        // Calculate time decay
                        let hours_old = (now - sent_at).num_hours() as f64;
                        let time_decay = calculate_time_decay(hours_old, decay_rate);
                        
                        // Combine similarity with time decay and importance
                        let importance = chunk.importance_score.unwrap_or(1.0);
                        let final_score = similarity * time_decay * importance;
                        
                        Some((final_score, chunk.chunk_text))
                    }
                    Err(e) => {
                        log::error!("Failed to deserialize embedding: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        })
        .collect();

    scored_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    Ok(scored_chunks
        .into_iter()
        .take(limit as usize)
        .map(|(_, text)| text)
        .collect())
}

/// Update importance score for a memory chunk
pub async fn update_chunk_importance(
    pool: &SqlitePool,
    chunk_id: i64,
    importance: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE memory_chunks SET importance_score = ? WHERE id = ?")
        .bind(importance)
        .bind(chunk_id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Summarization Functions ---

/// Save a chat summary
pub async fn save_chat_summary(
    pool: &SqlitePool,
    chat_id: i64,
    summary_text: &str,
    messages_from: i64,
    messages_to: i64,
    message_count: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO chat_summaries (chat_id, summary_text, messages_from, messages_to, message_count)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(chat_id)
    .bind(summary_text)
    .bind(messages_from)
    .bind(messages_to)
    .bind(message_count)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get recent summaries for a chat
pub async fn get_chat_summaries(
    pool: &SqlitePool,
    chat_id: i64,
    limit: u32,
) -> Result<Vec<ChatSummary>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT id, chat_id, summary_text, messages_from, messages_to, message_count, created_at
        FROM chat_summaries
        WHERE chat_id = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(chat_id)
    .bind(limit)
    .map(|row: SqliteRow| ChatSummary {
        id: row.get("id"),
        chat_id: row.get("chat_id"),
        summary_text: row.get("summary_text"),
        messages_from: row.get("messages_from"),
        messages_to: row.get("messages_to"),
        message_count: row.get("message_count"),
        created_at: row.get("created_at"),
    })
    .fetch_all(pool)
    .await
}

/// Get messages for summarization (messages not yet summarized)
pub async fn get_messages_for_summary(
    pool: &SqlitePool,
    chat_id: i64,
    limit: u32,
) -> Result<Vec<DbMessage>, sqlx::Error> {
    // Get the last summarized message ID
    let last_summary: Option<i64> = sqlx::query(
        "SELECT MAX(messages_to) as last_id FROM chat_summaries WHERE chat_id = ?"
    )
    .bind(chat_id)
    .map(|row: SqliteRow| row.get("last_id"))
    .fetch_optional(pool)
    .await?
    .flatten();

    let last_id = last_summary.unwrap_or(0);

    sqlx::query(
        r#"
        SELECT id, message_id, chat_id, user_id, username, text, sent_at
        FROM messages
        WHERE chat_id = ? AND id > ? AND text IS NOT NULL
        ORDER BY id ASC
        LIMIT ?
        "#,
    )
    .bind(chat_id)
    .bind(last_id)
    .bind(limit)
    .map(|row: SqliteRow| DbMessage {
        id: row.get("id"),
        message_id: row.get("message_id"),
        chat_id: row.get("chat_id"),
        user_id: row.get("user_id"),
        username: row.get("username"),
        text: row.get("text"),
        sent_at: row.get("sent_at"),
    })
    .fetch_all(pool)
    .await
}

/// Count unsummarized messages for a chat
pub async fn count_unsummarized_messages(
    pool: &SqlitePool,
    chat_id: i64,
) -> Result<i64, sqlx::Error> {
    let last_summary: Option<i64> = sqlx::query(
        "SELECT MAX(messages_to) as last_id FROM chat_summaries WHERE chat_id = ?"
    )
    .bind(chat_id)
    .map(|row: SqliteRow| row.get("last_id"))
    .fetch_optional(pool)
    .await?
    .flatten();

    let last_id = last_summary.unwrap_or(0);

    let count: i64 = sqlx::query(
        "SELECT COUNT(*) as cnt FROM messages WHERE chat_id = ? AND id > ? AND text IS NOT NULL"
    )
    .bind(chat_id)
    .bind(last_id)
    .map(|row: SqliteRow| row.get("cnt"))
    .fetch_one(pool)
    .await?;

    Ok(count)
}


// --- Persona Export/Import Functions ---

/// Export a single persona to JSON format
pub async fn export_persona(pool: &SqlitePool, id: i64) -> Result<Option<String>, sqlx::Error> {
    match get_persona_by_id(pool, id).await? {
        Some(persona) => {
            let export: PersonaExport = persona.into();
            Ok(serde_json::to_string_pretty(&export).ok())
        }
        None => Ok(None),
    }
}

/// Export all personas to JSON format
pub async fn export_all_personas(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let personas = get_all_personas(pool).await?;
    let exports: Vec<PersonaExport> = personas.into_iter().map(|p| p.into()).collect();
    Ok(serde_json::to_string_pretty(&exports).unwrap_or_else(|_| "[]".to_string()))
}

/// Import a persona from JSON format
pub async fn import_persona(pool: &SqlitePool, json: &str) -> Result<i64, ImportError> {
    let export: PersonaExport = serde_json::from_str(json)
        .map_err(|e: serde_json::Error| ImportError::ParseError(e.to_string()))?;
    
    if export.name.is_empty() || export.prompt.is_empty() {
        return Err(ImportError::ValidationError("Name and prompt cannot be empty".to_string()));
    }
    
    create_persona(pool, &export.name, &export.prompt)
        .await
        .map_err(|e| ImportError::DatabaseError(e.to_string()))
}

/// Import multiple personas from JSON array
pub async fn import_personas(pool: &SqlitePool, json: &str) -> Result<Vec<i64>, ImportError> {
    let exports: Vec<PersonaExport> = serde_json::from_str(json)
        .map_err(|e: serde_json::Error| ImportError::ParseError(e.to_string()))?;
    
    let mut ids = Vec::new();
    for export in exports {
        if export.name.is_empty() || export.prompt.is_empty() {
            continue;
        }
        match create_persona(pool, &export.name, &export.prompt).await {
            Ok(id) => ids.push(id),
            Err(e) => log::warn!("Failed to import persona '{}': {}", export.name, e),
        }
    }
    Ok(ids)
}

#[derive(Debug)]
pub enum ImportError {
    ParseError(String),
    ValidationError(String),
    DatabaseError(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::ParseError(e) => write!(f, "Parse error: {}", e),
            ImportError::ValidationError(e) => write!(f, "Validation error: {}", e),
            ImportError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for ImportError {}
