# 🏗️ Архитектура

## Обзор

```
┌─────────────────────────────────────────────────────────────┐
│                        Telegram                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      PersonaForge                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   Handlers  │  │   WebApp    │  │   Security  │          │
│  │  (teloxide) │  │   (axum)    │  │             │          │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘          │
│         │                │                │                  │
│         └────────────────┼────────────────┘                  │
│                          │                                   │
│                    ┌─────┴─────┐                             │
│                    │  AppState │                             │
│                    └─────┬─────┘                             │
│         ┌────────────────┼────────────────┐                  │
│         │                │                │                  │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐          │
│  │     DB      │  │     LLM     │  │    Voice    │          │
│  │   (sqlx)    │  │  (Ollama)   │  │  (Whisper)  │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

## Структура проекта

```
src/
├── main.rs              # Entry point, dispatcher setup
├── lib.rs               # Module exports
├── config.rs            # Environment configuration
├── state.rs             # Shared state (AppState)
├── logging.rs           # Colored logging system
│
├── bot/
│   └── handlers/
│       ├── mod.rs
│       ├── commands.rs  # /menu, /status, etc.
│       ├── messages.rs  # Message processing, RAG
│       └── callbacks.rs # Inline keyboard handlers
│
├── db/
│   └── mod.rs           # SQLx queries
│
├── llm/
│   ├── mod.rs
│   └── client.rs        # Ollama API client
│
├── security/
│   └── mod.rs           # Prompt injection, rate limiting
│
├── voice/
│   └── mod.rs           # Whisper client
│
├── web/
│   ├── mod.rs
│   └── search.rs        # DuckDuckGo client
│
└── webapp/
    ├── mod.rs
    ├── server.rs        # Axum router
    ├── api.rs           # REST endpoints
    ├── auth.rs          # Telegram WebApp auth
    └── static/          # Embedded frontend
```

## Ключевые компоненты

### AppState

Центральное хранилище состояния:

```rust
pub struct AppState {
    pub config: Config,
    pub db_pool: SqlitePool,
    pub http_client: reqwest::Client,
    pub llm_semaphore: Arc<Semaphore>,
    pub wizard_states: Arc<Mutex<HashMap<...>>>,
    pub triggers_cache: Arc<Mutex<HashMap<...>>>,
    // ...
}
```

### Handler Flow

```
Message → Filter → Handler → Response
                      │
                      ├── Check triggers
                      ├── Security check
                      ├── RAG retrieval
                      ├── LLM generation
                      └── Send response
```

### Database Schema

```sql
-- Персоны
personas (id, name, prompt, display_name, triggers, created_at)

-- Настройки чатов
chat_settings (chat_id, active_persona_id, rag_enabled, triggers, ...)

-- История сообщений
messages (id, chat_id, user_id, role, content, created_at)

-- RAG память
memory_chunks (id, chat_id, content, embedding, importance, created_at)

-- Суммаризации
chat_summaries (id, chat_id, summary, message_count, created_at)

-- Runtime конфиг
runtime_config (key, value, updated_at)
```

## Паттерны

### Async/Await

Всё асинхронное через Tokio:

```rust
async fn handle_message(msg: Message, state: AppState) -> Result<()> {
    let response = state.llm_client.generate(&prompt).await?;
    // ...
}
```

### Error Handling

`anyhow` для application errors:

```rust
use anyhow::{Context, Result};

async fn do_something() -> Result<()> {
    let data = fetch_data()
        .await
        .context("Failed to fetch data")?;
    Ok(())
}
```

### Concurrency Control

Semaphore для ограничения LLM запросов:

```rust
let _permit = state.llm_semaphore.acquire().await?;
// Только MAX_CONCURRENT_LLM_REQUESTS одновременно
let response = generate(&prompt).await?;
```

### State Sharing

`Arc<Mutex<>>` для shared mutable state:

```rust
let mut cache = state.triggers_cache.lock().await;
cache.insert(chat_id, triggers);
```

## Потоки данных

### Сообщение → Ответ

```
1. Telegram webhook/polling
2. teloxide dispatcher
3. Filter: is_relevant_message?
4. Security: check_injection()
5. RAG: retrieve_context()
6. LLM: generate_response()
7. Send: bot.send_message()
```

### Mini App → API

```
1. Telegram WebApp opens
2. initData validation (HMAC)
3. Owner check
4. API request
5. Database query
6. JSON response
```

## Расширение

### Добавить новую команду

1. `commands.rs`: добавь handler
2. `mod.rs`: зарегистрируй в dispatcher
3. Готово

### Добавить новый API endpoint

1. `api.rs`: добавь handler
2. `server.rs`: добавь route
3. Готово

### Добавить новую таблицу

1. Создай миграцию: `migrations/YYYYMMDDHHMMSS_name.sql`
2. `db/mod.rs`: добавь queries
3. Перезапусти — миграция применится автоматически

---

➡️ Далее: [[API|REST API]]
