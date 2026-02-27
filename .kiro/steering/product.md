# Puppeteer 🎭

Multi-account Telegram userbot orchestration system powered by AI for human-like interactions.

## Core Features
- **Multi-Account Management**: Orchestrate unlimited Telegram userbot accounts from a single admin interface
- **AI-Driven Responses**: Powered by Ollama with customizable system prompts per account
- **Humanization Engine**: 
  - Configurable reply probability (0-100%)
  - Read delays based on message length
  - Typing indicators with realistic timing
  - Natural conversation flow
- **RAG Memory**: Long-term conversation memory with vector embeddings
- **Voice Transcription**: Automatic voice message transcription via Whisper API
- **Vision Support**: Image analysis through multimodal LLM models
- **Security**: Prompt injection detection, strike system, rate limiting

## Target Users
- Bot owners (multi-admin model via `OWNER_IDS`)
- Automated userbot accounts managed by the system

## Key Behaviors
- Admin bot manages userbot lifecycle (add, start, stop, configure)
- Each userbot operates independently with MTProto
- Responds based on configurable probability and chat whitelist
- Simulates human-like delays and typing patterns
- Maintains separate TDLib sessions per account
