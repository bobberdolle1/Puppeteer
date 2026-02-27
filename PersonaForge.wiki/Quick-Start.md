# 🚀 Быстрый старт

Запусти Puppeteer за 5 минут!

## Вариант 1: Docker (Рекомендуется)

### 1. Клонируй и настрой

```bash
git clone https://github.com/bobberdolle1/Puppeteer.git
cd Puppeteer
cp .env.example .env
```

### 2. Получи учетные данные

**Telegram Bot Token** (для admin-бота):
1. Открой [@BotFather](https://t.me/BotFather)
2. `/newbot` → введи имя → введи username
3. Скопируй токен

**Твой User ID**:
1. Открой [@userinfobot](https://t.me/userinfobot)
2. Скопируй свой ID

**Telegram API** (для userbots):
1. Открой https://my.telegram.org/apps
2. Создай приложение
3. Скопируй `api_id` и `api_hash`

### 3. Заполни .env

```env
BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
OWNER_IDS=987654321
TELEGRAM_API_ID=12345678
TELEGRAM_API_HASH=abcdef1234567890abcdef1234567890
DATABASE_URL=sqlite:data/puppeteer.db
OLLAMA_URL=http://host.docker.internal:11434
```

### 4. Запусти Ollama

```bash
ollama serve
ollama pull llama2
```

### 5. Запусти Puppeteer

```bash
docker-compose up --build
```

### 6. Добавь первый userbot

1. Открой своего admin-бота в Telegram
2. Отправь `/add_account`
3. Следуй инструкциям:
   - Введи номер телефона (например, +1234567890)
   - Введи код подтверждения
   - Введи 2FA пароль (если включен)
4. Userbot запустится автоматически! 🎉

## Вариант 2: Ручная сборка

### 1. Установи зависимости

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y build-essential cmake gperf libssl-dev zlib1g-dev
```

**macOS:**
```bash
brew install cmake openssl
```

### 2. Установи Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Клонируй и собери

```bash
git clone https://github.com/bobberdolle1/Puppeteer.git
cd Puppeteer
cp .env.example .env
# Отредактируй .env
cargo build --release
./target/release/puppeteer
```

## Команды admin-бота

```
/add_account              # Добавить новый userbot
/list_accounts            # Список всех аккаунтов
/start_account <id>       # Запустить userbot
/stop_account <id>        # Остановить userbot
/set_prompt <id>          # Изменить системный промпт
/set_probability <id> <0-100>  # Установить вероятность ответа
/whitelist_chat <id> <chat_id> # Разрешить userbot в чате
/status                   # Статус системы
/help                     # Помощь
```

## Настройка

### Вероятность ответа

Контролируй как часто userbot отвечает (0-100%):
```
/set_probability 1 75  # 75% шанс ответить
```

### Системный промпт

Кастомизируй AI личность:
```
/set_prompt 1
# Затем отправь свой промпт
```

### Whitelist чатов

Ограничь userbot определенными чатами:
```
/whitelist_chat 1 -1001234567890
```

## Troubleshooting

### "Failed to connect to Ollama"
- Убедись что Ollama запущен: `ollama serve`
- Проверь `OLLAMA_URL` в `.env`
- Для Docker: используй `http://host.docker.internal:11434`

### "Invalid phone format"
- Используй международный формат: `+1234567890`
- Включи код страны с `+`

### "Account already exists"
- Каждый номер можно добавить только один раз
- Используй `/list_accounts` чтобы увидеть существующие

### Docker: "library 'tdjson' not found"
- Это ожидаемо при локальной сборке
- Используй Docker для правильной установки TDLib

---

➡️ Далее: [[Commands|Команды]]
