use crate::state::{AppState, WizardState};
use crate::db;
use teloxide::prelude::*;
use teloxide::types::{CallbackQueryId, ParseMode, InlineKeyboardButton, InlineKeyboardMarkup};

pub async fn handle_callback_query(bot: Bot, q: CallbackQuery, state: AppState) -> ResponseResult<()> {
    let Some(message) = &q.message else {
        bot.answer_callback_query(q.id.clone())
            .text("❌ Не удалось получить информацию о чате.")
            .await?;
        return Ok(());
    };

    let chat_id = message.chat().id;
    
    // Check if the user is the owner
    if q.from.id.0 != state.config.owner_id {
        bot.answer_callback_query(q.id.clone())
            .text("❌ У вас нет прав для выполнения этой команды.")
            .await?;
        return Ok(());
    }

    let callback_data = q.data.as_deref().unwrap_or("");
    
    match callback_data {
        // Main navigation
        "main_menu" => send_main_menu(bot, &q.id, chat_id).await?,
        "settings_menu" => send_settings_menu(bot, &q.id, chat_id).await?,
        
        // Submenus
        "personas_menu" => show_personas_menu(bot, &q.id, chat_id).await?,
        "model_settings" => show_model_settings_menu(bot, &q.id, chat_id).await?,
        "rag_settings" => show_rag_settings_menu(bot, &q.id, chat_id).await?,
        "chat_settings" => show_chat_settings_menu(bot, &q.id, chat_id).await?,
        "memory_settings" => show_memory_settings_menu(bot, &q.id, chat_id).await?,
        "model_params" => show_model_params_menu(bot, &q.id, chat_id).await?,
        "ghost_menu" => show_ghost_menu(bot, &q.id, chat_id, &state).await?,
        
        // Persona wizards
        "create_persona_wizard" => {
            state.set_wizard_state(chat_id, WizardState::CreatingPersonaName).await;
            bot.send_message(chat_id, "👤 <b>Создание персоны</b>\n\nВведите название:\n\n/cancel для отмены")
                .parse_mode(ParseMode::Html).await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "update_persona_wizard" => {
            state.set_wizard_state(chat_id, WizardState::UpdatingPersonaId).await;
            bot.send_message(chat_id, "✏️ <b>Обновление персоны</b>\n\nВведите ID персоны:\n\n/cancel для отмены")
                .parse_mode(ParseMode::Html).await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "list_personas" => {
            show_personas_list(bot.clone(), chat_id, &state).await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "activate_persona_wizard" => {
            show_personas_list(bot.clone(), chat_id, &state).await?;
            bot.send_message(chat_id, "Введите ID персоны для активации:").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "delete_persona_wizard" => {
            show_personas_list(bot.clone(), chat_id, &state).await?;
            bot.send_message(chat_id, "Введите ID персоны для удаления:").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        
        // Quick actions
        "system_status" => {
            show_system_status(bot.clone(), chat_id, &state).await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "help_info" => {
            super::commands::send_help_message(bot.clone(), chat_id).await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "enable_rag" => {
            let _ = db::toggle_rag_for_chat(&state.db_pool, chat_id.0, true).await;
            bot.send_message(chat_id, "✅ RAG включен.").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "disable_rag" => {
            let _ = db::toggle_rag_for_chat(&state.db_pool, chat_id.0, false).await;
            bot.send_message(chat_id, "✅ RAG отключен.").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "enable_auto_reply" => {
            let _ = db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, true).await;
            bot.send_message(chat_id, "✅ Автоответы включены.").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "disable_auto_reply" => {
            let _ = db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, false).await;
            bot.send_message(chat_id, "✅ Автоответы отключены.").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "reply_to_all" => {
            let _ = db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "all_messages").await;
            bot.send_message(chat_id, "✅ Режим: все сообщения.").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "reply_to_mention" => {
            let _ = db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "mention_only").await;
            bot.send_message(chat_id, "✅ Режим: только упоминания.").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "ghost_on" => {
            state.toggle_ghost_mode(chat_id, true, true).await;
            bot.send_message(chat_id, "👻 Режим призрака включен!\n\nВаши сообщения отправляются от имени бота.\n/ghost off для выхода").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        "ghost_off" => {
            state.toggle_ghost_mode(chat_id, false, false).await;
            bot.send_message(chat_id, "👻 Режим призрака отключен.").await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        
        // Text input prompts
        "set_model" | "set_temperature" | "set_max_tokens" | "set_memory_depth" | "set_cooldown" => {
            let hint = match callback_data {
                "set_model" => "/set_model название",
                "set_temperature" => "/set_temperature 0.0-2.0",
                "set_max_tokens" => "/set_max_tokens число",
                "set_memory_depth" => "/set_memory_depth 1-50",
                "set_cooldown" => "/set_cooldown секунды",
                _ => ""
            };
            bot.send_message(chat_id, format!("Используйте команду: <code>{}</code>", hint))
                .parse_mode(ParseMode::Html).await?;
            bot.answer_callback_query(q.id.clone()).await?;
        }
        
        _ => {
            bot.answer_callback_query(q.id.clone()).text("❌ Неизвестная команда.").await?;
        }
    }

    Ok(())
}


// --- Menu builders ---

pub async fn send_main_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("👤 Персоны", "personas_menu")],
        vec![InlineKeyboardButton::callback("⚙️ Модель", "model_settings")],
        vec![InlineKeyboardButton::callback("🧠 RAG", "rag_settings")],
        vec![InlineKeyboardButton::callback("💬 Чат", "chat_settings")],
        vec![InlineKeyboardButton::callback("👻 Призрак", "ghost_menu")],
        vec![InlineKeyboardButton::callback("📊 Статус", "system_status")],
        vec![InlineKeyboardButton::callback("ℹ️ Помощь", "help_info")],
    ]);

    bot.send_message(chat_id, "🤖 <b>PersonaForge</b>\n\nВыберите раздел:")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

pub async fn send_settings_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🎭 Персона", "personas_menu")],
        vec![InlineKeyboardButton::callback("🧠 Память", "memory_settings")],
        vec![InlineKeyboardButton::callback("⚙️ Модель", "model_params")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main_menu")],
    ]);

    bot.send_message(chat_id, "🔧 <b>Настройки</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

async fn show_personas_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("📋 Список", "list_personas")],
        vec![InlineKeyboardButton::callback("🆕 Создать", "create_persona_wizard")],
        vec![InlineKeyboardButton::callback("✏️ Изменить", "update_persona_wizard")],
        vec![InlineKeyboardButton::callback("✅ Активировать", "activate_persona_wizard")],
        vec![InlineKeyboardButton::callback("🗑️ Удалить", "delete_persona_wizard")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main_menu")],
    ]);

    bot.send_message(chat_id, "👤 <b>Управление персонами</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

async fn show_model_settings_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🏷️ Модель", "set_model")],
        vec![InlineKeyboardButton::callback("🌡️ Температура", "set_temperature")],
        vec![InlineKeyboardButton::callback("🔢 Токены", "set_max_tokens")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main_menu")],
    ]);

    bot.send_message(chat_id, "⚙️ <b>Настройки модели</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

async fn show_rag_settings_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🟢 Включить", "enable_rag")],
        vec![InlineKeyboardButton::callback("🔴 Отключить", "disable_rag")],
        vec![InlineKeyboardButton::callback("🧠 Глубина", "set_memory_depth")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main_menu")],
    ]);

    bot.send_message(chat_id, "🧠 <b>Настройки RAG</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

async fn show_chat_settings_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🟢 Автоответы вкл", "enable_auto_reply")],
        vec![InlineKeyboardButton::callback("🔴 Автоответы выкл", "disable_auto_reply")],
        vec![InlineKeyboardButton::callback("💬 Все сообщения", "reply_to_all")],
        vec![InlineKeyboardButton::callback("👤 Только упоминания", "reply_to_mention")],
        vec![InlineKeyboardButton::callback("⏱️ Cooldown", "set_cooldown")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main_menu")],
    ]);

    bot.send_message(chat_id, "💬 <b>Настройки чата</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

async fn show_memory_settings_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🧠 Глубина памяти", "set_memory_depth")],
        vec![InlineKeyboardButton::callback("🟢 RAG вкл", "enable_rag")],
        vec![InlineKeyboardButton::callback("🔴 RAG выкл", "disable_rag")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "settings_menu")],
    ]);

    bot.send_message(chat_id, "🧠 <b>Настройки памяти</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

async fn show_model_params_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🏷️ Модель", "set_model")],
        vec![InlineKeyboardButton::callback("🌡️ Температура", "set_temperature")],
        vec![InlineKeyboardButton::callback("🔢 Токены", "set_max_tokens")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "settings_menu")],
    ]);

    bot.send_message(chat_id, "⚙️ <b>Параметры модели</b>")
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

async fn show_ghost_menu(bot: Bot, callback_id: &CallbackQueryId, chat_id: ChatId, state: &AppState) -> ResponseResult<()> {
    let is_active = state.is_ghost_mode(chat_id).await;
    let status = if is_active { "🟢 Активен" } else { "🔴 Выключен" };
    
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("👻 Включить", "ghost_on")],
        vec![InlineKeyboardButton::callback("🚫 Выключить", "ghost_off")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main_menu")],
    ]);

    bot.send_message(chat_id, format!("👻 <b>Режим призрака</b>\n\nСтатус: {}\n\nВ этом режиме ваши сообщения отправляются от имени бота.", status))
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(callback_id.clone()).await?;
    Ok(())
}

