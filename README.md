<div align="center">

<!-- ANIMATED HEADER -->
<img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=6,11,20&height=180&section=header&text=PersonaForge&fontSize=42&fontColor=fff&animation=twinkling&fontAlignY=32&desc=🤖%20AI%20Personas%20•%20🧠%20RAG%20Memory%20•%20🎤%20Voice%20•%20👁️%20Vision&descAlignY=52&descSize=18"/>

<!-- TYPING ANIMATION -->
<a href="https://git.io/typing-svg"><img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&size=22&pause=1000&color=6C63FF&center=true&vCenter=true&multiline=true&repeat=false&width=600&height=80&lines=Telegram+Bot+with+Customizable+AI+Personas;Long-term+Memory+%26+Multimodal+Capabilities" alt="Typing SVG" /></a>

<!-- BADGES ROW 1 -->
<p>
<a href="https://github.com/bobberdolle1/PersonaForge/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/bobberdolle1/PersonaForge/ci.yml?style=for-the-badge&logo=github-actions&logoColor=white&label=CI&color=2ea44f" alt="CI"></a>
<a href="https://github.com/bobberdolle1/PersonaForge/releases"><img src="https://img.shields.io/github/v/release/bobberdolle1/PersonaForge?style=for-the-badge&logo=semantic-release&logoColor=white&color=6C63FF" alt="Release"></a>
<a href="https://github.com/bobberdolle1/PersonaForge/blob/main/LICENSE"><img src="https://img.shields.io/github/license/bobberdolle1/PersonaForge?style=for-the-badge&logo=opensourceinitiative&logoColor=white&color=green" alt="License"></a>
</p>

<!-- BADGES ROW 2 -->
<p>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
<img src="https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white" alt="Telegram">
<img src="https://img.shields.io/badge/Ollama-000000?style=for-the-badge&logo=ollama&logoColor=white" alt="Ollama">
<img src="https://img.shields.io/badge/SQLite-003B57?style=for-the-badge&logo=sqlite&logoColor=white" alt="SQLite">
<img src="https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white" alt="Docker">
</p>

<!-- STATS -->
<p>
<img src="https://img.shields.io/github/stars/bobberdolle1/PersonaForge?style=for-the-badge&logo=github&color=yellow" alt="Stars">
<img src="https://img.shields.io/github/forks/bobberdolle1/PersonaForge?style=for-the-badge&logo=github&color=blue" alt="Forks">
<img src="https://img.shields.io/github/issues/bobberdolle1/PersonaForge?style=for-the-badge&logo=github&color=red" alt="Issues">
<img src="https://img.shields.io/github/last-commit/bobberdolle1/PersonaForge?style=for-the-badge&logo=github&color=purple" alt="Last Commit">
</p>

<!-- QUICK LINKS -->
<p>
<a href="https://github.com/bobberdolle1/PersonaForge/wiki"><img src="https://img.shields.io/badge/📖_Documentation-Wiki-blue?style=for-the-badge" alt="Wiki"></a>
<a href="https://github.com/bobberdolle1/PersonaForge/issues/new?template=bug_report.md"><img src="https://img.shields.io/badge/🐛_Report-Bug-red?style=for-the-badge" alt="Bug"></a>
<a href="https://github.com/bobberdolle1/PersonaForge/issues/new?template=feature_request.md"><img src="https://img.shields.io/badge/✨_Request-Feature-green?style=for-the-badge" alt="Feature"></a>
<a href="https://github.com/bobberdolle1/PersonaForge/discussions"><img src="https://img.shields.io/badge/💬_Join-Discussions-purple?style=for-the-badge" alt="Discussions"></a>
</p>

</div>

<!-- DEMO GIF -->
<div align="center">
<br>
<img src="https://raw.githubusercontent.com/bobberdolle1/PersonaForge/main/.github/assets/demo.gif" width="700" alt="PersonaForge Demo">
<br><br>
</div>

---


<!-- FEATURES SECTION -->
## <img src="https://media.giphy.com/media/iY8CRBdQXODJSCERIr/giphy.gif" width="30"> Features

<table>
<tr>
<td width="50%">

### 🎭 AI Personas
Create unique AI personalities with custom prompts, display names, and trigger keywords. Switch between personas on the fly.

```json
{
  "name": "Philosopher",
  "display_name": "Сократ",
  "triggers": "философия,смысл,почему",
  "prompt": "Ты — Сократ..."
}
```

</td>
<td width="50%">

### 🧠 RAG Memory
Vector-based conversation memory with time-decay weighting. The bot remembers context and uses relevant information.

