use bincode::{deserialize, serialize};
use futures::stream::TryStreamExt;
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


// --- Public Functions: Messages & RAG ---

pub async fn save_message(pool: &SqlitePool, msg: &Message) -> Result<i64, sqlx::Error> {
    let user = msg.from();
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
