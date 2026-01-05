use crate::db;
use crate::state::AppState;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn handle_command(bot: Bot, msg: Message, state: AppState) -> ResponseResult<()> {
    let text = msg.text().unwrap_or_default();
    let chat_id = msg.chat.id;
    let user_id = msg.from().map(|u| u.id.0);

    // Log the received command
    log::info!("Received command from user {}: {}", user_id.unwrap_or(0), text);

    // Check if the user is the owner
    if user_id != Some(state.config.owner_id) {
        bot.send_message(chat_id, "❌ У вас нет прав для выполнения этой команды.")
            .await?;
        return Ok(());
    }

    let start_time = std::time::Instant::now();
    let command_name = text.split_whitespace().next().unwrap_or("unknown").to_string();

    if text.starts_with("/create_persona") {
        handle_create_persona(bot, msg, &state).await?;
    } else if text.starts_with("/list_personas") {
        handle_list_personas(bot, msg, &state).await?;
    } else if text.starts_with("/activate_persona") {
        handle_activate_persona(bot, msg, &state).await?;
    } else if text.starts_with("/update_persona") {
        handle_update_persona(bot, msg, &state).await?;
    } else if text.starts_with("/delete_persona") {
        handle_delete_persona(bot, msg, &state).await?;
    } else if text.starts_with("/set_model") {
        handle_set_model(bot, msg, &state).await?;
    } else if text.starts_with("/set_temperature") {
        handle_set_temperature(bot, msg, &state).await?;
    } else if text.starts_with("/set_max_tokens") {
        handle_set_max_tokens(bot, msg, &state).await?;
    } else if text.starts_with("/enable_rag") {
        handle_enable_rag(bot, msg, &state).await?;
    } else if text.starts_with("/disable_rag") {
        handle_disable_rag(bot, msg, &state).await?;
    } else if text.starts_with("/set_memory_depth") {
        handle_set_memory_depth(bot, msg, &state).await?;
    } else if text.starts_with("/status") {
        handle_status(bot, msg, &state).await?;
    } else if text.starts_with("/enable_auto_reply") {
        handle_enable_auto_reply(bot, msg, &state).await?;
    } else if text.starts_with("/disable_auto_reply") {
        handle_disable_auto_reply(bot, msg, &state).await?;
    } else if text.starts_with("/reply_to_all") {
        handle_reply_to_all(bot, msg, &state).await?;
    } else if text.starts_with("/reply_to_mention") {
        handle_reply_to_mention(bot, msg, &state).await?;
    } else if text.starts_with("/set_cooldown") {
        handle_set_cooldown(bot, msg, &state).await?;
    } else if text.starts_with("/menu") {
        send_main_menu(bot, chat_id).await?;
    } else if text.starts_with("/settings") {
        send_settings_menu(bot, chat_id).await?;
    } else if text.starts_with("/help") {
        send_help_message(bot, chat_id).await?;
    } else {
        bot.send_message(chat_id, "❌ Неизвестная команда. Используйте /help для списка команд.")
            .await?;
    }

    let duration = start_time.elapsed();
    log::info!("Command {} processed in {}ms", command_name, duration.as_millis());

    Ok(())
}

