# Quick Start Guide

Get Puppeteer up and running in 5 minutes.

## Prerequisites

- Docker & Docker Compose (recommended)
- OR: Rust 1.70+, TDLib dependencies, and Ollama

## Option 1: Docker (Recommended)

### 1. Clone and Configure

```bash
git clone https://github.com/yourusername/puppeteer.git
cd puppeteer
cp .env.example .env
```

### 2. Edit `.env` File

```bash
# Required: Get from @BotFather
BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz

# Required: Your Telegram user ID (get from @userinfobot)
OWNER_IDS=123456789

# Required: Get from https://my.telegram.org/apps
TELEGRAM_API_ID=12345678
TELEGRAM_API_HASH=abcdef1234567890abcdef1234567890

# Optional: Adjust if Ollama is elsewhere
OLLAMA_URL=http://host.docker.internal:11434
```

### 3. Start Ollama (if not running)

```bash
# macOS/Linux
ollama serve

# Pull a model
ollama pull llama2
```

### 4. Launch Puppeteer

```bash
docker-compose up --build
```

### 5. Add Your First Userbot

1. Open Telegram and find your bot
2. Send `/add_account`
3. Follow the prompts:
   - Enter phone number (e.g., +1234567890)
   - Enter verification code
   - Enter 2FA password (if enabled)
4. Wait for confirmation

Your userbot is now running! 🎉

## Option 2: Manual Build

### 1. Install Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y build-essential cmake gperf libssl-dev zlib1g-dev
```

**macOS:**
```bash
brew install cmake openssl
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Clone and Configure

```bash
git clone https://github.com/yourusername/puppeteer.git
cd puppeteer
cp .env.example .env
# Edit .env with your credentials
```

### 4. Build and Run

```bash
cargo build --release
./target/release/puppeteer
```

## Admin Commands

Once running, use these commands in Telegram:

- `/add_account` - Add a new userbot account
- `/list_accounts` - List all registered accounts
- `/start_account <id>` - Start a specific userbot
- `/stop_account <id>` - Stop a specific userbot
- `/set_prompt <id>` - Update system prompt
- `/set_probability <id> <0-100>` - Set reply probability
- `/whitelist_chat <id> <chat_id>` - Allow userbot in a chat
- `/status` - Show system status
- `/help` - Display all commands

## Configuration Tips

### Reply Probability

Control how often userbots respond (0-100%):
```bash
/set_probability 1 75  # 75% chance to reply
```

### System Prompt

Customize AI personality:
```bash
/set_prompt 1
# Then send your custom prompt
```

### Chat Whitelisting

Restrict userbots to specific chats:
```bash
/whitelist_chat 1 -1001234567890
```

## Troubleshooting

### "Failed to connect to Ollama"
- Ensure Ollama is running: `ollama serve`
- Check `OLLAMA_URL` in `.env`
- For Docker: Use `http://host.docker.internal:11434`

### "Invalid phone format"
- Use international format: `+1234567890`
- Include country code with `+`

### "Account already exists"
- Each phone can only be added once
- Use `/list_accounts` to see existing accounts

### "Failed to start userbot"
- Check logs: `docker-compose logs -f`
- Verify TDLib session files in `data/tdlib/`
- Ensure database is writable

### Docker: "library 'tdjson' not found"
- This is expected during local builds
- Use Docker for proper TDLib installation
- Or manually install TDLib system-wide

## Next Steps

- Read [README.md](README.md) for full documentation
- Check [CONTRIBUTING.md](CONTRIBUTING.md) to contribute
- Review [SECURITY.md](SECURITY.md) for security best practices
- Explore the [wiki](wiki/) for advanced features

## Support

- GitHub Issues: Report bugs and request features
- Discussions: Ask questions and share ideas

---

**Happy automating! 🎭**
