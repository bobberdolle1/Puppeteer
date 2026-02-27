use super::models::*;
use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// Repository for account operations
pub struct AccountRepository;

impl AccountRepository {
    /// Create a new account
    pub async fn create(pool: &SqlitePool, new_account: NewAccount) -> Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            INSERT INTO accounts (phone_number, session_data, system_prompt, reply_probability, allowed_chats)
            VALUES (?, ?, ?, 100, '[]')
            RETURNING *
            "#,
        )
        .bind(&new_account.phone_number)
        .bind(&new_account.session_data)
        .bind(&new_account.system_prompt)
        .fetch_one(pool)
        .await
        .context("Failed to create account")?;

        tracing::info!("Created account {} with ID {}", account.phone_number, account.id);
        Ok(account)
    }

    /// Get an account by ID
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch account")?;

        Ok(account)
    }

    /// Get an account by phone number
    pub async fn get_by_phone(pool: &SqlitePool, phone: &str) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE phone_number = ?"
        )
        .bind(phone)
        .fetch_optional(pool)
        .await
        .context("Failed to fetch account by phone")?;

        Ok(account)
    }

    /// List all accounts
    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .context("Failed to list accounts")?;

        Ok(accounts)
    }

    /// List only active accounts
    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE is_active = 1 ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .context("Failed to list active accounts")?;

        Ok(accounts)
    }

    /// Update account's system prompt
    pub async fn update_system_prompt(
        pool: &SqlitePool,
        account_id: i64,
        new_prompt: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE accounts SET system_prompt = ? WHERE id = ?"
        )
        .bind(new_prompt)
        .bind(account_id)
        .execute(pool)
        .await
        .context("Failed to update system prompt")?;

        tracing::info!("Updated system prompt for account {}", account_id);
        Ok(())
    }

    /// Update account's active status
    pub async fn set_active(pool: &SqlitePool, account_id: i64, is_active: bool) -> Result<()> {
        sqlx::query(
            "UPDATE accounts SET is_active = ? WHERE id = ?"
        )
        .bind(is_active)
        .bind(account_id)
        .execute(pool)
        .await
        .context("Failed to update account status")?;

        tracing::info!("Set account {} active status to {}", account_id, is_active);
        Ok(())
    }

    /// Delete an account
    pub async fn delete(pool: &SqlitePool, account_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(account_id)
            .execute(pool)
            .await
            .context("Failed to delete account")?;

        tracing::info!("Deleted account {}", account_id);
        Ok(())
    }

    /// Update account's reply probability
    pub async fn update_reply_probability(
        pool: &SqlitePool,
        account_id: i64,
        probability: i64,
    ) -> Result<()> {
        if !(0..=100).contains(&probability) {
            anyhow::bail!("Reply probability must be between 0 and 100");
        }

        sqlx::query(
            "UPDATE accounts SET reply_probability = ? WHERE id = ?"
        )
        .bind(probability)
        .bind(account_id)
        .execute(pool)
        .await
        .context("Failed to update reply probability")?;

        tracing::info!("Updated reply probability for account {} to {}", account_id, probability);
        Ok(())
    }

    /// Add a chat to the allowed chats list
    pub async fn add_allowed_chat(
        pool: &SqlitePool,
        account_id: i64,
        chat_id: i64,
    ) -> Result<()> {
        let account = Self::get_by_id(pool, account_id)
            .await?
            .context("Account not found")?;

        let mut allowed_chats: Vec<i64> = serde_json::from_str(&account.allowed_chats)
            .unwrap_or_default();

        if !allowed_chats.contains(&chat_id) {
            allowed_chats.push(chat_id);
            let json = serde_json::to_string(&allowed_chats)?;

            sqlx::query("UPDATE accounts SET allowed_chats = ? WHERE id = ?")
                .bind(&json)
                .bind(account_id)
                .execute(pool)
                .await
                .context("Failed to add allowed chat")?;

            tracing::info!("Added chat {} to allowed list for account {}", chat_id, account_id);
        }

        Ok(())
    }

    /// Remove a chat from the allowed chats list
    pub async fn remove_allowed_chat(
        pool: &SqlitePool,
        account_id: i64,
        chat_id: i64,
    ) -> Result<()> {
        let account = Self::get_by_id(pool, account_id)
            .await?
            .context("Account not found")?;

        let mut allowed_chats: Vec<i64> = serde_json::from_str(&account.allowed_chats)
            .unwrap_or_default();

        allowed_chats.retain(|&id| id != chat_id);
        let json = serde_json::to_string(&allowed_chats)?;

        sqlx::query("UPDATE accounts SET allowed_chats = ? WHERE id = ?")
            .bind(&json)
            .bind(account_id)
            .execute(pool)
            .await
            .context("Failed to remove allowed chat")?;

        tracing::info!("Removed chat {} from allowed list for account {}", chat_id, account_id);
        Ok(())
    }
}

/// Repository for message history operations
pub struct MessageRepository;

impl MessageRepository {
    /// Save a new message to history
    pub async fn create(pool: &SqlitePool, new_message: NewMessage) -> Result<MessageHistory> {
        let message = sqlx::query_as::<_, MessageHistory>(
            r#"
            INSERT INTO messages_history (account_id, chat_id, role, content)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(new_message.account_id)
        .bind(new_message.chat_id)
        .bind(new_message.role.as_str())
        .bind(&new_message.content)
        .fetch_one(pool)
        .await
        .context("Failed to create message")?;

        Ok(message)
    }

    /// Get recent messages for a specific account and chat (for RAG context)
    pub async fn get_recent_messages(
        pool: &SqlitePool,
        account_id: i64,
        chat_id: i64,
        limit: i64,
    ) -> Result<Vec<MessageHistory>> {
        let messages = sqlx::query_as::<_, MessageHistory>(
            r#"
            SELECT * FROM messages_history
            WHERE account_id = ? AND chat_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(account_id)
        .bind(chat_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .context("Failed to fetch recent messages")?;

        // Reverse to get chronological order
        Ok(messages.into_iter().rev().collect())
    }

    /// Get total message count for an account
    pub async fn count_by_account(pool: &SqlitePool, account_id: i64) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM messages_history WHERE account_id = ?"
        )
        .bind(account_id)
        .fetch_one(pool)
        .await
        .context("Failed to count messages")?;

        Ok(count.0)
    }

    /// Delete old messages (cleanup)
    pub async fn delete_older_than(pool: &SqlitePool, days: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM messages_history
            WHERE created_at < datetime('now', '-' || ? || ' days')
            "#,
        )
        .bind(days)
        .execute(pool)
        .await
        .context("Failed to delete old messages")?;

        tracing::info!("Deleted {} old messages", result.rows_affected());
        Ok(result.rows_affected())
    }
}