// --- Helper functions ---

async fn show_personas_list(bot: Bot, chat_id: ChatId, state: &AppState) -> ResponseResult<()> {
    match db::get_all_personas(&state.db_pool).await {
        Ok(personas) if !personas.is_empty() => {
            let mut text = "📋 <b>Персоны:</b>\n\n".to_string();
            for p in personas {
                let status = if p.is_active { "🟢" } else { "⚪" };
                let prompt_preview = if p.prompt.len() > 50 { 
                    format!("{}...", &p.prompt[..50]) 
                } else { 
                    p.prompt.clone() 
                };
                text.push_str(&format!("{} <b>{}</b> (ID: {})\n<i>{}</i>\n\n", status, p.name, p.id, prompt_preview));
            }
            bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
        }
        _ => {
            bot.send_message(chat_id, "📋 Нет созданных персон.").await?;
        }
    }
    Ok(())
}

async fn show_system_status(bot: Bot, chat_id: ChatId, state: &AppState) -> ResponseResult<()> {
    let ollama_ok = state.llm_client.check_health().await.unwrap_or(false);
    let db_ok = db::check_db_health(&state.db_pool).await.unwrap_or(false);
    
    let persona = match db::get_active_persona(&state.db_pool).await {
        Ok(Some(p)) => p.name,
        _ => "Не выбрана".to_string(),
    };

    let ghost = if state.is_ghost_mode(chat_id).await { "🟢" } else { "🔴" };
    let stats = state.queue_stats.lock().await;

    let text = format!(
        r#"📊 <b>Статус</b>

<b>Сервисы:</b>
• Ollama: {}
• БД: {}

<b>Настройки:</b>
• Персона: {}
• Призрак: {}

<b>Очередь:</b>
• Слотов: {}/{}
• Запросов: {} (✅{} ❌{})"#,
        if ollama_ok { "🟢" } else { "🔴" },
        if db_ok { "🟢" } else { "🔴" },
        persona,
        ghost,
        state.llm_semaphore.available_permits(),
        state.config.max_concurrent_llm_requests.unwrap_or(3),
        stats.total_requests,
        stats.successful_requests,
        stats.failed_requests
    );

    bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
    Ok(())
}
