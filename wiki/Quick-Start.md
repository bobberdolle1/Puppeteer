# 🚀 Быстрый старт

Запусти PersonaForge за 5 минут!

## 1. Установи зависимости

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Модели
ollama pull llama3.2
ollama pull nomic-embed-text
```

## 2. Клонируй и настрой

```bash
git clone https://github.com/bobberdolle1/PersonaForge.git
cd PersonaForge
cp .env.example .env
```

## 3. Получи токен бота

1. Открой [@BotFather](https://t.me/BotFather)
2. `/newbot` → введи имя → введи username
3. Скопируй токен

## 4. Получи свой ID

1. Открой [@userinfobot](https://t.me/userinfobot)
2. Скопируй свой ID

## 5. Заполни .env

```env
TELOXIDE_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
OWNER_ID=987654321
DATABASE_URL=sqlite:persona_forge.db
```

## 6. Запусти!

```bash
cargo run --release
```

## 7. Проверь

1. Открой своего бота в Telegram
2. Отправь `/start`
3. Отправь `/menu`

## Что дальше?

- [[Personas|Создай первую персону]]
- [[Commands|Изучи команды]]
- [[Mini-App|Настрой веб-панель]]

## Быстрые команды

```
/menu              # Главное меню
/status            # Проверить статус
/create_persona Тест|Ты тестовая персона  # Создать персону
/list_personas     # Список персон
```

## Troubleshooting

### Бот не отвечает

1. Проверь что Ollama запущен: `ollama list`
2. Проверь токен в `.env`
3. Проверь логи в консоли

### Ошибка подключения к Ollama

```bash
# Проверь что Ollama работает
curl http://localhost:11434/api/tags

# Если нет — запусти
ollama serve
```

### Ошибка базы данных

```bash
# Удали старую БД и перезапусти
rm persona_forge.db
cargo run --release
```

---

➡️ Далее: [[Commands|Команды]]
