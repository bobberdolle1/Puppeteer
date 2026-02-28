# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-28

### 🎨 Phase 2: Advanced Media Processing & Humanization

This release brings Puppeteer to a whole new level with comprehensive media support, extreme humanization, and intelligent behavior patterns.

### Added

#### 🖼️ Media Processing
- **Photo Analysis**: Automatic image analysis using Ollama vision models (llava, minicpm-v)
- **GIF/Animation Support**: Extracts 3 frames (start, middle, end) for intelligent content understanding
- **Voice Transcription**: Automatic voice message transcription via Whisper API
- **Video Circles (Кружки)**: Frame extraction and analysis for video messages
- **Sticker Recognition**: Smart sticker handling with casual responses
- **Animated Stickers**: Full support for animated stickers with humanized reactions

#### 🎭 Extreme Humanization
- **Multi-texting**: Splits responses by `||` separator into multiple messages
- **Distracted Typist**: 20% chance of starting to type, pausing, then continuing (like real humans!)
- **Smart Ignore System**: AI can return `<IGNORE>` to skip meaningless messages
- **Casual Sticker Responses**: Random reactions like "ахах", "жиза", "норм", "кек", "лол", "хд"
- **Lower Media Probability**: Stickers get 1/4 probability, other media 1/2 probability
- **Read Receipts**: Instant message marking as read, then realistic delays
- **Random Read Delays**: 5-60 seconds of "thinking time" before responding

#### 🛡️ Rate Limiting & Security
- **User Rate Limiting**: Automatically ignores users sending >5 messages per minute
- **Timestamp Tracking**: Per-user message history with automatic cleanup
- **Spam Prevention**: Built-in protection against message flooding
- **Prompt Injection Defense**: System prompt includes strict rules to ignore manipulation attempts

#### 🤖 Dynamic Persona System
- **Persona Generator**: Create custom AI personalities on-the-fly
- **Horde Personality Engine**: Pre-built persona templates (Sarcastic, Friendly, Professional, etc.)
- **Per-Account Personas**: Each userbot can have unique personality traits
- **Adaptive Responses**: Context-aware behavior based on conversation flow

#### 🎯 Inline Keyboard UI
- **Modern Admin Interface**: Beautiful inline keyboards instead of plain commands
- **Account Management Panel**: Visual account list with status indicators (🟢/🔴)
- **Quick Actions**: Start/Stop, Edit Prompt, Set Probability, Manage Chats, Delete
- **Interactive Navigation**: Smooth menu system with callback query handling

#### 🔧 Technical Improvements
- **Vision API**: Full multimodal support in Ollama client
- **Frame Extraction**: FFmpeg integration for video/GIF processing
- **Base64 Encoding**: Efficient image data handling
- **Async Media Processing**: Non-blocking media downloads and analysis
- **Send Trait Fixes**: Replaced `thread_rng()` with `rand::random()` for async safety
- **Lazy Static**: Global rate limiting state management

### Changed
- **Message Handling**: Complete rewrite with media type detection
- **Response Generation**: Now supports media context in AI prompts
- **Probability System**: Dynamic adjustment based on message type
- **Typing Indicators**: More realistic timing with variance
- **Reply Logic**: Smart decision-making (only in active dialogues or based on probability)

### Fixed
- **Async Send Issues**: Resolved `Send` trait problems with random number generation
- **Borrow Checker**: Fixed lifetime issues in message processing
- **Thread Safety**: Proper Arc/RwLock usage for shared state
- **Memory Leaks**: Automatic cleanup of old rate limit timestamps

### Technical Details
- Added dependencies: `base64 = "0.21"`, `lazy_static = "1.4"`
- New functions: `process_photo()`, `process_animation()`, `process_voice()`, `process_video_note()`
- Enhanced `OllamaClient` with `vision()` method
- Frame extraction using `ffmpeg` and `ffprobe`
- Rate limiting with `HashMap<user_id, Vec<timestamp>>`

## [Unreleased]

### 🎭 Added - Extreme Humanization System

#### Phase 4: Multi-texting & Smart Ignore (2026-02-28)
- **Multi-Texting Engine**: AI responses automatically split by `||` separator into multiple consecutive messages
  - Each chunk sent as separate message with individual typing indicators
  - Random 0.5-1.5s pauses between chunks for natural flow
  - Simulates real human multi-message behavior
- **Smart Ignore Mechanism**: AI can return `<IGNORE>` to skip replying to conversation enders
  - Handles "ok", "thanks", "bye", "спс", "давай" naturally
  - No more awkward "you're welcome" responses
  - Filters `<IGNORE>` from message chunks automatically
- **Instant Read Receipts**: Messages marked as read immediately via TDLib `ViewMessages`
  - Simulates instant "seen" status
  - Followed by realistic 5-60 second "read delay" before responding
  - Mimics real user behavior of reading but not replying immediately
