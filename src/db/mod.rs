use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

pub mod models;
pub mod repository;

pub use models::*;
pub use repository::*;

/// Initialize the database connection pool with WAL mode for high concurrency
pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    // Ensure the data directory exists
    if let Some(parent) = Path::new(database_url.trim_start_matches("sqlite:")).parent() {
        tokio::fs::create_dir_all(parent).await
            .context("Failed to create data directory")?;
    }

    // Create connection pool with optimized settings
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("Failed to connect to database")?;

    // Enable WAL mode for better concurrency
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await
        .context("Failed to enable WAL mode")?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .context("Failed to enable foreign keys")?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    tracing::info!("Database initialized successfully with WAL mode enabled");

    Ok(pool)
}