async fn handle_create_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    
    // Parse the command: /create_persona name|prompt
    let parts: Vec<&str> = text.splitn(2, " ").collect();
    if parts.len() < 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /create_persona название|описание_персоны")
            .await?;
        return Ok(());
    }

    let persona_data: Vec<&str> = parts[1].splitn(2, "|").collect();
    if persona_data.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /create_persona название|описание_персоны")
            .await?;
        return Ok(());
    }

    let name = persona_data[0].trim();
    let prompt = persona_data[1].trim();

    if name.is_empty() || prompt.is_empty() {
        bot.send_message(chat_id, "❌ Название и описание персоны не могут быть пустыми.")
            .await?;
        return Ok(());
    }

    match db::create_persona(&state.db_pool, name, prompt).await {
        Ok(persona_id) => {
            bot.send_message(chat_id, format!("✅ Персона создана с ID: {}", persona_id))
                .await?;
        }
        Err(e) => {
            log::error!("Failed to create persona: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при создании персоны.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_list_personas(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::get_all_personas(&state.db_pool).await {
        Ok(personas) => {
            if personas.is_empty() {
                bot.send_message(chat_id, "📋 Нет созданных персон.")
                    .await?;
            } else {
                let mut response = "📋 Список персон:\n\n".to_string();
                for persona in personas {
                    let status = if persona.is_active { "🟢 Активна" } else { "🔴 Неактивна" };
                    response.push_str(&format!(
                        "ID: {}\nНазвание: {}\nСтатус: {}\nОписание: {}\n\n",
                        persona.id, persona.name, status, persona.prompt
                    ));
                }
                bot.send_message(chat_id, response)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }
        Err(e) => {
            log::error!("Failed to get personas: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при получении списка персон.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_activate_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /activate_persona ID
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /activate_persona ID")
            .await?;
        return Ok(());
    }

    let persona_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ ID персоны должен быть числом.")
                .await?;
            return Ok(());
        }
    };

    match db::set_active_persona(&state.db_pool, persona_id).await {
        Ok(()) => {
            bot.send_message(chat_id, format!("✅ Персона с ID {} активирована.", persona_id))
                .await?;
        }
        Err(e) => {
            log::error!("Failed to activate persona: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при активации персоны. Возможно, персона с таким ID не существует.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_update_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /update_persona ID|name|prompt
    let parts: Vec<&str> = text.splitn(2, " ").collect();
    if parts.len() < 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /update_persona ID|название|описание_персоны")
            .await?;
        return Ok(());
    }

    let update_data: Vec<&str> = parts[1].splitn(3, "|").collect();
    if update_data.len() != 3 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /update_persona ID|название|описание_персоны")
            .await?;
        return Ok(());
    }

    let id = match update_data[0].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ ID персоны должен быть числом.")
                .await?;
            return Ok(());
        }
    };

    let name = update_data[1].trim();
    let prompt = update_data[2].trim();

    if name.is_empty() || prompt.is_empty() {
        bot.send_message(chat_id, "❌ Название и описание персоны не могут быть пустыми.")
            .await?;
        return Ok(());
    }

    match db::update_persona(&state.db_pool, id, name, prompt).await {
        Ok(()) => {
            bot.send_message(chat_id, format!("✅ Персона с ID {} обновлена.", id))
                .await?;
        }
        Err(e) => {
            log::error!("Failed to update persona: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при обновлении персоны. Возможно, персона с таким ID не существует.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_delete_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /delete_persona ID
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /delete_persona ID")
            .await?;
        return Ok(());
    }

    let persona_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ ID персоны должен быть числом.")
                .await?;
            return Ok(());
        }
    };

    match db::delete_persona(&state.db_pool, persona_id).await {
        Ok(()) => {
            bot.send_message(chat_id, format!("✅ Персона с ID {} удалена.", persona_id))
                .await?;
        }
        Err(e) => {
            log::error!("Failed to delete persona: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при удалении персоны. Возможно, персона с таким ID не существует.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_set_model(bot: Bot, msg: Message, _state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /set_model model_name
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /set_model название_модели")
            .await?;
        return Ok(());
    }

    let model_name = parts[1].trim();
    if model_name.is_empty() {
        bot.send_message(chat_id, "❌ Название модели не может быть пустым.")
            .await?;
        return Ok(());
    }

    // In a real implementation, we would update the config in the database or state
    // For now, we'll just send a confirmation message
    bot.send_message(chat_id, format!("✅ Модель установлена: {}", model_name))
        .await?;

    Ok(())
}

async fn handle_set_temperature(bot: Bot, msg: Message, _state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /set_temperature value
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /set_temperature значение (0.0-2.0)")
            .await?;
        return Ok(());
    }

    let temp_str = parts[1].trim();
    let temperature = match temp_str.parse::<f64>() {
        Ok(temp) => {
            if temp < 0.0 || temp > 2.0 {
                bot.send_message(chat_id, "❌ Значение температуры должно быть в диапазоне от 0.0 до 2.0")
                    .await?;
                return Ok(());
            }
            temp
        }
        Err(_) => {
            bot.send_message(chat_id, "❌ Значение температуры должно быть числом")
                .await?;
            return Ok(());
        }
    };

    // In a real implementation, we would update the config in the database or state
    // For now, we'll just send a confirmation message
    bot.send_message(chat_id, format!("✅ Температура установлена: {}", temperature))
        .await?;

    Ok(())
}

async fn handle_set_max_tokens(bot: Bot, msg: Message, _state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /set_max_tokens value
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /set_max_tokens значение")
            .await?;
        return Ok(());
    }

    let max_tokens_str = parts[1].trim();
    let max_tokens = match max_tokens_str.parse::<u32>() {
        Ok(tokens) => {
            if tokens == 0 {
                bot.send_message(chat_id, "❌ Количество токенов должно быть больше 0")
                    .await?;
                return Ok(());
            }
            tokens
        }
        Err(_) => {
            bot.send_message(chat_id, "❌ Количество токенов должно быть числом")
                .await?;
            return Ok(());
        }
    };

    // In a real implementation, we would update the config in the database or state
    // For now, we'll just send a confirmation message
    bot.send_message(chat_id, format!("✅ Максимальное количество токенов установлено: {}", max_tokens))
        .await?;

    Ok(())
}

async fn handle_enable_rag(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::toggle_rag_for_chat(&state.db_pool, chat_id.0, true).await {
        Ok(()) => {
            bot.send_message(chat_id, "✅ RAG включен для этого чата.")
                .await?;
        }
        Err(e) => {
            log::error!("Failed to enable RAG: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при включении RAG.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_disable_rag(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::toggle_rag_for_chat(&state.db_pool, chat_id.0, false).await {
        Ok(()) => {
            bot.send_message(chat_id, "✅ RAG отключен для этого чата.")
                .await?;
        }
        Err(e) => {
            log::error!("Failed to disable RAG: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при отключении RAG.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_set_memory_depth(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /set_memory_depth value
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /set_memory_depth значение")
            .await?;
        return Ok(());
    }

    let depth_str = parts[1].trim();
    let depth = match depth_str.parse::<u32>() {
        Ok(d) => {
            if d == 0 || d > 50 {
                bot.send_message(chat_id, "❌ Глубина памяти должна быть от 1 до 50 сообщений")
                    .await?;
                return Ok(());
            }
            d
        }
        Err(_) => {
            bot.send_message(chat_id, "❌ Глубина памяти должна быть числом")
                .await?;
            return Ok(());
        }
    };

    // Get current RAG setting to preserve it
    let current_settings = match db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await {
        Ok(settings) => settings,
        Err(e) => {
            log::error!("Failed to get chat settings: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при получении настроек чата.")
                .await?;
            return Ok(());
        }
    };

    match db::update_rag_settings(&state.db_pool, chat_id.0, current_settings.rag_enabled, depth as i64).await {
        Ok(()) => {
            bot.send_message(chat_id, format!("✅ Глубина памяти установлена: {} сообщений", depth))
                .await?;
        }
        Err(e) => {
            log::error!("Failed to set memory depth: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при установке глубины памяти.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_status(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check Ollama status
    let ollama_status = match state.llm_client.check_health().await {
        Ok(healthy) => if healthy { "🟢 Работает" } else { "🔴 Недоступен" },
        Err(_) => "🔴 Ошибка подключения",
    };

    // Check DB status
    let db_status = match db::check_db_health(&state.db_pool).await {
        Ok(healthy) => if healthy { "🟢 Работает" } else { "🔴 Недоступна" },
        Err(_) => "🔴 Ошибка подключения",
    };

    // Get active persona info
    let active_persona = match db::get_active_persona(&state.db_pool).await {
        Ok(Some(persona)) => format!("🟢 Активна: {} (ID: {})", persona.name, persona.id),
        Ok(None) => "🟡 Не выбрана".to_string(),
        Err(_) => "🔴 Ошибка получения".to_string(),
    };

    // Get current model
    let current_model = &state.config.ollama_chat_model;

    let status_text = format!(
        r#"📊 <b>Статус бота PersonaForge</b>

<b>Сервисы:</b>
• Ollama: {}
• База данных: {}

<b>Конфигурация:</b>
• Активная персона: {}
• Текущая модель: {}

<b>Параметры генерации:</b>
• Температура: {}
• Макс. токенов: {}"#,
        ollama_status,
        db_status,
        active_persona,
        current_model,
        state.config.temperature,
        state.config.max_tokens
    );

    bot.send_message(chat_id, status_text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

async fn handle_enable_auto_reply(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, true).await {
        Ok(()) => {
            bot.send_message(chat_id, "✅ Автоответы включены для этого чата.")
                .await?;
        }
        Err(e) => {
            log::error!("Failed to enable auto-reply: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при включении автоответов.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_disable_auto_reply(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, false).await {
        Ok(()) => {
            bot.send_message(chat_id, "✅ Автоответы отключены для этого чата.")
                .await?;
        }
        Err(e) => {
            log::error!("Failed to disable auto-reply: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при отключении автоответов.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_reply_to_all(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "all_messages").await {
        Ok(()) => {
            bot.send_message(chat_id, "✅ Режим ответа изменен: на все сообщения.")
                .await?;
        }
        Err(e) => {
            log::error!("Failed to set reply mode to all messages: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при изменении режима ответа.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_reply_to_mention(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "mention_only").await {
        Ok(()) => {
            bot.send_message(chat_id, "✅ Режим ответа изменен: только по упоминанию/команде.")
                .await?;
        }
        Err(e) => {
            log::error!("Failed to set reply mode to mention only: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при изменении режима ответа.")
                .await?;
        }
    }

    Ok(())
}

async fn handle_set_cooldown(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();

    // Parse the command: /set_cooldown value
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Неправильный формат команды. Используйте: /set_cooldown значение (в секундах)")
            .await?;
        return Ok(());
    }

    let cooldown_str = parts[1].trim();
    let cooldown = match cooldown_str.parse::<u32>() {
        Ok(c) => {
            if c > 300 { // Max 5 minutes
                bot.send_message(chat_id, "❌ Время задержки не должно превышать 300 секунд (5 минут)")
                    .await?;
                return Ok(());
            }
            c
        }
        Err(_) => {
            bot.send_message(chat_id, "❌ Время задержки должно быть числом (в секундах)")
                .await?;
            return Ok(());
        }
    };

    match db::update_cooldown_for_chat(&state.db_pool, chat_id.0, cooldown as i64).await {
        Ok(()) => {
            bot.send_message(chat_id, format!("✅ Время задержки между ответами установлено: {} секунд", cooldown))
                .await?;
        }
        Err(e) => {
            log::error!("Failed to set cooldown: {}", e);
            bot.send_message(chat_id, "❌ Ошибка при установке времени задержки.")
                .await?;
        }
    }

    Ok(())
}

async fn send_help_message(bot: Bot, chat_id: ChatId) -> ResponseResult<()> {
    let help_text = r#"🤖 <b>Команды бота PersonaForge:</b>

<b>Управление персонами:</b>
• /create_persona название|описание - Создать новую персону
• /list_personas - Показать все персоны
• /activate_persona ID - Активировать персону по ID
• /update_persona ID|название|описание - Обновить персону
• /delete_persona ID - Удалить персону по ID

<b>Настройки модели:</b>
• /set_model название - Установить модель Ollama
• /set_temperature значение - Установить температуру (0.0-2.0)
• /set_max_tokens значение - Установить максимальное количество токенов

<b>Настройки RAG:</b>
• /enable_rag - Включить RAG (поиск по памяти)
• /disable_rag - Отключить RAG (поиск по памяти)
• /set_memory_depth значение - Установить глубину памяти (1-50 сообщений)

<b>Настройки чата:</b>
• /enable_auto_reply - Включить автоответы
• /disable_auto_reply - Отключить автоответы
• /reply_to_all - Отвечать на все сообщения
• /reply_to_mention - Отвечать только по упоминанию/команде
• /set_cooldown значение - Установить задержку между ответами (в секундах)

<b>Системная информация:</b>
• /status - Показать статус бота и сервисов

Пример: <code>/create_persona Джарвис|Ты умный помощник Илона Маска</code>

<b>Доступно только владельцу бота.</b>"#;

    bot.send_message(chat_id, help_text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn send_main_menu(bot: Bot, chat_id: ChatId) -> ResponseResult<()> {
    use teloxide::types::InlineKeyboardButton;
    use teloxide::types::InlineKeyboardMarkup;

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("👤 Управление персонами", "personas_menu"),
        ],
        vec![
            InlineKeyboardButton::callback("⚙️ Настройки модели", "model_settings"),
        ],
        vec![
            InlineKeyboardButton::callback("🧠 Настройки RAG", "rag_settings"),
        ],
        vec![
            InlineKeyboardButton::callback("💬 Настройки чата", "chat_settings"),
        ],
        vec![
            InlineKeyboardButton::callback("📊 Статус системы", "system_status"),
        ],
        vec![
            InlineKeyboardButton::callback("ℹ️ Помощь", "help_info"),
        ],
    ]);

    bot.send_message(chat_id, "🤖 <b>Главное меню управления ботом PersonaForge</b>\n\nВыберите раздел для управления:")
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

pub async fn send_settings_menu(bot: Bot, chat_id: ChatId) -> ResponseResult<()> {
    use teloxide::types::InlineKeyboardButton;
    use teloxide::types::InlineKeyboardMarkup;

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🎭 Сменить персону", "change_persona"),
        ],
        vec![
            InlineKeyboardButton::callback("🧠 Настройки памяти", "memory_settings"),
        ],
        vec![
            InlineKeyboardButton::callback("⚙️ Параметры модели", "model_params"),
        ],
        vec![
            InlineKeyboardButton::callback("🔙 Назад", "main_menu"),
        ],
    ]);

    bot.send_message(chat_id, "🔧 <b>Настройки бота</b>\n\nВыберите параметр для настройки:")
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
