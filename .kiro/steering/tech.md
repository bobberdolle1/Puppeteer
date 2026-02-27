# Tech Stack

## Language & Runtime
- **Rust** (Edition 2021)
- **Tokio** async runtime

## Core Dependencies
- `teloxide` - Telegram Bot API framework (admin bot)
- `rust-tdlib` - TDLib Rust wrapper for MTProto (userbots)
- `sqlx` - Async SQLite with compile-time query checking
- `reqwest` - HTTP client for Ollama/Whisper APIs
- `serde` / `serde_json` - Serialization
- `anyhow` / `thiserror` - Error handling
- `tracing` / `tracing-subscriber` - Logging

## External Services
- **Ollama** - Local LLM inference (chat, embeddings, vision)
- **Whisper API** - Voice transcription (optional)
- **TDLib** - Telegram Database Library for MTProto

## Database
- SQLite with WAL mode for high concurrency
- SQLx migrations in `migrations/`
- Tables: `accounts`, `messages_history`, `chat_whitelist`

## Build & Run

```bash
# Development
cargo run

# Release build
cargo build --release

# Run binary
./target/release/puppeteer
```

## Docker

```bash
# Build and run
docker-compose up --build

# Just build
docker build -t puppeteer .
```

## Environment
- Configuration via `.env` file (see `.env.example`)
- Required: `BOT_TOKEN`, `OWNER_IDS`, `TELEGRAM_API_ID`, `TELEGRAM_API_HASH`, `DATABASE_URL`
- Ollama URL defaults to `http://localhost:11434`

## Migrations
- Auto-applied on startup via `sqlx::migrate!()`
- Files in `migrations/` with timestamp prefix
- Create new: name file `YYYYMMDDHHMMSS_description.sql`

## TDLib Setup
- C++ build dependencies required: `build-essential`, `cmake`, `gperf`, `libssl-dev`
- Session files stored in `data/tdlib/{phone_number}/`
- Each userbot has isolated TDLib instance