```
score = similarity × e^(-decay × hours/24)
```

</td>
</tr>
<tr>
<td width="50%">

### 🎤 Voice Messages
Whisper-powered voice transcription. Send voice messages and get intelligent responses through your active persona.

</td>
<td width="50%">

### 👁️ Vision & Media
Analyze images, GIFs (3-frame extraction), and video messages. Multimodal understanding through LLaVA/BakLLaVA.

</td>
</tr>
<tr>
<td width="50%">

### 🌐 Web Search
DuckDuckGo integration for real-time information. No API keys required — privacy-focused search.

</td>
<td width="50%">

### 🛡️ Security
40+ prompt injection patterns detection, strike system, adaptive rate limiting, and automatic blocking.

</td>
</tr>
</table>

---


<!-- QUICK START -->
## <img src="https://media.giphy.com/media/WUlplcMpOCEmTGBtBW/giphy.gif" width="30"> Quick Start

<details>
<summary><b>📋 Prerequisites</b></summary>
<br>

- [Rust](https://rustup.rs/) 1.70+
- [Ollama](https://ollama.ai/) with models
- Telegram Bot Token from [@BotFather](https://t.me/BotFather)

</details>

### ⚡ One-liner Install

```bash
git clone https://github.com/bobberdolle1/PersonaForge.git && cd PersonaForge && cp .env.example .env
```

### 🔧 Configure

```env
TELOXIDE_TOKEN=your_bot_token_here
OWNER_ID=your_telegram_id
DATABASE_URL=sqlite:persona_forge.db
OLLAMA_CHAT_MODEL=llama3.2
```

### 🚀 Run

<table>
<tr>
<td>

**Cargo**
```bash
cargo run --release
```

</td>
<td>

**Docker**
```bash
docker-compose up --build
```

</td>
</tr>
</table>

---


<!-- ARCHITECTURE -->
## <img src="https://media.giphy.com/media/QssGEmpkyEOhBCb7e1/giphy.gif" width="25"> Architecture

```mermaid
graph TB
    subgraph Telegram
        TG[Telegram API]
    end
    
    subgraph PersonaForge
        BOT[🤖 Bot Handlers]
        WEB[🌐 Mini App]
        SEC[🛡️ Security]
        RAG[🧠 RAG Engine]
        DB[(💾 SQLite)]
    end
    
    subgraph External
        OLL[🦙 Ollama]
        WHI[🎤 Whisper]
        DDG[🔍 DuckDuckGo]
    end
    
    TG <--> BOT
    TG <--> WEB
    BOT --> SEC
    BOT --> RAG
    BOT <--> DB
    RAG <--> DB
    BOT <--> OLL
    BOT <--> WHI
    BOT <--> DDG
    WEB <--> DB
    
    style BOT fill:#6C63FF,color:#fff
    style RAG fill:#00D9FF,color:#000
    style SEC fill:#FF6B6B,color:#fff
    style DB fill:#4CAF50,color:#fff
```

<details>
<summary><b>📁 Project Structure</b></summary>

```
src/
├── main.rs              # Entry point, dispatcher setup
├── config.rs            # Environment configuration
├── state.rs             # Shared state (AppState)
├── logging.rs           # Colored logging system
│
├── bot/handlers/
│   ├── commands.rs      # /menu, /status, /create_persona...
│   ├── messages.rs      # Message processing, RAG retrieval
│   └── callbacks.rs     # Inline keyboard handlers
│
├── db/                  # SQLx queries
├── llm/                 # Ollama client
├── security/            # Prompt injection protection
├── voice/               # Whisper integration
├── web/                 # DuckDuckGo search
└── webapp/              # Mini App (Axum + embedded frontend)
```

</details>

---


<!-- COMMANDS -->
## <img src="https://media.giphy.com/media/jSKBmKkvo2dPQQtsR1/giphy.gif" width="25"> Commands

<div align="center">

| Command | Description |
|:--------|:------------|
| `/menu` | 🎛️ Interactive main menu |
| `/status` | 📊 System status (Ollama, DB, queue) |
| `/create_persona name\|prompt` | 🎭 Create new persona |
| `/list_personas` | 📋 List all personas |
| `/activate_persona ID` | ✅ Activate persona |
| `/set_model name` | 🧠 Change LLM model |
| `/set_temperature 0.7` | 🌡️ Set temperature |
| `/triggers word1, word2` | 🎯 Set trigger keywords |
| `/enable_rag` / `/disable_rag` | 🧠 Toggle RAG memory |
| `/block user_id [min]` | 🚫 Block user |
| `/whoami` | 👤 What bot knows about you |

</div>

---


<!-- MINI APP -->
## <img src="https://media.giphy.com/media/ln7z2eWriiQAllfVcn/giphy.gif" width="25"> Mini App

<div align="center">
<table>
<tr>
<td align="center"><b>📊 Status</b><br><sub>Real-time monitoring</sub></td>
<td align="center"><b>🎭 Personas</b><br><sub>Create & manage</sub></td>
<td align="center"><b>💬 Chats</b><br><sub>Settings per chat</sub></td>
<td align="center"><b>🛡️ Security</b><br><sub>Block & monitor</sub></td>
<td align="center"><b>⚙️ Config</b><br><sub>Runtime settings</sub></td>
</tr>
</table>
</div>

<details>
<summary><b>🔧 Setup Mini App</b></summary>

1. **Start HTTPS tunnel:**
```bash
ssh -R 80:localhost:8080 serveo.net
# or: ngrok http 8080
```

2. **Create in @BotFather:**
```
/newapp → Select bot → Name: PersonaForge Panel → URL: https://your-url.com
```

3. **Add menu button:**
```
/setmenubutton → Select bot → web_app → 🎛️ Panel → URL
```

</details>

---


<!-- CONFIGURATION -->
## <img src="https://media.giphy.com/media/VgCDAzcKvsR6OM0uWg/giphy.gif" width="25"> Configuration

<details>
<summary><b>📝 Full .env Example</b></summary>

```env
# ═══════════════════════════════════════════════════════════════
# 🤖 TELEGRAM
# ═══════════════════════════════════════════════════════════════
TELOXIDE_TOKEN=your_bot_token
OWNER_ID=123456789

# ═══════════════════════════════════════════════════════════════
# 💾 DATABASE
# ═══════════════════════════════════════════════════════════════
DATABASE_URL=sqlite:persona_forge.db

# ═══════════════════════════════════════════════════════════════
# 🦙 OLLAMA
# ═══════════════════════════════════════════════════════════════
OLLAMA_URL=http://localhost:11434
OLLAMA_CHAT_MODEL=llama3.2
OLLAMA_EMBEDDING_MODEL=nomic-embed-text
OLLAMA_VISION_MODEL=llava

# ═══════════════════════════════════════════════════════════════
# ⚡ GENERATION
# ═══════════════════════════════════════════════════════════════
TEMPERATURE=0.7
MAX_TOKENS=2048
LLM_TIMEOUT_SECONDS=120

# ═══════════════════════════════════════════════════════════════
# 🎛️ FEATURES
# ═══════════════════════════════════════════════════════════════
VISION_ENABLED=true
VOICE_ENABLED=true
WEB_SEARCH_ENABLED=true

# ═══════════════════════════════════════════════════════════════
# 🎤 WHISPER
# ═══════════════════════════════════════════════════════════════
WHISPER_URL=http://localhost:8080/inference

# ═══════════════════════════════════════════════════════════════
# 🧠 RAG
# ═══════════════════════════════════════════════════════════════
RAG_DECAY_RATE=0.1
SUMMARY_THRESHOLD=50

# ═══════════════════════════════════════════════════════════════
# 📊 QUEUE
# ═══════════════════════════════════════════════════════════════
MAX_CONCURRENT_LLM_REQUESTS=3
QUEUE_TIMEOUT_SECONDS=30

# ═══════════════════════════════════════════════════════════════
# 🌐 WEBAPP
# ═══════════════════════════════════════════════════════════════
WEBAPP_PORT=8080
```

</details>

---


<!-- PERSONAS EXAMPLES -->
## <img src="https://media.giphy.com/media/3oKIPnAiaMCws8nOsE/giphy.gif" width="25"> Persona Examples

<table>
<tr>
<td>

**🧙 Philosopher**
```json
{
  "name": "Сократ",
  "triggers": "философия,смысл",
  "prompt": "Ты — Сократ. Отвечаешь вопросами, подводя к истине."
}
```

</td>
<td>

**🤖 Tech Expert**
```json
{
  "name": "Техник",
  "triggers": "код,баг,ошибка",
  "prompt": "Ты — senior разработчик. Даёшь чёткие ответы с примерами кода."
}
```

</td>
</tr>
<tr>
<td>

**🎬 Character**
```json
{
  "name": "Чувак",
  "triggers": "dude,боулинг",
  "prompt": "Ты — The Dude из 'Большой Лебовски'. Расслабленный философ."
}
```

</td>
<td>

**👋 Friend**
```json
{
  "name": "Бро",
  "triggers": "бро,друг",
  "prompt": "Ты — лучший друг. Поддерживаешь, шутишь, общаешься неформально."
}
```

</td>
</tr>
</table>

---


<!-- TECH STACK -->
## <img src="https://media.giphy.com/media/uhQuegHFqkVYuFMXMQ/giphy.gif" width="25"> Tech Stack

<div align="center">

| Category | Technologies |
|:--------:|:-------------|
| **Language** | ![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white) ![Tokio](https://img.shields.io/badge/Tokio-async-blue?style=flat-square) |
| **Bot** | ![Teloxide](https://img.shields.io/badge/Teloxide-Telegram_Bot-2CA5E0?style=flat-square&logo=telegram) |
| **Web** | ![Axum](https://img.shields.io/badge/Axum-Web_Framework-orange?style=flat-square) |
| **Database** | ![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white) ![SQLx](https://img.shields.io/badge/SQLx-async-green?style=flat-square) |
| **AI** | ![Ollama](https://img.shields.io/badge/Ollama-Local_LLM-black?style=flat-square) ![Whisper](https://img.shields.io/badge/Whisper-Voice-yellow?style=flat-square) |
| **Search** | ![DuckDuckGo](https://img.shields.io/badge/DuckDuckGo-Privacy-DE5833?style=flat-square&logo=duckduckgo&logoColor=white) |
| **Deploy** | ![Docker](https://img.shields.io/badge/Docker-2496ED?style=flat-square&logo=docker&logoColor=white) ![GitHub Actions](https://img.shields.io/badge/GitHub_Actions-2088FF?style=flat-square&logo=github-actions&logoColor=white) |

</div>

---


<!-- LOGGING -->
## <img src="https://media.giphy.com/media/KzJkzjggfGN5Py6nkT/giphy.gif" width="25"> Beautiful Logging

```
╔══════════════════════════════════════════════════════════════╗
║   ██████╗ ███████╗██████╗ ███████╗ ██████╗ ███╗   ██╗ █████╗ ║
║   ██╔══██╗██╔════╝██╔══██╗██╔════╝██╔═══██╗████╗  ██║██╔══██╗║
║   ██████╔╝█████╗  ██████╔╝███████╗██║   ██║██╔██╗ ██║███████║║
║   ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║██║   ██║██║╚██╗██║██╔══██║║
║   ██║     ███████╗██║  ██║███████║╚██████╔╝██║ ╚████║██║  ██║║
║   ╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝║
║              🤖 F O R G E   v1.0.0                           ║
╚══════════════════════════════════════════════════════════════╝

┌─ Configuration ─────────────────────────────────────────────┐
│  🤖  Bot Name    │ PersonaForge                             │
│  🧠  LLM Model   │ llama3.2                                 │
│  ✓   Vision      │ Enabled                                  │
└──────────────────────────────────────────────────────────────┘

✓  Database connected: sqlite:persona_forge.db
✓  Bot identity: MyBot (@my_bot)
✓  WebApp listening on port 8080

🚀  PersonaForge is ready and listening!

12:34:56 INF [messages] 💬 User in -123456: "Привет!"
12:34:57 INF [llm] 🧠 Response in 1234ms (156 chars)
```

---


<!-- CONTRIBUTING -->
## <img src="https://media.giphy.com/media/du3J3cXyzhj75IOgvA/giphy.gif" width="25"> Contributing

<div align="center">

Contributions are welcome! 🎉

[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=for-the-badge)](https://github.com/bobberdolle1/PersonaForge/pulls)

</div>

1. Fork the repository
2. Create your branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'feat: add amazing feature'`
4. Push: `git push origin feature/amazing-feature`
5. Open a Pull Request

<details>
<summary><b>📋 Before submitting</b></summary>

```bash
cargo fmt          # Format code
cargo clippy       # Lint
cargo test         # Run tests
cargo audit        # Security check
```

</details>

---


<!-- FOOTER -->
## <img src="https://media.giphy.com/media/LnQjpWaON8nhr21vNW/giphy.gif" width="25"> Support

<div align="center">

If you like this project, please give it a ⭐!

[![Star History Chart](https://api.star-history.com/svg?repos=bobberdolle1/PersonaForge&type=Date)](https://star-history.com/#bobberdolle1/PersonaForge&Date)

</div>

---

<div align="center">

### 📜 License

This project is licensed under the [MIT License](LICENSE)

---

<sub>Made with 🦀 Rust and ❤️</sub>

<img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=6,11,20&height=100&section=footer"/>

</div>
