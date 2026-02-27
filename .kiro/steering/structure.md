# Project Structure

```
src/
├── main.rs              # Entry point, admin bot dispatcher setup
├── lib.rs               # Module exports
├── config.rs            # Environment config (Config struct)
├── state.rs             # Shared state (AppState), userbot registry
│
├── bot/
│   ├── mod.rs           # Admin bot setup and dialogue routing
│   ├── handlers.rs      # Admin command handlers (/add_account, /list_accounts, etc.)
│   ├── dialogues.rs     # TDLib authentication flows (phone, code, 2FA)
│   └── middleware.rs    # Owner verification middleware
│
├── userbot/
│   ├── mod.rs           # Userbot module exports
│   └── worker.rs        # MTProto event loop, message handling, humanization
│
├── ai/
│   ├── mod.rs           # AI module exports
│   ├── ollama.rs        # Ollama API client (chat, embeddings, vision)
│   └── whisper.rs       # Whisper API client (voice transcription)
│
└── db/
    ├── mod.rs           # Database operations (accounts, messages, whitelist)
    ├── models.rs        # Data models (Account, Message, etc.)
    └── repository.rs    # Repository pattern for database access

migrations/              # SQLx migration files
data/
├── puppeteer.db         # SQLite database
└── tdlib/               # TDLib session files per account
```

## Key Patterns

### State Management
- `AppState` contains config, database pool, and userbot registry
- `userbots: Arc<RwLock<HashMap<i64, UserbotHandle>>>` tracks active userbots
- Each `UserbotHandle` contains TDLib client and shutdown signal

### Admin Bot Flow
1. `main.rs` sets up teloxide dispatcher with dialogue handler
2. `handlers.rs` processes admin commands (/add_account, /start_account, etc.)
3. `dialogues.rs` manages multi-step authentication (phone → code → 2FA → prompt)
4. `middleware.rs` enforces owner-only access via `OWNER_IDS`

### Userbot Lifecycle
1. Admin sends `/add_account` command
2. Dialogue flow authenticates with TDLib (MTProto)
3. Account saved to database with session data
4. `userbot::spawn_userbot()` creates worker task
5. Worker runs event loop, processes messages with AI
6. Humanization: read delays, typing indicators, reply probability

### Database Access
- Repository pattern in `src/db/repository.rs`
- All queries use SQLx with parameterized statements
- Tables: `accounts`, `messages_history`, `chat_whitelist`

### Authentication
- Admin bot: Owner verification via `OWNER_IDS` environment variable
- Userbots: TDLib handles MTProto authentication with phone/code/2FA
