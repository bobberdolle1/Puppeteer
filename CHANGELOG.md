# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
