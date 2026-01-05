use bincode::{deserialize, serialize};
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
}

#[derive(Debug, FromRow, Clone)]
pub struct Persona {
    pub id: i64,
    pub name: String,
    pub prompt: String,
    pub is_active: bool,
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
