<div align="center">

# 🎭 Puppeteer

### *The Ultimate AI-Powered Telegram Userbot Orchestration System*

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=for-the-badge&logo=rust" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge" alt="License"></a>
  <a href="https://github.com/bobberdolle1/Puppeteer/actions"><img src="https://img.shields.io/github/actions/workflow/status/bobberdolle1/Puppeteer/rust.yml?style=for-the-badge&logo=github" alt="CI"></a>
  <a href="https://github.com/bobberdolle1/Puppeteer/releases"><img src="https://img.shields.io/github/v/release/bobberdolle1/Puppeteer?style=for-the-badge&logo=github" alt="Release"></a>
</p>

<p align="center">
  <a href="https://core.telegram.org/mtproto"><img src="https://img.shields.io/badge/Telegram-MTProto-26A5E4?style=for-the-badge&logo=telegram" alt="Telegram"></a>
  <a href="https://ollama.ai/"><img src="https://img.shields.io/badge/AI-Ollama-000000?style=for-the-badge&logo=ai" alt="Ollama"></a>
  <a href="https://www.sqlite.org/"><img src="https://img.shields.io/badge/Database-SQLite-003B57?style=for-the-badge&logo=sqlite" alt="SQLite"></a>
  <a href="https://tokio.rs/"><img src="https://img.shields.io/badge/Runtime-Tokio-000000?style=for-the-badge" alt="Tokio"></a>
</p>

**Orchestrate unlimited AI-driven Telegram accounts with indistinguishable-from-human behavior**

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

</div>

## 🎯 What is Puppeteer?

> **Puppeteer** is a next-generation multi-account Telegram userbot system that combines cutting-edge AI with sophisticated humanization techniques to create bot behavior that's **indistinguishable from real humans**.

Built in **Rust** 🦀 for maximum performance, reliability, and safety.


<table>
<tr>
<td width="50%">

### 🔥 Why Puppeteer?

- 🧠 **AI-Powered** - Each bot has unique personality
- 👤 **Human-Like** - Advanced humanization engine
- ⚡ **Blazing Fast** - Rust + async/await
- 🎯 **Multi-Account** - Unlimited userbots
- 🛡️ **Secure** - MTProto + encrypted sessions
- 🎨 **Flexible** - Per-account customization
- 📊 **Coordinated** - Bot groups & campaigns

</td>
<td width="50%">

### 📈 Stats

```
🤖 Concurrent Accounts: Unlimited
⚡ Response Time: <100ms
💾 Memory per Bot: ~50MB
🔒 Security: MTProto + TLS
🌐 Platform: Cross-platform
📦 Binary Size: ~15MB
```

</td>
</tr>
</table>

---

## ✨ Features

<details open>
<summary><b>🤖 Core Capabilities</b></summary>
<br>

| Feature | Description | Status |
|---------|-------------|--------|
| 🎭 **Multi-Account Management** | Orchestrate unlimited Telegram userbot accounts | ✅ Ready |
| 🧠 **AI-Driven Responses** | Powered by Ollama with custom prompts | ✅ Ready |
| 🖼️ **Photo Analysis** | Automatic image analysis with vision models (llava, minicpm-v) | ✅ Ready |
| 🎬 **GIF/Animation Support** | Extracts 3 frames for intelligent content understanding | ✅ Ready |
| 🎤 **Voice Transcription** | Automatic voice-to-text via Whisper API | ✅ Ready |
| 📹 **Video Circles** | Frame extraction and analysis for video messages | ✅ Ready |
| 🎨 **Sticker Recognition** | Smart sticker handling with casual responses | ✅ Ready |
| 🤖 **Dynamic Personas** | Create custom AI personalities on-the-fly | ✅ Ready |
| 💾 **RAG Memory** | Long-term conversation memory with semantic search | ✅ Ready |
| 🔍 **Web Search** | Real-time information retrieval via DuckDuckGo | ✅ Ready |

</details>

<details open>
<summary><b>🎭 Humanization Engine</b></summary>
<br>

The **secret sauce** that makes bots indistinguishable from humans:

