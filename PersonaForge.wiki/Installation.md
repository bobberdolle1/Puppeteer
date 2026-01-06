# 📦 Установка

## Требования

- **Rust** 1.70+ ([rustup.rs](https://rustup.rs/))
- **Ollama** ([ollama.ai](https://ollama.ai/))
- **Telegram Bot Token** ([@BotFather](https://t.me/BotFather))

## Способ 1: Из исходников

```bash
# Клонируй репозиторий
git clone https://github.com/bobberdolle1/PersonaForge.git
cd PersonaForge

# Скопируй конфиг
cp .env.example .env

# Отредактируй .env (см. Конфигурация)
nano .env

# Собери и запусти
cargo run --release
```

## Способ 2: Docker

```bash
# Клонируй репозиторий
git clone https://github.com/bobberdolle1/PersonaForge.git
cd PersonaForge

# Настрой .env
cp .env.example .env
nano .env

# Запусти
docker-compose up --build
```

## Способ 3: Готовый бинарник

Скачай бинарник для своей платформы из [Releases](https://github.com/bobberdolle1/PersonaForge/releases):

- `persona-forge-linux-amd64.tar.gz` — Linux x64
- `persona-forge-linux-arm64.tar.gz` — Linux ARM64
- `persona-forge-macos-amd64.tar.gz` — macOS Intel
- `persona-forge-macos-arm64.tar.gz` — macOS Apple Silicon

```bash
# Распакуй
tar -xzf persona-forge-linux-amd64.tar.gz

# Создай .env
cp .env.example .env
nano .env

# Запусти
./PersonaForge
```

## Установка Ollama

```bash
# Linux/macOS
curl -fsSL https://ollama.ai/install.sh | sh

# Скачай модели
ollama pull llama3.2
ollama pull nomic-embed-text

# Для vision (опционально)
ollama pull llava
```

## Создание Telegram бота

1. Открой [@BotFather](https://t.me/BotFather) в Telegram
2. Отправь `/newbot`
3. Введи имя бота (например: `PersonaForge`)
4. Введи username (например: `my_persona_forge_bot`)
5. Скопируй токен в `.env` → `TELOXIDE_TOKEN`

## Получение OWNER_ID

1. Открой [@userinfobot](https://t.me/userinfobot) в Telegram
2. Отправь любое сообщение
3. Скопируй свой ID в `.env` → `OWNER_ID`

---

➡️ Далее: [[Configuration|Конфигурация]]
