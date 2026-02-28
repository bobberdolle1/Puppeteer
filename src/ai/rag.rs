use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

/// Generate embedding for text using Ollama
pub async fn generate_embedding(
    client: &Client,
    ollama_url: &str,
    model: &str,
    text: &str,
) -> Result<Vec<f32>> {
    let url = format!("{}/api/embeddings", ollama_url);
    
    let request = EmbeddingRequest {
        model: model.to_string(),
        prompt: text.to_string(),
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to send embedding request")?;

    let embedding_response: EmbeddingResponse = response
        .json()
        .await
        .context("Failed to parse embedding response")?;

    Ok(embedding_response.embedding)
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

/// Store a memory with its embedding
pub async fn store_memory(
    pool: &SqlitePool,
    account_id: i64,
    chat_id: i64,
    content: &str,
    embedding: &[f32],
) -> Result<()> {
    let embedding_bytes = bincode::serialize(embedding)
        .context("Failed to serialize embedding")?;

    sqlx::query(
        r#"
        INSERT INTO long_term_memory (account_id, chat_id, content, embedding)
        VALUES (?, ?, ?, ?)
        "#
    )
    .bind(account_id)
    .bind(chat_id)
    .bind(content)
    .bind(embedding_bytes)
    .execute(pool)
    .await
    .context("Failed to store memory")?;

    Ok(())
}

#[derive(Debug)]
pub struct Memory {
    pub content: String,
    pub similarity: f32,
}

/// Retrieve top N most relevant memories for a query
pub async fn retrieve_memories(
    pool: &SqlitePool,
    account_id: i64,
    chat_id: i64,
    query_embedding: &[f32],
    top_n: usize,
) -> Result<Vec<Memory>> {
    let rows = sqlx::query(
        r#"
        SELECT content, embedding
        FROM long_term_memory
        WHERE account_id = ? AND chat_id = ?
        ORDER BY created_at DESC
        LIMIT 100
        "#
    )
    .bind(account_id)
    .bind(chat_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch memories")?;

    let mut memories_with_similarity: Vec<Memory> = rows
        .into_iter()
        .filter_map(|row| {
            let content: String = row.try_get("content").ok()?;
            let embedding_bytes: Vec<u8> = row.try_get("embedding").ok()?;
            let embedding: Vec<f32> = bincode::deserialize(&embedding_bytes).ok()?;
            let similarity = cosine_similarity(query_embedding, &embedding);
            
            Some(Memory {
                content,
                similarity,
            })
        })
        .collect();

    // Sort by similarity (highest first)
    memories_with_similarity.sort_by(|a, b| {
        b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top N
    Ok(memories_with_similarity.into_iter().take(top_n).collect())
}

/// Clean up old memories (keep last 1000 per chat)
pub async fn cleanup_old_memories(
    pool: &SqlitePool,
    account_id: i64,
    chat_id: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM long_term_memory
        WHERE account_id = ? AND chat_id = ?
        AND id NOT IN (
            SELECT id FROM long_term_memory
            WHERE account_id = ? AND chat_id = ?
            ORDER BY created_at DESC
            LIMIT 1000
        )
        "#
    )
    .bind(account_id)
    .bind(chat_id)
    .bind(account_id)
    .bind(chat_id)
    .execute(pool)
    .await
    .context("Failed to cleanup old memories")?;

    Ok(())
}