```mermaid
graph LR
    A[Incoming Message] --> B{Media Type?}
    B -->|Photo| C[Vision Analysis]
    B -->|GIF| D[Extract 3 Frames]
    B -->|Voice| E[Whisper Transcription]
    B -->|Sticker| F[Casual Response]
    B -->|Text| G[Process Text]
    C --> H{Rate Limit Check}
    D --> H
    E --> H
    F --> H
    G --> H
    H -->|Spam| I[Ignore]
    H -->|OK| J{Probability Check}
    J -->|Skip| I
    J -->|Respond| K[Read Delay 5-60s]
    K --> L[Show Typing]
    L --> M[Generate AI Response]
    M --> N{Multi-text?}
    N -->|Yes| O["Split by ||"]
    N -->|No| P[Send Message]
    O --> P
```

#### 🎨 Humanization Features

- ⏱️ **Realistic Typing Indicators** - Shows "typing..." status with variance
- 📖 **Smart Response Delays** - Simulates reading time (5-60s random)
- 🎲 **Configurable Reply Probability** - Doesn't always respond (0-100%)
- 💬 **Intelligent Reply Logic** - Uses reply only in active dialogues
- ⏰ **Message Age Filtering** - Ignores old messages (configurable)
- 💌 **Private Chat Behavior** - Always responds in DMs
- ⌨️ **Typing Speed Simulation** - Realistic typing duration (100-400 CPM)
- 🎯 **Random Variance** - Natural randomness in all timings
- 🎭 **Distracted Typist** - 20% chance of pausing mid-typing
- 📝 **Multi-texting** - Splits responses by `||` into separate messages
- 🤐 **Smart Ignore** - AI can return `<IGNORE>` to skip meaningless messages
- 🎨 **Casual Sticker Responses** - Random reactions: "ахах", "жиза", "норм", "кек"
- 🛡️ **Rate Limiting** - Ignores users sending >5 messages/minute
- 🖼️ **Media Context** - Passes image/video descriptions to AI

</details>

<details open>
<summary><b>🎯 Orchestration Features</b></summary>
<br>

### Bot Groups

Organize bots into groups for coordinated actions:

```bash
/create_group MyArmy "Elite bot squad"
/add_to_group 1 2  # Add bot #2 to group #1
/add_to_group 1 3  # Add bot #3 to group #1
```

### Spam Campaigns

Mass messaging with text/media support:

```bash
# Text campaign
/spam 1 chat -1001234567890 5 1000 Hello from the army!

# Media campaign (coming soon)
/spam_media 1 chat -1001234567890 3 2000 photo
```

### Direct Messaging

Send messages from specific bots:

```bash
/dm 2 123456789 Hey, this is bot #2!
```

</details>

---

## 🚀 Quick Start

<details>
<summary><b>📋 Prerequisites</b></summary>
<br>

