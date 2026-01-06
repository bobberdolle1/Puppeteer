# Changelog

All notable changes to PersonaForge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-01-06

### Added
- 🎭 **Personas System**
  - Create, edit, delete AI personas
  - Custom prompts and display names
  - Trigger keywords for persona activation
  - Export/import personas as JSON

- 🧠 **RAG Memory**
  - Vector-based conversation memory
  - Cosine similarity search
  - Time-decay weighting for relevance
  - Automatic conversation summarization

- 🎤 **Voice Support**
  - Whisper API integration
  - Voice message transcription
  - Responses through active persona

- 👁️ **Vision Capabilities**
  - Image analysis via multimodal LLM
  - GIF animation support (3-frame extraction)
  - Video message (circles) analysis
  - Combined video + audio transcription

- 🌐 **Web Search**
  - DuckDuckGo integration
  - Real-time information retrieval
  - No API key required

- 🛡️ **Security**
  - Prompt injection detection (40+ patterns)
  - Strike system with temporary blocks
  - Adaptive rate limiting
  - Input sanitization

- 📱 **Mini App**
  - Web-based control panel
  - Persona management UI
  - Chat settings configuration
  - Real-time status monitoring
  - Security dashboard

- 💬 **Telegram Features**
  - Forum/topic support
  - Typing indicators
  - Markdown formatting with fallback
  - Mention and trigger detection
  - Reply probability settings

- 🎨 **Logging System**
  - Colored console output
  - Session metrics on shutdown
  - Spam suppression
  - Response time indicators

### Technical
- Rust 2021 edition
- Async runtime with Tokio
- SQLite database with sqlx
- Axum web framework for Mini App
- Embedded static files with rust-embed

---

## Version History

- **1.0.0** - First stable release with full feature set

[Unreleased]: https://github.com/bobberdolle1/PersonaForge/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/bobberdolle1/PersonaForge/releases/tag/v1.0.0