- **Distracted Typist Behavior**: 20% chance of typing interruption
  - Start typing → 2-4s → Cancel typing → 3-10s pause → Resume typing
  - Simulates real users getting distracted mid-reply
  - Adds unpredictability to typing patterns
- **Extreme Dryness System Prompt**: Zero-emoji, ultra-dry default personality
  - No emojis (only text reactions: ")", "(", "пхпх", "мда")
  - Lowercase writing, minimal punctuation
  - Short, unenthusiastic responses
  - Perfect for technical/professional contexts
- **Media Context Tags**: Passive handling of non-text messages
  - Stickers: `[Пользователь отправил стикер]`
  - GIFs: `[Пользователь отправил GIF]`
  - Photos/Videos: `[Пользователь отправил фото/видео]`
  - Voice messages: `[Пользователь отправил голосовое сообщение]`
  - AI decides how to react based on context

#### Phase 5: Dynamic Persona Generator (2026-02-28)
- **Horde Personality Engine**: 8 unique personality archetypes for diverse bot behavior
  - **Tired Techie**: Dry, exhausted IT worker (no emojis, minimal responses)
  - **Ironic Zoomer**: Post-ironic Gen Z with slang and emoji spam ("база 💀😭")
  - **Toxic Gamer**: Aggressive, easily triggered, uses caps when angry
  - **Clueless Boomer**: 40-50 year old, tech-confused, uses old emojis (🌹, 🙏)
  - **Paranoid Theorist**: Sees conspiracies everywhere, suspicious tone
  - **Wholesome Helper**: Kind, supportive, positive emojis (❤️, ✨)
  - **Minimalist**: Ultra-laconic, one-word answers only
  - **Sarcastic Intellectual**: Smart but sarcastic, witty responses
- **Persona Management Commands**:
  - `/list_personas` - View all available personality archetypes
  - `/random_persona <id>` - Assign random persona to account
  - `/set_persona <id> <name>` - Assign specific persona by name
- **Core Rules System**: Enforced across all personas
  - Multi-texting with `||` separator
  - `<IGNORE>` mechanism for conversation enders
  - No markdown/lists/structured formatting
  - Natural, human-like communication patterns
- **Anti-Detection Benefits**:
  - Each bot has unique response patterns
  - Emoji usage varies realistically per persona
  - No "clone signature" detection possible
  - Natural behavioral diversity in bot hordes
- **Comprehensive Documentation**: Added `wiki/Personas.md` with:
  - Detailed archetype descriptions
  - Use cases and best practices
  - Context matching guidelines
  - Technical implementation details

### 📚 Documentation
- Added comprehensive Persona System documentation (`wiki/Personas.md`)
- Updated `wiki/Home.md` with Personas link
- Updated `wiki/Commands.md` with persona management commands
- Separated userbot personas from legacy PersonaForge commands

### 🔧 Technical Improvements
- Created `src/ai/personas.rs` module with procedural persona generation
- Exported persona functions from `ai` module
- Added persona archetype constants with examples
- Updated `DEFAULT_SYSTEM_PROMPT` with fallback note
- Improved code organization and modularity

### 🎨 User Experience
- Bots now feel genuinely human with realistic delays
- No more overly helpful AI cheerfulness
- Natural ignoring of messages that don't need replies
- Realistic typing interruptions and distractions
- Diverse personalities prevent clone detection

---

## [0.2.0] - 2026-02-28

### Added
- Initial release of Puppeteer
- Multi-account Telegram userbot orchestration system
- Admin bot with teloxide (Bot API)
- MTProto userbots with rust-tdlib
- AI-driven responses via Ollama integration
- Humanization engine with configurable reply probability
- Read delays and typing indicators for natural behavior
- RAG memory system for long-term conversation context
- Voice message transcription via Whisper API
- Vision support for image analysis
- Security features: prompt injection detection, strike system, rate limiting
- SQLite database with WAL mode for high concurrency
- Docker and docker-compose support
- Comprehensive admin commands for account management
- Chat whitelisting system
- Customizable system prompts per account
- Russian casual conversation style default prompt
- GitHub Actions CI/CD pipeline
- Community health files (CODE_OF_CONDUCT, CONTRIBUTING)
- Professional README with architecture diagram

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- Implemented owner-only access control
- Added prompt injection detection
- Implemented strike system for abuse prevention
- Added rate limiting for API calls

## [0.1.0] - 2024-02-28

### Added
- Initial project structure
- Core functionality for multi-account userbot management
- Basic AI integration with Ollama
- Database schema and migrations
- Admin bot command handlers
- TDLib integration for MTProto

---

**Note**: This is the first public release of Puppeteer. Future updates will be documented in this file following the Keep a Changelog format.