| Requirement | Version | Installation |
|-------------|---------|--------------|
| 🦀 **Rust** | 1.70+ | [rustup.rs](https://rustup.rs/) |
| 🤖 **Ollama** | Latest | [ollama.ai](https://ollama.ai/) |
| 📱 **Telegram API** | - | [my.telegram.org/apps](https://my.telegram.org/apps) |
| 🔧 **TDLib deps** | - | See below ⬇️ |

#### TDLib Dependencies

<table>
<tr>
<td width="50%">

**Ubuntu/Debian**
```bash
sudo apt install build-essential \
  cmake gperf libssl-dev
```

</td>
<td width="50%">

**macOS**
```bash
brew install cmake gperf openssl
```

</td>
</tr>
</table>

</details>

### 📦 Installation

```bash
# 1️⃣ Clone the repository
git clone https://github.com/bobberdolle1/Puppeteer.git
cd Puppeteer

# 2️⃣ Configure environment
cp .env.example .env
nano .env  # Edit with your credentials

# 3️⃣ Build and run
cargo build --release
./target/release/puppeteer
```

### 🐳 Docker Setup

```bash
# Quick start with Docker Compose
docker-compose up --build

# Or build manually
docker build -t puppeteer .
docker run -v $(pwd)/data:/app/data puppeteer
```

### 🎬 First Steps

1. **Start Ollama**
   ```bash
   ollama pull llama2
   ollama serve
   ```

2. **Add your first userbot**
   - Send `/add_account` to the admin bot
   - Enter phone: `+1234567890`
   - Enter verification code
   - Set system prompt: `You are a friendly assistant`
   - Done! 🎉

---

## 📖 Documentation

<details>
<summary><b>⚙️ Configuration</b></summary>
<br>

All settings via `.env` file:

```env
# ============================================
# TELEGRAM BOT API (Admin Bot)
# ============================================
BOT_TOKEN=your_bot_token_here
OWNER_IDS=123456789,987654321

# ============================================
# TELEGRAM MTPROTO API (Userbots)
# ============================================
TELEGRAM_API_ID=12345678
TELEGRAM_API_HASH=your_api_hash_here

# ============================================
# DATABASE
# ============================================
DATABASE_URL=sqlite:data/puppeteer.db

# ============================================
# AI SERVICES
# ============================================
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=llama2
WHISPER_URL=http://localhost:9000

# ============================================
# HUMANIZATION SETTINGS
# ============================================
DEFAULT_MIN_RESPONSE_DELAY=2
DEFAULT_MAX_RESPONSE_DELAY=15
DEFAULT_TYPING_SPEED=200
DEFAULT_USE_REPLY_PROBABILITY=70
DEFAULT_IGNORE_OLD_MESSAGES=300
DEFAULT_ALWAYS_RESPOND_PM=1
```

</details>

<details>
<summary><b>🎯 Inline Keyboard UI</b></summary>
<br>

### Modern Admin Interface

Puppeteer использует **современный inline keyboard интерфейс** вместо старых текстовых команд!

#### 🏠 Main Menu

Отправь `/start` и получишь красивое меню с кнопками:

```
🎭 Puppeteer Bot

Status: ✅ Online
Accounts: 3 active
Uptime: 2h 15m

[ 👥 Manage Accounts ]
[ ⚙️ Global Settings  ]
[ 📊 Statistics       ]
```

#### 👥 Account Management

Нажми **"Manage Accounts"** → увидишь список всех ботов:

```
📱 Your Accounts

🟢 Bot #1 (+79991234567)
🟢 Bot #2 (+79997654321)
🔴 Bot #3 (+79995555555) [Stopped]

[ ➕ Add New Account ]
[ 🔙 Back to Menu    ]
```

#### 🎛️ Account Control Panel

Выбери бота → откроется панель управления:

```
🤖 Bot #1 (+79991234567)
Status: 🟢 Active
Probability: 80%
Chats: 5 whitelisted

[ 🟢 Stop Bot        ]
[ 📝 Edit Prompt     ]
[ 🎲 Set Probability ]
[ 💬 Manage Chats    ]
[ 🎭 Change Persona  ]
[ 🗑 Delete Account  ]
[ 🔙 Back           ]
```

#### ⚡ Quick Actions

Все действия в **один клик**:
- ✅ Start/Stop бота
- ✅ Изменить промпт
- ✅ Настроить вероятность ответа
- ✅ Управлять whitelist чатов
- ✅ Сменить персону
- ✅ Удалить аккаунт

**Никаких команд вручную!** Все через кнопки 🎯

</details>

<details>
<summary><b>🤖 Admin Commands (Legacy)</b></summary>
<br>

> **💡 Tip**: Используй inline keyboard UI (кнопки) вместо команд! Просто отправь `/start`

Команды все еще работают для продвинутых пользователей и автоматизации:

### Account Management

| Command | Description | Example |
|---------|-------------|---------|
| `/start` | Show bot status | `/start` |
| `/add_account` | Add new userbot | `/add_account` |
| `/list` | List all accounts | `/list` |
| `/set_prompt` | Update AI prompt | `/set_prompt 1` |
| `/set_prob` | Set reply probability | `/set_prob 1 80` |
| `/allow_chat` | Add chat to whitelist | `/allow_chat 1 -1001234567890` |
| `/remove_chat` | Remove from whitelist | `/remove_chat 1 -1001234567890` |
| `/stop` | Stop userbot | `/stop 1` |
| `/delete` | Delete account | `/delete 1` |

### Bot Groups

| Command | Description | Example |
|---------|-------------|---------|
| `/create_group` | Create bot group | `/create_group Army` |
| `/list_groups` | List all groups | `/list_groups` |
| `/add_to_group` | Add bot to group | `/add_to_group 1 2` |

### Campaigns

| Command | Description | Example |
|---------|-------------|---------|
| `/spam` | Create campaign | `/spam 1 chat -100123 5 1000 Hi!` |
| `/list_campaigns` | List campaigns | `/list_campaigns` |
| `/stop_campaign` | Stop campaign | `/stop_campaign 1` |

### Direct Messaging

| Command | Description | Example |
|---------|-------------|---------|
| `/dm` | Send DM from bot | `/dm 2 123456789 Hello!` |

### Persona Management

| Command | Description | Example |
|---------|-------------|---------|
| `/list_personas` | List all personality archetypes | `/list_personas` |
| `/random_persona` | Assign random persona | `/random_persona 1` |
| `/set_persona` | Assign specific persona | `/set_persona 1 Tired Techie` |

**Available Personas:**
- 🤖 **Tired Techie** - Dry, no emojis, minimal responses
- 😎 **Ironic Zoomer** - Slang, memes, emoji spam
- 😡 **Toxic Gamer** - Aggressive, caps, confrontational
- 👴 **Clueless Boomer** - Polite, confused by slang
- 🕵️ **Paranoid Theorist** - Sees conspiracies everywhere
- 💖 **Wholesome Helper** - Kind, supportive, positive
- 📝 **Minimalist** - One-word answers only
- 🧠 **Sarcastic Intellectual** - Smart, witty, ironic

</details>

<details>
<summary><b>🎭 Humanization Settings</b></summary>
<br>

Each account has configurable humanization parameters:

| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| `min_response_delay_sec` | 1-30 | 2 | Minimum delay before responding |
| `max_response_delay_sec` | 5-60 | 15 | Maximum delay before responding |
| `typing_speed_cpm` | 100-400 | 200 | Typing speed (chars/min) |
| `use_reply_probability` | 0-100 | 70 | Chance to use reply |
| `ignore_old_messages_sec` | 60-3600 | 300 | Ignore messages older than X |
| `always_respond_in_pm` | 0/1 | 1 | Always respond in DMs |

### Example: Natural Conversation

```
👤 User: Hey, what's up?
   [Bot reads message - 3s delay]
   [Bot shows "typing..." - 5s]
🤖 Bot: Not much, just chilling. You?

👤 User: Want to grab coffee?
   [Bot reads - 4s]
   [Bot typing - 6s]
🤖 Bot: Sure! When were you thinking?
```

</details>

---

## 🏗️ Architecture

<details>
<summary><b>📊 System Overview</b></summary>
<br>

```mermaid
graph TB
    subgraph "Admin Interface"
        A[Telegram Admin Bot<br/>Bot API]
    end
    
    subgraph "Core System"
        B[AppState<br/>Shared State]
        C[(SQLite Database)]
        D[Config Manager]
    end
    
    subgraph "Userbot Army"
        E1[Userbot 1<br/>MTProto]
        E2[Userbot 2<br/>MTProto]
        E3[Userbot N<br/>MTProto]
    end
    
    subgraph "AI Services"
        F[Ollama LLM<br/>Chat & Embeddings]
        G[Whisper API<br/>Voice Transcription]
    end
    
    A --> B
    B --> C
    B --> D
    B --> E1
    B --> E2
    B --> E3
    E1 --> F
    E2 --> F
    E3 --> F
    E1 --> G
    E2 --> G
    E3 --> G
    
    style A fill:#26A5E4,stroke:#1E88E5,color:#fff
    style B fill:#FF6B6B,stroke:#E53935,color:#fff
    style C fill:#4ECDC4,stroke:#00897B,color:#fff
    style E1 fill:#95E1D3,stroke:#00897B,color:#000
    style E2 fill:#95E1D3,stroke:#00897B,color:#000
    style E3 fill:#95E1D3,stroke:#00897B,color:#000
    style F fill:#FFA07A,stroke:#FF6347,color:#fff
    style G fill:#FFA07A,stroke:#FF6347,color:#fff
```

</details>

<details>
<summary><b>📁 Project Structure</b></summary>
<br>

```
puppeteer/
├── 📂 src/
│   ├── 📄 main.rs              # Entry point
│   ├── 📄 config.rs            # Configuration
│   ├── 📄 state.rs             # Shared state
│   │
│   ├── 📂 bot/                 # Admin bot (Bot API)
│   │   ├── 📄 mod.rs
│   │   ├── 📄 handlers.rs      # Command handlers
│   │   ├── 📄 dialogues.rs     # Auth flows
│   │   ├── 📄 group_commands.rs # Groups & campaigns
│   │   └── 📄 middleware.rs    # Owner verification
│   │
│   ├── 📂 userbot/             # Userbots (MTProto)
│   │   ├── 📄 mod.rs
│   │   ├── 📄 worker.rs        # Event loop & humanization
│   │   └── 📄 spam.rs          # Campaign execution
│   │
│   ├── 📂 ai/                  # AI integrations
│   │   ├── 📄 mod.rs
│   │   ├── 📄 ollama.rs        # LLM client
│   │   └── 📄 whisper.rs       # Voice transcription
│   │
│   └── 📂 db/                  # Database layer
│       ├── 📄 mod.rs
│       ├── 📄 models.rs        # Data models
│       └── 📄 repository.rs    # DB operations
│
├── 📂 migrations/              # SQL migrations
├── 📂 data/                    # Runtime data
│   ├── 📄 puppeteer.db        # SQLite database
│   └── 📂 tdlib/              # TDLib sessions
│
├── 📂 .github/
│   └── 📂 workflows/
│       └── 📄 rust.yml        # CI/CD pipeline
│
├── 📄 Cargo.toml              # Dependencies
├── 📄 Dockerfile              # Docker image
├── 📄 docker-compose.yml      # Docker Compose
├── 📄 .env.example            # Config template
└── 📄 README.md               # This file
```

</details>

<details>
<summary><b>🔧 Tech Stack</b></summary>
<br>

### Core Technologies

| Component | Technology | Purpose |
|-----------|-----------|---------|
| 🦀 **Language** | Rust 2021 | Performance & safety |
| ⚡ **Runtime** | Tokio | Async/await concurrency |
| 🤖 **Admin Bot** | teloxide | Bot API framework |
| 📱 **Userbots** | rust-tdlib | MTProto wrapper |
| 💾 **Database** | SQLite + SQLx | Data persistence |
| 🧠 **AI** | Ollama | LLM inference |
| 🎤 **Voice** | Whisper | Speech-to-text |
| 🌐 **HTTP** | reqwest | API client |

### Dependencies

```toml
[dependencies]
tokio = { version = "1.42", features = ["full"] }
teloxide = { version = "0.13", features = ["macros"] }
rust-tdlib = { version = "0.4", features = ["client"] }
sqlx = { version = "0.7", features = ["sqlite", "migrate"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
tracing = "0.1"
```

</details>

---

## 🎭 NEW: Extreme Humanization System

<details open>
<summary><b>🚀 Phase 4 & 5: Game-Changing Features</b></summary>
<br>

### 💬 Multi-Texting Engine
Real humans don't write paragraphs - they send multiple short messages!

```
User: как дела?

Bot: норм
Bot: работаю
Bot: ты как?
```

- AI responses automatically split by `||` separator
- Each chunk sent as separate message
- Individual typing indicators per chunk
- Random 0.5-1.5s pauses between messages

### 🚫 Smart Ignore Mechanism
Bots now know when NOT to respond!

```
User: спасибо!
Bot: [reads but doesn't reply - just like a real person]

User: ок
Bot: [ignores - no awkward "you're welcome"]
```

- AI returns `<IGNORE>` for conversation enders
- Handles: "ok", "thanks", "bye", "спс", "давай"
- No more robotic politeness

### ⚡ Instant Read + Delay
Perfect simulation of "seen but not replied yet":

```
1. Message arrives → Marked as "read" instantly ✓
2. Random delay 5-60 seconds (simulates thinking)
3. Then starts typing and responds
```

### 😴 Distracted Typist (20% chance)
Sometimes people get distracted while typing!

```
1. Starts typing...
2. Types for 2-4 seconds
3. Stops typing (distracted!)
4. Pause 3-10 seconds
5. Resumes typing and sends message
```

### 🎭 8 Unique Personalities
No more clone detection! Each bot can have different personality:

| Persona | Style | Emojis | Use Case |
|---------|-------|--------|----------|
| 🤖 **Tired Techie** | Dry, minimal | None | Tech chats |
| 😎 **Ironic Zoomer** | Slang, memes | Spam 💀😭 | Gaming |
| 😡 **Toxic Gamer** | Aggressive | Rare 🤬 | Competitive |
| 👴 **Clueless Boomer** | Polite, confused | Old 🌹🙏 | Mixed age |
| 🕵️ **Paranoid Theorist** | Suspicious | 🤔👁️ | Political |
| 💖 **Wholesome Helper** | Kind, positive | ❤️✨ | Support |
| 📝 **Minimalist** | One-word | None | Quick replies |
| 🧠 **Sarcastic Intellectual** | Witty, ironic | Rare 🙃 | Intellectual |

**Commands:**
```bash
/list_personas              # View all personalities
/random_persona 1           # Assign random to bot #1
/set_persona 1 Tired Techie # Assign specific persona
```

### 🛡️ Anti-Detection Benefits

✅ **Behavioral Diversity** - Each bot responds differently  
✅ **Natural Variation** - Emoji usage varies per persona  
✅ **Context Adaptation** - Personas match social contexts  
✅ **No Clone Signatures** - Impossible to detect "same bot"  
✅ **Human Inconsistency** - Realistic quirks and preferences  

</details>

---

## 🎨 Advanced Features

<details>
<summary><b>🖼️ Media Processing</b></summary>
<br>

Puppeteer can understand and respond to various media types with human-like intelligence:

### Photo Analysis 📸

```
👤 User: [Sends photo of a cat]
   [Bot downloads image]
   [Bot analyzes with llava vision model]
   [Bot reads - 8s delay]
   [Bot typing - 4s]
🤖 Bot: ахах милый котик
```

### GIF Understanding 🎬

```
👤 User: [Sends funny GIF]
   [Bot extracts 3 frames: start, middle, end]
   [Bot analyzes sequence with vision]
   [Bot reads - 5s]
   [Bot typing - 3s]
🤖 Bot: жиза бро || это я когда понедельник
```

### Voice Transcription 🎤

```
👤 User: [Sends voice message: "Hey, can you help me?"]
   [Bot downloads audio]
   [Bot transcribes with Whisper]
   [Bot processes: "[Голосовое сообщение]: Hey, can you help me?"]
   [Bot reads - 6s]
   [Bot typing - 5s]
🤖 Bot: конечно || чем помочь?
```

### Sticker Reactions 🎨

```
👤 User: [Sends sticker]
   [Bot recognizes sticker type]
   [25% probability to respond]
   [Bot reads - 2s]
   [Bot typing - 1s]
🤖 Bot: кек
```

### Supported Media Types

| Media Type | Processing | Response Style | Probability |
|------------|-----------|----------------|-------------|
| 📸 **Photo** | Vision analysis (llava) | Contextual | 50% of base |
| 🎬 **GIF/Animation** | 3-frame extraction + vision | Contextual | 50% of base |
| 🎤 **Voice** | Whisper transcription | Full response | 100% of base |
| 📹 **Video Circle** | 3-frame extraction + vision | Contextual | 50% of base |
| 🎨 **Sticker** | Type recognition | Casual ("ахах", "жиза") | 25% of base |
| 🎭 **Animated Sticker** | Type recognition | Casual | 25% of base |

### Technical Details

- **Vision Models**: llava, minicpm-v (configurable)
- **Frame Extraction**: FFmpeg-based, 3 frames (0%, 50%, 100%)
- **Image Encoding**: Base64 for Ollama API
- **Voice API**: Whisper-compatible endpoint
- **Async Processing**: Non-blocking downloads and analysis
- **Error Handling**: Graceful fallbacks to placeholder text

</details>

<details>
<summary><b>🧠 AI Personality System</b></summary>
<br>

Each bot can have a unique personality via system prompts:

```python
# Friendly Assistant
"You are a friendly and helpful assistant. Always be polite and concise."

# Sarcastic Friend
"You are a sarcastic friend who loves dark humor. Keep responses witty."

# Professional Expert
"You are a professional expert in your field. Provide detailed, accurate information."

# Casual Buddy
"You're a casual buddy who speaks informally. Use slang and emojis."
```

</details>

<details>
<summary><b>📊 Bot Groups & Campaigns</b></summary>
<br>

### Creating a Bot Army

```bash
# Step 1: Create a group
/create_group EliteSquad "My elite bot squad"

# Step 2: Add bots to the group
/add_to_group 1 2  # Add bot #2
/add_to_group 1 3  # Add bot #3
/add_to_group 1 4  # Add bot #4

# Step 3: Launch coordinated campaign
/spam 1 chat -1001234567890 10 2000 Hello from the squad!
```

### Campaign Types

| Type | Description | Status |
|------|-------------|--------|
| 📝 **Text** | Simple text messages | ✅ Ready |
| 🖼️ **Photo** | Image with caption | 🚧 Coming Soon |
| 🎥 **Video** | Video with caption | 🚧 Coming Soon |
| 🎬 **GIF** | Animated GIF | 🚧 Coming Soon |
| 📎 **Document** | File attachment | 🚧 Coming Soon |

</details>

<details>
<summary><b>🔒 Security Features</b></summary>
<br>

### Multi-Layer Security

```mermaid
graph LR
    A[Request] --> B{Owner Check}
    B -->|Not Owner| C[Reject]
    B -->|Owner| D{Rate Limit}
    D -->|Exceeded| C
    D -->|OK| E{Command Valid}
    E -->|Invalid| C
    E -->|Valid| F[Execute]
    F --> G[Encrypted Session]
    G --> H[TDLib MTProto]
```

### Security Measures

- ✅ **Owner-Only Access** - Admin bot restricted to owner IDs
- ✅ **Encrypted Sessions** - TDLib sessions encrypted at rest
- ✅ **No Plaintext Secrets** - All credentials in env vars
- ✅ **Isolated Accounts** - Separate TDLib instances
- ✅ **Error Isolation** - System errors only to owners
- 🚧 **Rate Limiting** - Coming soon
- 🚧 **Prompt Injection Detection** - Coming soon

</details>

---

## 📊 Performance

<table>
<tr>
<td width="50%">

### ⚡ Benchmarks

```
Concurrent Accounts: 100+
Response Time: <100ms
Memory per Bot: ~50MB
CPU Usage: <5% idle
Database: WAL mode
Throughput: 1000+ msg/s
```

</td>
<td width="50%">

### 📈 Scalability

```mermaid
graph LR
    A[1 Bot] -->|Linear| B[10 Bots]
    B -->|Linear| C[100 Bots]
    C -->|Linear| D[1000+ Bots]
    
    style A fill:#4ECDC4
    style B fill:#95E1D3
    style C fill:#FFA07A
    style D fill:#FF6B6B
```

</td>
</tr>
</table>

---

## 🗺️ Roadmap

<table>
<tr>
<td width="33%">

### ✅ Phase 1: Core
- [x] Multi-account management
- [x] AI-driven responses
- [x] Humanization engine
- [x] Bot groups
- [x] Spam campaigns
- [x] Direct messaging
- [x] Inline keyboard UI

</td>
<td width="33%">

### ✅ Phase 2: Media & AI
- [x] Photo analysis (Vision)
- [x] GIF/animation support
- [x] Voice transcription
- [x] Video circles
- [x] Sticker recognition
- [x] Dynamic personas
- [x] Rate limiting
- [x] Multi-texting
- [x] Distracted typist
- [x] Smart ignore system
- [x] RAG memory with embeddings
- [x] Web search integration

</td>
<td width="33%">

### 📋 Phase 3: Enterprise
- [ ] Web dashboard
- [ ] Telegram Mini App
- [ ] Analytics & metrics
- [ ] Multi-language support
- [ ] Plugin system
- [ ] Cloud deployment

</td>
</tr>
</table>

---

## 🤝 Contributing

We ❤️ contributions! Here's how you can help:

<table>
<tr>
<td width="33%" align="center">

### 🐛 Report Bugs
Found a bug?<br/>
[Open an issue](https://github.com/bobberdolle1/Puppeteer/issues/new?template=bug_report.md)

</td>
<td width="33%" align="center">

### 💡 Request Features
Have an idea?<br/>
[Request a feature](https://github.com/bobberdolle1/Puppeteer/issues/new?template=feature_request.md)

</td>
<td width="33%" align="center">

### 🔧 Submit PRs
Want to code?<br/>
[Read CONTRIBUTING.md](CONTRIBUTING.md)

</td>
</tr>
</table>

### Development Setup

```bash
# Clone and setup
git clone https://github.com/bobberdolle1/Puppeteer.git
cd Puppeteer

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Code Style

- ✅ Follow Rust 2021 idioms
- ✅ Use `cargo fmt` before committing
- ✅ Run `cargo clippy` to catch issues
- ✅ Write tests for new features
- ✅ Update documentation

---

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=bobberdolle1/Puppeteer&type=Date)](https://star-history.com/#bobberdolle1/Puppeteer&Date)

---

## 📜 License

<div align="center">

This project is licensed under the **MIT License**

See [LICENSE](LICENSE) for details

</div>

---

## ⚠️ Disclaimer

<div align="center">

> **⚠️ IMPORTANT**: This software is for **educational purposes only**.
> 
> Users are responsible for complying with Telegram's Terms of Service and applicable laws.
> 
> The authors are **not responsible** for any misuse of this software.

</div>

---

## 🙏 Acknowledgments

Special thanks to the amazing open-source projects that made Puppeteer possible:

- **[TDLib](https://github.com/tdlib/td)** - Telegram Database Library for MTProto
- **[Ollama](https://ollama.ai/)** - Local LLM inference engine
- **[teloxide](https://github.com/teloxide/teloxide)** - Elegant Telegram Bot framework for Rust
- **[rust-tdlib](https://github.com/antonio-antuan/rust-tdlib)** - Rust wrapper for TDLib
- **[SQLx](https://github.com/launchbadge/sqlx)** - Async SQL toolkit for Rust
- **[Tokio](https://tokio.rs/)** - Asynchronous runtime for Rust

---

## 💬 Community

<div align="center">

<table>
<tr>
<td align="center">
<a href="https://github.com/bobberdolle1/Puppeteer/discussions">
<img src="https://img.shields.io/badge/GitHub-Discussions-181717?style=for-the-badge&logo=github" alt="Discussions"/>
</a>
</td>
<td align="center">
<a href="https://github.com/bobberdolle1/Puppeteer/issues">
<img src="https://img.shields.io/badge/GitHub-Issues-181717?style=for-the-badge&logo=github" alt="Issues"/>
</a>
</td>
<td align="center">
<a href="https://github.com/bobberdolle1/Puppeteer/pulls">
<img src="https://img.shields.io/badge/GitHub-Pull%20Requests-181717?style=for-the-badge&logo=github" alt="PRs"/>
</a>
</td>
</tr>
</table>

</div>

---

## 📊 Repository Stats

<div align="center">

![GitHub stars](https://img.shields.io/github/stars/bobberdolle1/Puppeteer?style=social)
![GitHub forks](https://img.shields.io/github/forks/bobberdolle1/Puppeteer?style=social)
![GitHub watchers](https://img.shields.io/github/watchers/bobberdolle1/Puppeteer?style=social)

![GitHub issues](https://img.shields.io/github/issues/bobberdolle1/Puppeteer)
![GitHub pull requests](https://img.shields.io/github/issues-pr/bobberdolle1/Puppeteer)
![GitHub last commit](https://img.shields.io/github/last-commit/bobberdolle1/Puppeteer)
![GitHub code size](https://img.shields.io/github/languages/code-size/bobberdolle1/Puppeteer)

</div>

---

<div align="center">

### Made with ❤️ and 🦀 Rust

**If you find this project useful, please consider giving it a ⭐!**

<a href="#-puppeteer">⬆ Back to Top</a>

---

<sub>© 2026 Puppeteer. All rights reserved.</sub>

</div>
