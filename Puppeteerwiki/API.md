# 🔌 REST API

Mini App использует REST API для управления ботом.

## Аутентификация

Все запросы требуют заголовок:

```
X-Telegram-Init-Data: <initData from Telegram WebApp>
```

Сервер валидирует подпись и проверяет `user_id == OWNER_ID`.

## Endpoints

### Status

```http
GET /api/status
```

Response:
```json
{
  "ollama": {
    "connected": true,
    "model": "llama3.2"
  },
  "database": {
    "connected": true,
    "path": "sqlite:persona_forge.db"
  },
  "queue": {
    "current": 0,
    "max": 3
  },
  "uptime_seconds": 3600,
  "paused": false
}
```

### Personas

#### List

```http
GET /api/personas
```

Response:
```json
[
  {
    "id": 1,
    "name": "Default",
    "prompt": "You are a helpful assistant",
    "display_name": null,
    "triggers": null,
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Create

```http
POST /api/personas
Content-Type: application/json

{
  "name": "Philosopher",
  "prompt": "You are Socrates...",
  "display_name": "Сократ",
  "triggers": "философия,смысл"
}
```

#### Update

```http
PUT /api/personas/:id
Content-Type: application/json

{
  "name": "Updated Name",
  "prompt": "Updated prompt",
  "display_name": "New Display Name",
  "triggers": "new,triggers"
}
```

#### Delete

```http
DELETE /api/personas/:id
```

#### Activate

```http
POST /api/personas/:id/activate?chat_id=123456
```

### Chats

#### List

```http
GET /api/chats
```

Response:
```json
[
  {
    "chat_id": -123456789,
    "title": "My Group",
    "active_persona_id": 1,
    "rag_enabled": true,
    "triggers": "бот,помоги",
    "response_mode": "mention_only"
  }
]
```

#### Update

```http
PUT /api/chats/:id
Content-Type: application/json

{
  "active_persona_id": 2,
  "rag_enabled": false,
  "triggers": "new,triggers",
  "response_mode": "all_messages"
}
```

### Config

#### Get

```http
GET /api/config
```

Response:
```json
{
  "ollama_chat_model": "llama3.2",
  "temperature": 0.7,
  "max_tokens": 2048,
  "vision_enabled": true,
  "voice_enabled": true,
  "web_search_enabled": true
}
```

#### Update

```http
POST /api/config
Content-Type: application/json

{
  "ollama_chat_model": "mistral",
  "temperature": 0.5,
  "max_tokens": 4096,
  "vision_enabled": false
}
```

### Security

#### Get Config

```http
GET /api/security
```

Response:
```json
{
  "injection_detection_enabled": true,
  "rate_limiting_enabled": true,
  "max_strikes": 5
}
```

#### User Status

```http
GET /api/security/users/:id
```

Response:
```json
{
  "user_id": 123456789,
  "strikes": 2,
  "blocked": false,
  "blocked_until": null,
  "last_violation": "2024-01-01T12:00:00Z"
}
```

#### Block User

```http
POST /api/security/users/:id/block
Content-Type: application/json

{
  "duration_minutes": 60
}
```

#### Unblock User

```http
POST /api/security/users/:id/unblock
```

### Pause

#### Get Status

```http
GET /api/pause
```

Response:
```json
{
  "paused": false
}
```

#### Toggle

```http
POST /api/pause
```

Response:
```json
{
  "paused": true
}
```

## Коды ответов

| Код | Описание |
|-----|----------|
| 200 | OK |
| 201 | Created |
| 400 | Bad Request |
| 401 | Unauthorized (invalid initData) |
| 403 | Forbidden (not owner) |
| 404 | Not Found |
| 500 | Internal Server Error |

## Примеры с curl

```bash
# Получить статус
curl -H "X-Telegram-Init-Data: $INIT_DATA" \
  http://localhost:8080/api/status

# Создать персону
curl -X POST \
  -H "X-Telegram-Init-Data: $INIT_DATA" \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","prompt":"Test prompt"}' \
  http://localhost:8080/api/personas

# Заблокировать пользователя
curl -X POST \
  -H "X-Telegram-Init-Data: $INIT_DATA" \
  -H "Content-Type: application/json" \
  -d '{"duration_minutes":60}' \
  http://localhost:8080/api/security/users/123456/block
```

---

➡️ Далее: [[Contributing|Контрибьютинг]]
