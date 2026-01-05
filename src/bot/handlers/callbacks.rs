use crate::state::{AppState, WizardState};
use crate::db;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, InlineKeyboardButton, InlineKeyboardMarkup, MessageId};

pub async fn handle_callback_query(bot: Bot, q: CallbackQuery, state: AppState) -> ResponseResult<()> {
    let Some(message) = &q.message else {
        bot.answer_callback_query(q.id.clone())
            .text("❌ Не удалось получить информацию о чате.")
            .await?;
        return Ok(());
    };

    let chat_id = message.chat().id;
    let msg_id = message.id();
    
    // Check if the user is the owner
    if q.from.id.0 != state.config.owner_id {
        bot.answer_callback_query(q.id.clone())
            .text("❌ У вас нет прав.")
            .await?;
        return Ok(());
    }

    let data = q.data.as_deref().unwrap_or("");
    
    // Parse callback data
    let parts: Vec<&str> = data.split(':').collect();
    let action = parts[0];
    let param = parts.get(1).copied();

    match action {
        // === MAIN MENU ===
        "main" => edit_main_menu(&bot, chat_id, msg_id).await?,
        
        // === PERSONAS ===
        "personas" => edit_personas_menu(&bot, chat_id, msg_id).await?,
        "p_list" => show_personas_list_inline(&bot, chat_id, msg_id, &state).await?,
        "p_create" => {
            state.set_wizard_state(chat_id, WizardState::CreatingPersonaName).await;
            bot.edit_message_text(chat_id, msg_id, "👤 <b>Создание персоны</b>\n\nВведите название:\n\n/cancel для отмены")
                .parse_mode(ParseMode::Html).await?;
        }
        "p_activate" => {
            if let Some(id) = param.and_then(|p| p.parse::<i64>().ok()) {
                let _ = db::set_active_persona(&state.db_pool, id).await;
                bot.answer_callback_query(q.id.clone()).text("✅ Персона активирована").await?;
                show_personas_list_inline(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        "p_delete" => {
            if let Some(id) = param.and_then(|p| p.parse::<i64>().ok()) {
                let _ = db::delete_persona(&state.db_pool, id).await;
                bot.answer_callback_query(q.id.clone()).text("✅ Персона удалена").await?;
                show_personas_list_inline(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        "p_edit" => {
            if let Some(id) = param.and_then(|p| p.parse::<i64>().ok()) {
                state.set_wizard_state(chat_id, WizardState::UpdatingPersonaName { id }).await;
                bot.edit_message_text(chat_id, msg_id, format!("✏️ <b>Редактирование персоны ID {}</b>\n\nВведите новое название:\n\n/cancel для отмены", id))
                    .parse_mode(ParseMode::Html).await?;
            }
        }
        "p_view" => {
            if let Some(id) = param.and_then(|p| p.parse::<i64>().ok()) {
                show_persona_detail(&bot, chat_id, msg_id, &state, id).await?;
            }
        }
        "p_export" => {
            if let Some(id) = param.and_then(|p| p.parse::<i64>().ok()) {
                export_persona_inline(&bot, chat_id, &state, id).await?;
                bot.answer_callback_query(q.id.clone()).text("📤 Экспорт отправлен").await?;
                return Ok(());
            }
        }
        "p_export_all" => {
            export_all_personas_inline(&bot, chat_id, &state).await?;
            bot.answer_callback_query(q.id.clone()).text("📤 Экспорт всех персон отправлен").await?;
            return Ok(());
        }
        "p_import" => {
            state.set_wizard_state(chat_id, WizardState::ImportingPersona).await;
            bot.edit_message_text(chat_id, msg_id, "📥 <b>Импорт персоны</b>\n\nОтправьте JSON-файл или текст в формате:\n<code>{\"name\":\"...\",\"prompt\":\"...\"}</code>\n\n/cancel для отмены")
                .parse_mode(ParseMode::Html).await?;
        }
        
        // === CONFIG ===
        "config" => edit_config_menu(&bot, chat_id, msg_id, &state).await?,
        "cfg_model" => edit_model_select(&bot, chat_id, msg_id, &state).await?,
        "cfg_set_model" => {
            if let Some(model) = param {
                let _ = db::set_config(&state.db_pool, "ollama_chat_model", model).await;
                bot.answer_callback_query(q.id.clone()).text(format!("✅ Модель: {}", model)).await?;
                edit_config_menu(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        "cfg_temp" => edit_temperature_menu(&bot, chat_id, msg_id, &state).await?,
        "cfg_set_temp" => {
            if let Some(temp) = param {
                let _ = db::set_config(&state.db_pool, "temperature", temp).await;
                bot.answer_callback_query(q.id.clone()).text(format!("✅ Температура: {}", temp)).await?;
                edit_config_menu(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        "cfg_tokens" => edit_tokens_menu(&bot, chat_id, msg_id, &state).await?,
        "cfg_set_tokens" => {
            if let Some(tokens) = param {
                let _ = db::set_config(&state.db_pool, "max_tokens", tokens).await;
                bot.answer_callback_query(q.id.clone()).text(format!("✅ Токены: {}", tokens)).await?;
                edit_config_menu(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        "cfg_toggle" => {
            if let Some(key) = param {
                let current = db::get_config_bool(&state.db_pool, key, false).await;
                let new_val = if current { "false" } else { "true" };
                let _ = db::set_config(&state.db_pool, key, new_val).await;
                let status = if !current { "включено" } else { "выключено" };
                bot.answer_callback_query(q.id.clone()).text(format!("✅ {} {}", key, status)).await?;
                edit_config_menu(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        
        // === CHAT SETTINGS ===
        "chat" => edit_chat_menu(&bot, chat_id, msg_id, &state).await?,
        "chat_auto_on" => {
            let _ = db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, true).await;
            bot.answer_callback_query(q.id.clone()).text("✅ Автоответы включены").await?;
            edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "chat_auto_off" => {
            let _ = db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, false).await;
            bot.answer_callback_query(q.id.clone()).text("✅ Автоответы выключены").await?;
            edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "chat_mode_all" => {
            let _ = db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "all_messages").await;
            bot.answer_callback_query(q.id.clone()).text("✅ Режим: все сообщения").await?;
            edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "chat_mode_mention" => {
            let _ = db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "mention_only").await;
            bot.answer_callback_query(q.id.clone()).text("✅ Режим: упоминания").await?;
            edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "chat_rag_on" => {
            let _ = db::toggle_rag_for_chat(&state.db_pool, chat_id.0, true).await;
            bot.answer_callback_query(q.id.clone()).text("✅ RAG включен").await?;
            edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "chat_rag_off" => {
            let _ = db::toggle_rag_for_chat(&state.db_pool, chat_id.0, false).await;
            bot.answer_callback_query(q.id.clone()).text("✅ RAG выключен").await?;
            edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "chat_cooldown" => edit_cooldown_menu(&bot, chat_id, msg_id).await?,
        "chat_set_cd" => {
            if let Some(cd) = param.and_then(|p| p.parse::<i64>().ok()) {
                let _ = db::update_cooldown_for_chat(&state.db_pool, chat_id.0, cd).await;
                bot.answer_callback_query(q.id.clone()).text(format!("✅ Cooldown: {}с", cd)).await?;
                edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        "chat_triggers" => {
            state.set_wizard_state(chat_id, WizardState::SettingKeywords).await;
            let current = state.keyword_triggers.lock().await.get(&chat_id).cloned();
            let current_str = current.map(|k| k.join(", ")).unwrap_or_else(|| "не заданы".to_string());
            bot.edit_message_text(chat_id, msg_id, format!("🎯 <b>Триггеры</b>\n\nТекущие: {}\n\nВведите ключевые слова через запятую:\n\n/cancel для отмены", current_str))
                .parse_mode(ParseMode::Html).await?;
        }
        "chat_triggers_clear" => {
            state.keyword_triggers.lock().await.remove(&chat_id);
            bot.answer_callback_query(q.id.clone()).text("✅ Триггеры удалены").await?;
            edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "chat_depth" => edit_memory_depth_menu(&bot, chat_id, msg_id, &state).await?,
        "chat_set_depth" => {
            if let Some(depth) = param.and_then(|p| p.parse::<i64>().ok()) {
                let settings = db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await
                    .unwrap_or(db::ChatSettings { chat_id: chat_id.0, auto_reply_enabled: true, reply_mode: "mention_only".into(), cooldown_seconds: 5, context_depth: 10, rag_enabled: true });
                let _ = db::update_rag_settings(&state.db_pool, chat_id.0, settings.rag_enabled, depth).await;
                bot.answer_callback_query(q.id.clone()).text(format!("✅ Глубина памяти: {}", depth)).await?;
                edit_chat_menu(&bot, chat_id, msg_id, &state).await?;
                return Ok(());
            }
        }
        
        // === GHOST MODE ===
        "ghost" => edit_ghost_menu(&bot, chat_id, msg_id, &state).await?,
        "ghost_on" => {
            state.toggle_ghost_mode(chat_id, true, true).await;
            bot.answer_callback_query(q.id.clone()).text("👻 Ghost Mode включен").await?;
            edit_ghost_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "ghost_on_nosave" => {
            state.toggle_ghost_mode(chat_id, true, false).await;
            bot.answer_callback_query(q.id.clone()).text("👻 Ghost Mode включен (без сохранения)").await?;
            edit_ghost_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        "ghost_off" => {
            state.toggle_ghost_mode(chat_id, false, false).await;
            bot.answer_callback_query(q.id.clone()).text("👻 Ghost Mode выключен").await?;
            edit_ghost_menu(&bot, chat_id, msg_id, &state).await?;
            return Ok(());
        }
        
        // === TOOLS ===
        "tools" => edit_tools_menu(&bot, chat_id, msg_id).await?,
        "tools_broadcast" => {
            state.set_wizard_state(chat_id, WizardState::Broadcasting).await;
            bot.edit_message_text(chat_id, msg_id, "📢 <b>Рассылка</b>\n\nВведите текст сообщения для рассылки по всем чатам:\n\n/cancel для отмены")
                .parse_mode(ParseMode::Html).await?;
        }
        "tools_clear_history" => edit_clear_history_menu(&bot, chat_id, msg_id).await?,
        "tools_clear_confirm" => {
            let _ = db::clear_chat_history(&state.db_pool, chat_id.0).await;
            bot.answer_callback_query(q.id.clone()).text("✅ История очищена").await?;
            edit_tools_menu(&bot, chat_id, msg_id).await?;
            return Ok(());
        }
        "tools_clear_memory" => {
            let _ = db::clear_chat_memory(&state.db_pool, chat_id.0).await;
            bot.answer_callback_query(q.id.clone()).text("✅ RAG память очищена").await?;
            edit_tools_menu(&bot, chat_id, msg_id).await?;
            return Ok(());
        }
        
        // === SECURITY ===
        "security" => edit_security_menu(&bot, chat_id, msg_id, &state).await?,
        "sec_check_user" => {
            bot.edit_message_text(chat_id, msg_id, "🔍 <b>Проверка пользователя</b>\n\nВведите user_id для проверки:\n\n/cancel для отмены")
                .parse_mode(ParseMode::Html).await?;
        }
        
        // === STATUS ===
        "status" => edit_status(&bot, chat_id, msg_id, &state).await?,
        
        // === HELP ===
        "help" => edit_help(&bot, chat_id, msg_id).await?,
        "help_personas" => edit_help_personas(&bot, chat_id, msg_id).await?,
        "help_config" => edit_help_config(&bot, chat_id, msg_id).await?,
        "help_chat" => edit_help_chat(&bot, chat_id, msg_id).await?,
        "help_ghost" => edit_help_ghost(&bot, chat_id, msg_id).await?,
        "help_rag" => edit_help_rag(&bot, chat_id, msg_id).await?,
        "help_commands" => edit_help_commands(&bot, chat_id, msg_id).await?,
        
        _ => {
            bot.answer_callback_query(q.id.clone()).text("❌ Неизвестная команда").await?;
        }
    }

    bot.answer_callback_query(q.id.clone()).await?;
    Ok(())
}


// === MENU BUILDERS ===

async fn edit_main_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🎭 Персоны", "personas"),
            InlineKeyboardButton::callback("⚙️ Конфиг", "config"),
        ],
        vec![
            InlineKeyboardButton::callback("💬 Чат", "chat"),
            InlineKeyboardButton::callback("👻 Ghost", "ghost"),
        ],
        vec![
            InlineKeyboardButton::callback("🛠️ Инструменты", "tools"),
            InlineKeyboardButton::callback("📊 Статус", "status"),
        ],
        vec![InlineKeyboardButton::callback("❓ Помощь", "help")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, "🤖 <b>PersonaForge</b>\n\nВыберите раздел:")
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_personas_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("📋 Список персон", "p_list")],
        vec![InlineKeyboardButton::callback("➕ Создать", "p_create")],
        vec![
            InlineKeyboardButton::callback("📥 Импорт", "p_import"),
            InlineKeyboardButton::callback("📤 Экспорт всех", "p_export_all"),
        ],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, "🎭 <b>Персоны</b>\n\nУправление AI-личностями")
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn show_personas_list_inline(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let personas = db::get_all_personas(&state.db_pool).await.unwrap_or_default();
    
    if personas.is_empty() {
        let kb = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback("➕ Создать", "p_create")],
            vec![InlineKeyboardButton::callback("📥 Импорт", "p_import")],
            vec![InlineKeyboardButton::callback("🔙 Назад", "personas")],
        ]);
        bot.edit_message_text(chat_id, msg_id, "📋 <b>Персоны</b>\n\nСписок пуст")
            .parse_mode(ParseMode::Html)
            .reply_markup(kb)
            .await?;
        return Ok(());
    }
    
    let mut text = "📋 <b>Персоны:</b>\n\n".to_string();
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];
    
    for p in &personas {
        let status = if p.is_active { "🟢" } else { "⚪" };
        let preview = if p.prompt.len() > 50 { format!("{}...", &p.prompt[..50]) } else { p.prompt.clone() };
        text.push_str(&format!("{} <b>{}</b> (ID: {})\n<i>{}</i>\n\n", status, p.name, p.id, preview));
        
        let mut row = vec![];
        if !p.is_active {
            row.push(InlineKeyboardButton::callback("✅", format!("p_activate:{}", p.id)));
        }
        row.push(InlineKeyboardButton::callback("👁️", format!("p_view:{}", p.id)));
        row.push(InlineKeyboardButton::callback("✏️", format!("p_edit:{}", p.id)));
        row.push(InlineKeyboardButton::callback("📤", format!("p_export:{}", p.id)));
        row.push(InlineKeyboardButton::callback("🗑️", format!("p_delete:{}", p.id)));
        buttons.push(row);
    }
    
    buttons.push(vec![InlineKeyboardButton::callback("➕ Создать", "p_create")]);
    buttons.push(vec![InlineKeyboardButton::callback("🔙 Назад", "personas")]);
    
    let kb = InlineKeyboardMarkup::new(buttons);
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn show_persona_detail(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState, id: i64) -> ResponseResult<()> {
    let personas = db::get_all_personas(&state.db_pool).await.unwrap_or_default();
    let persona = personas.iter().find(|p| p.id == id);
    
    match persona {
        Some(p) => {
            let status = if p.is_active { "🟢 Активна" } else { "⚪ Неактивна" };
            let text = format!(
                "🎭 <b>{}</b>\n\n\
                <b>ID:</b> {}\n\
                <b>Статус:</b> {}\n\n\
                <b>Промпт:</b>\n<code>{}</code>",
                p.name, p.id, status, p.prompt
            );
            
            let mut buttons = vec![
                vec![
                    InlineKeyboardButton::callback("✏️ Редактировать", format!("p_edit:{}", id)),
                    InlineKeyboardButton::callback("📤 Экспорт", format!("p_export:{}", id)),
                ],
            ];
            if !p.is_active {
                buttons.push(vec![InlineKeyboardButton::callback("✅ Активировать", format!("p_activate:{}", id))]);
            }
            buttons.push(vec![InlineKeyboardButton::callback("🗑️ Удалить", format!("p_delete:{}", id))]);
            buttons.push(vec![InlineKeyboardButton::callback("🔙 К списку", "p_list")]);
            
            let kb = InlineKeyboardMarkup::new(buttons);
            bot.edit_message_text(chat_id, msg_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await?;
        }
        None => {
            bot.edit_message_text(chat_id, msg_id, "❌ Персона не найдена")
                .reply_markup(InlineKeyboardMarkup::new(vec![
                    vec![InlineKeyboardButton::callback("🔙 К списку", "p_list")]
                ]))
                .await?;
        }
    }
    Ok(())
}

async fn export_persona_inline(bot: &Bot, chat_id: ChatId, state: &AppState, id: i64) -> ResponseResult<()> {
    if let Ok(Some(json)) = db::export_persona(&state.db_pool, id).await {
        let filename = format!("persona_{}.json", id);
        let doc = teloxide::types::InputFile::memory(json.into_bytes()).file_name(filename);
        bot.send_document(chat_id, doc).caption("📤 Экспорт персоны").await?;
    }
    Ok(())
}

async fn export_all_personas_inline(bot: &Bot, chat_id: ChatId, state: &AppState) -> ResponseResult<()> {
    if let Ok(json) = db::export_all_personas(&state.db_pool).await {
        let doc = teloxide::types::InputFile::memory(json.into_bytes()).file_name("personas_export.json");
        bot.send_document(chat_id, doc).caption("📤 Экспорт всех персон").await?;
    }
    Ok(())
}

async fn edit_config_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let model = db::get_config(&state.db_pool, "ollama_chat_model").await.ok().flatten()
        .unwrap_or_else(|| state.config.ollama_chat_model.clone());
    let temp = db::get_config_f64(&state.db_pool, "temperature", state.config.temperature).await;
    let tokens = db::get_config_u32(&state.db_pool, "max_tokens", state.config.max_tokens).await;
    let vision = db::get_config_bool(&state.db_pool, "vision_enabled", state.config.vision_enabled).await;
    let voice = db::get_config_bool(&state.db_pool, "voice_enabled", state.config.voice_enabled).await;
    let web = db::get_config_bool(&state.db_pool, "web_search_enabled", state.config.web_search_enabled).await;
    
    let text = format!(
        "⚙️ <b>Конфигурация</b>\n\n\
        🤖 Модель: <code>{}</code>\n\
        🌡️ Температура: <code>{}</code>\n\
        📝 Токены: <code>{}</code>\n\n\
        👁️ Vision: {}\n\
        🎤 Voice: {}\n\
        🌐 Web: {}",
        model, temp, tokens,
        if vision { "✅" } else { "❌" },
        if voice { "✅" } else { "❌" },
        if web { "✅" } else { "❌" }
    );
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🤖 Модель", "cfg_model")],
        vec![
            InlineKeyboardButton::callback("🌡️ Темп", "cfg_temp"),
            InlineKeyboardButton::callback("📝 Токены", "cfg_tokens"),
        ],
        vec![
            InlineKeyboardButton::callback(format!("👁️ Vision {}", if vision { "✅" } else { "❌" }), "cfg_toggle:vision_enabled"),
            InlineKeyboardButton::callback(format!("🎤 Voice {}", if voice { "✅" } else { "❌" }), "cfg_toggle:voice_enabled"),
        ],
        vec![
            InlineKeyboardButton::callback(format!("🌐 Web {}", if web { "✅" } else { "❌" }), "cfg_toggle:web_search_enabled"),
        ],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_model_select(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let models = state.llm_client.list_models().await.unwrap_or_default();
    let current = db::get_config(&state.db_pool, "ollama_chat_model").await.ok().flatten()
        .unwrap_or_else(|| state.config.ollama_chat_model.clone());
    
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];
    
    if models.is_empty() {
        buttons.push(vec![InlineKeyboardButton::callback("⚠️ Ollama недоступен", "config")]);
    } else {
        for model in models.iter().take(12) {
            let label = if model == &current { format!("✅ {}", model) } else { model.clone() };
            buttons.push(vec![InlineKeyboardButton::callback(label, format!("cfg_set_model:{}", model))]);
        }
    }
    buttons.push(vec![InlineKeyboardButton::callback("🔙 Назад", "config")]);
    
    let kb = InlineKeyboardMarkup::new(buttons);
    bot.edit_message_text(chat_id, msg_id, format!("🤖 <b>Выбор модели</b>\n\nТекущая: <code>{}</code>", current))
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_temperature_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let current = db::get_config_f64(&state.db_pool, "temperature", state.config.temperature).await;
    
    let temps = ["0.1", "0.3", "0.5", "0.7", "0.9", "1.0", "1.2", "1.5"];
    let buttons: Vec<Vec<InlineKeyboardButton>> = temps.chunks(4).map(|chunk| {
        chunk.iter().map(|t| {
            let val: f64 = t.parse().unwrap();
            let label = if (val - current).abs() < 0.01 { format!("✅ {}", t) } else { t.to_string() };
            InlineKeyboardButton::callback(label, format!("cfg_set_temp:{}", t))
        }).collect()
    }).collect();
    
    let mut kb_buttons = buttons;
    kb_buttons.push(vec![InlineKeyboardButton::callback("🔙 Назад", "config")]);
    
    let kb = InlineKeyboardMarkup::new(kb_buttons);
    bot.edit_message_text(chat_id, msg_id, format!("🌡️ <b>Температура</b>\n\nТекущая: <code>{}</code>\n\n• Ниже = точнее, предсказуемее\n• Выше = креативнее, разнообразнее", current))
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_tokens_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let current = db::get_config_u32(&state.db_pool, "max_tokens", state.config.max_tokens).await;
    
    let tokens = ["256", "512", "1024", "2048", "4096", "8192"];
    let buttons: Vec<Vec<InlineKeyboardButton>> = tokens.chunks(3).map(|chunk| {
        chunk.iter().map(|t| {
            let val: u32 = t.parse().unwrap();
            let label = if val == current { format!("✅ {}", t) } else { t.to_string() };
            InlineKeyboardButton::callback(label, format!("cfg_set_tokens:{}", t))
        }).collect()
    }).collect();
    
    let mut kb_buttons = buttons;
    kb_buttons.push(vec![InlineKeyboardButton::callback("🔙 Назад", "config")]);
    
    let kb = InlineKeyboardMarkup::new(kb_buttons);
    bot.edit_message_text(chat_id, msg_id, format!("📝 <b>Макс. токенов</b>\n\nТекущее: <code>{}</code>\n\nМаксимальная длина ответа модели", current))
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}


async fn edit_chat_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let settings = db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await
        .unwrap_or(db::ChatSettings {
            chat_id: chat_id.0,
            auto_reply_enabled: true,
            reply_mode: "mention_only".into(),
            cooldown_seconds: 5,
            context_depth: 10,
            rag_enabled: true,
        });
    
    let triggers = state.keyword_triggers.lock().await.get(&chat_id).cloned();
    let triggers_str = triggers.as_ref().map(|k| k.join(", ")).unwrap_or_else(|| "не заданы".to_string());
    let has_triggers = triggers.is_some() && !triggers.as_ref().unwrap().is_empty();
    
    let text = format!(
        "💬 <b>Настройки чата</b>\n\n\
        🔄 Автоответы: {}\n\
        📨 Режим: {}\n\
        🧠 RAG: {}\n\
        📚 Глубина памяти: {}\n\
        ⏱️ Cooldown: {}с\n\
        🎯 Триггеры: {}",
        if settings.auto_reply_enabled { "✅" } else { "❌" },
        if settings.reply_mode == "all_messages" { "все сообщения" } else { "только упоминания" },
        if settings.rag_enabled { "✅" } else { "❌" },
        settings.context_depth,
        settings.cooldown_seconds,
        triggers_str
    );
    
    let mut buttons = vec![
        vec![
            InlineKeyboardButton::callback(
                format!("🔄 Авто {}", if settings.auto_reply_enabled { "✅" } else { "❌" }),
                if settings.auto_reply_enabled { "chat_auto_off" } else { "chat_auto_on" }
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                if settings.reply_mode == "all_messages" { "📨 Все ✅" } else { "📨 Все" },
                "chat_mode_all"
            ),
            InlineKeyboardButton::callback(
                if settings.reply_mode == "mention_only" { "👤 Упом. ✅" } else { "👤 Упом." },
                "chat_mode_mention"
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                format!("🧠 RAG {}", if settings.rag_enabled { "✅" } else { "❌" }),
                if settings.rag_enabled { "chat_rag_off" } else { "chat_rag_on" }
            ),
            InlineKeyboardButton::callback("📚 Глубина", "chat_depth"),
        ],
        vec![
            InlineKeyboardButton::callback("⏱️ Cooldown", "chat_cooldown"),
            InlineKeyboardButton::callback("🎯 Триггеры", "chat_triggers"),
        ],
    ];
    
    if has_triggers {
        buttons.push(vec![InlineKeyboardButton::callback("🗑️ Очистить триггеры", "chat_triggers_clear")]);
    }
    
    buttons.push(vec![InlineKeyboardButton::callback("🔙 Назад", "main")]);
    
    let kb = InlineKeyboardMarkup::new(buttons);
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_cooldown_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let cooldowns = ["0", "3", "5", "10", "30", "60", "120"];
    let buttons: Vec<Vec<InlineKeyboardButton>> = cooldowns.chunks(4).map(|chunk| {
        chunk.iter().map(|cd| {
            InlineKeyboardButton::callback(format!("{}с", cd), format!("chat_set_cd:{}", cd))
        }).collect()
    }).collect();
    
    let mut kb_buttons = buttons;
    kb_buttons.push(vec![InlineKeyboardButton::callback("🔙 Назад", "chat")]);
    
    let kb = InlineKeyboardMarkup::new(kb_buttons);
    bot.edit_message_text(chat_id, msg_id, "⏱️ <b>Cooldown</b>\n\nМинимальный интервал между автоответами бота.\n0 = без ограничений")
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_memory_depth_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let settings = db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await
        .unwrap_or(db::ChatSettings { chat_id: chat_id.0, auto_reply_enabled: true, reply_mode: "mention_only".into(), cooldown_seconds: 5, context_depth: 10, rag_enabled: true });
    let current = settings.context_depth;
    
    let depths = ["5", "10", "15", "20", "30", "50"];
    let buttons: Vec<Vec<InlineKeyboardButton>> = depths.chunks(3).map(|chunk| {
        chunk.iter().map(|d| {
            let val: i64 = d.parse().unwrap();
            let label = if val == current { format!("✅ {}", d) } else { d.to_string() };
            InlineKeyboardButton::callback(label, format!("chat_set_depth:{}", d))
        }).collect()
    }).collect();
    
    let mut kb_buttons = buttons;
    kb_buttons.push(vec![InlineKeyboardButton::callback("🔙 Назад", "chat")]);
    
    let kb = InlineKeyboardMarkup::new(kb_buttons);
    bot.edit_message_text(chat_id, msg_id, format!("📚 <b>Глубина памяти RAG</b>\n\nТекущая: <code>{}</code>\n\nСколько сообщений учитывать для контекста", current))
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_ghost_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let is_active = state.is_ghost_mode(chat_id).await;
    let ghost_state = state.ghost_mode.lock().await.get(&chat_id).cloned();
    let save_examples = ghost_state.as_ref().map(|g| g.save_as_examples).unwrap_or(true);
    let duration = ghost_state.as_ref().map(|g| g.started_at.elapsed().as_secs() / 60).unwrap_or(0);
    
    let text = if is_active {
        format!(
            "👻 <b>Ghost Mode</b>\n\n\
            Статус: 🟢 <b>Активен</b> ({}м)\n\
            Сохранение: {}\n\n\
            <b>Сейчас:</b> твои сообщения отправляются от имени бота.\n\n\
            <b>Быстрые команды в чате:</b>\n\
            • <code>!status</code> — статус\n\
            • <code>!exit</code> — выход",
            duration,
            if save_examples { "✅ примеры сохраняются" } else { "❌ без сохранения" }
        )
    } else {
        "👻 <b>Ghost Mode</b>\n\n\
        Статус: 🔴 Выключен\n\n\
        <b>Что это:</b>\n\
        Режим, в котором ты пишешь от имени бота.\n\n\
        <b>Зачем:</b>\n\
        • Обучить персону на примерах\n\
        • Ответить за бота когда он тупит\n\
        • Показать как надо отвечать\n\n\
        <b>Как работает:</b>\n\
        1. Включаешь режим\n\
        2. Пишешь сообщение\n\
        3. Твоё сообщение удаляется\n\
        4. Появляется от имени бота\n\
        5. Сохраняется в RAG-память".to_string()
    };
    
    let kb = if is_active {
        InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback("🔴 Выключить", "ghost_off")],
            vec![InlineKeyboardButton::callback("🔙 Назад", "main")],
        ])
    } else {
        InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback("🟢 Включить", "ghost_on"),
            ],
            vec![
                InlineKeyboardButton::callback("🟡 Без сохранения", "ghost_on_nosave"),
            ],
            vec![InlineKeyboardButton::callback("🔙 Назад", "main")],
        ])
    };
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_tools_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("📢 Рассылка", "tools_broadcast")],
        vec![
            InlineKeyboardButton::callback("🗑️ Очистить историю", "tools_clear_history"),
            InlineKeyboardButton::callback("🧹 Очистить RAG", "tools_clear_memory"),
        ],
        vec![InlineKeyboardButton::callback("🛡️ Безопасность", "security")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, "🛠️ <b>Инструменты</b>\n\nДополнительные функции управления")
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_clear_history_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("⚠️ Да, очистить историю", "tools_clear_confirm")],
        vec![InlineKeyboardButton::callback("🔙 Отмена", "tools")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, "🗑️ <b>Очистка истории</b>\n\n⚠️ Это удалит всю историю сообщений в этом чате.\n\nВы уверены?")
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_status(bot: &Bot, chat_id: ChatId, msg_id: MessageId, state: &AppState) -> ResponseResult<()> {
    let ollama_ok = state.llm_client.check_health().await.unwrap_or(false);
    let db_ok = db::check_db_health(&state.db_pool).await.unwrap_or(false);
    
    let persona = db::get_active_persona(&state.db_pool).await.ok().flatten()
        .map(|p| p.name).unwrap_or_else(|| "—".to_string());
    
    let model = db::get_config(&state.db_pool, "ollama_chat_model").await.ok().flatten()
        .unwrap_or_else(|| state.config.ollama_chat_model.clone());
    
    let stats = state.queue_stats.lock().await.clone();
    let ghost = state.is_ghost_mode(chat_id).await;
    
    let msg_count = db::get_message_count(&state.db_pool, chat_id.0).await.unwrap_or(0);
    let memory_count = db::get_memory_count(&state.db_pool, chat_id.0).await.unwrap_or(0);
    
    let text = format!(
        "📊 <b>Статус системы</b>\n\n\
        <b>Сервисы:</b>\n\
        • Ollama: {}\n\
        • БД: {}\n\n\
        <b>Конфигурация:</b>\n\
        • Модель: <code>{}</code>\n\
        • Персона: {}\n\
        • Ghost: {}\n\n\
        <b>Очередь LLM:</b>\n\
        • Слотов: {}/{}\n\
        • Запросов: {} (✅{} ❌{})\n\
        • Среднее время: {}мс\n\n\
        <b>Этот чат:</b>\n\
        • Сообщений: {}\n\
        • RAG чанков: {}",
        if ollama_ok { "🟢" } else { "🔴" },
        if db_ok { "🟢" } else { "🔴" },
        model,
        persona,
        if ghost { "🟢" } else { "🔴" },
        state.llm_semaphore.available_permits(),
        state.config.max_concurrent_llm_requests.unwrap_or(3),
        stats.total_requests,
        stats.successful_requests,
        stats.failed_requests,
        stats.avg_response_time_ms,
        msg_count,
        memory_count
    );
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔄 Обновить", "status")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}


// === HELP SECTIONS ===

async fn edit_help(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🎭 Персоны", "help_personas"),
            InlineKeyboardButton::callback("⚙️ Конфиг", "help_config"),
        ],
        vec![
            InlineKeyboardButton::callback("💬 Чат", "help_chat"),
            InlineKeyboardButton::callback("👻 Ghost", "help_ghost"),
        ],
        vec![
            InlineKeyboardButton::callback("🧠 RAG", "help_rag"),
            InlineKeyboardButton::callback("📋 Команды", "help_commands"),
        ],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, "❓ <b>Помощь</b>\n\nВыберите раздел для подробной информации:")
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_help_personas(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let text = r#"🎭 <b>Персоны</b>

Персоны — это AI-личности с уникальными характерами и стилями общения.

<b>Что такое персона:</b>
• Имя — отображаемое название
• Промпт — системные инструкции для AI
• Статус — активна/неактивна

<b>Как создать:</b>
1. Меню → Персоны → Создать
2. Введите название
3. Введите системный промпт

<b>Системный промпт:</b>
Описывает характер, стиль речи, знания персоны.

<b>Пример промпта:</b>
<code>Ты Олег — расслабленный технический эксперт из чата. Говоришь прямо и честно, но без агрессии. Используешь живую речь: "Чел", "братан", "слушай". БЕЗ списков, БЕЗ "рад помочь". Если не знаешь — честно скажи. Признаёшь ошибки легко.</code>

<b>Советы:</b>
• Опишите характер живым языком
• Укажите как персона общается (примеры фраз)
• Добавьте правила поведения
• Можно указать что НЕ делать"#;
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 К помощи", "help")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_help_config(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let text = r#"⚙️ <b>Конфигурация</b>

<b>🤖 Модель</b>
Выбор LLM модели из установленных в Ollama.
• Большие модели умнее, но медленнее
• Маленькие быстрее, но проще

<b>🌡️ Температура (0.1-1.5)</b>
Контролирует "креативность" ответов:
• 0.1-0.3 — точные, предсказуемые ответы
• 0.5-0.7 — баланс точности и разнообразия
• 0.9-1.5 — креативные, неожиданные ответы

<b>📝 Макс. токенов</b>
Максимальная длина ответа модели.
• 512 — короткие ответы
• 2048 — средние
• 8192 — длинные, детальные

<b>👁️ Vision</b>
Анализ изображений (требует multimodal модель)

<b>🎤 Voice</b>
Распознавание голосовых сообщений (Whisper API)

<b>🌐 Web Search</b>
Поиск актуальной информации в интернете"#;
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 К помощи", "help")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_help_chat(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let text = r#"💬 <b>Настройки чата</b>

<b>🔄 Автоответы</b>
Включает/выключает автоматические ответы бота.

<b>📨 Режим ответов</b>
• <b>Все сообщения</b> — бот отвечает на всё
• <b>Упоминания</b> — только при @упоминании или реплае

<b>🧠 RAG</b>
Retrieval-Augmented Generation — бот помнит контекст разговора и использует релевантные воспоминания.

<b>📚 Глубина памяти</b>
Сколько последних сообщений учитывать для контекста (5-50).

<b>⏱️ Cooldown</b>
Минимальный интервал между автоответами.
Защита от спама в активных чатах.

<b>🎯 Триггеры</b>
Ключевые слова для активации бота.
Бот ответит если сообщение содержит триггер.

<b>Пример триггеров:</b>
<code>бот, помоги, вопрос</code>"#;
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 К помощи", "help")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_help_ghost(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let text = r#"👻 <b>Ghost Mode — пиши от имени бота</b>

<b>Что это:</b>
Режим, в котором твои сообщения отправляются от имени бота. Твоё сообщение удаляется, а вместо него появляется такое же — но от бота.

<b>Зачем нужен:</b>
• 📚 <b>Обучение</b> — показать персоне как надо отвечать
• 🔧 <b>Фикс</b> — ответить за бота когда он тупит
• 🎭 <b>Демо</b> — показать возможности бота

<b>Два режима:</b>
• 🟢 <b>С сохранением</b> — примеры идут в RAG-память, персона учится
• 🟡 <b>Без сохранения</b> — просто отправка, без обучения

<b>Как использовать:</b>
<code>/ghost on</code> — включить (с сохранением)
<code>/ghost on nosave</code> — включить (без сохранения)

<b>Быстрые команды в режиме:</b>
<code>!status</code> — сколько времени активен
<code>!exit</code> — выйти из режима

<b>Пример:</b>
1. Пишешь <code>/ghost on</code>
2. Пишешь "Привет! Как дела?"
3. Твоё сообщение исчезает
4. Появляется от бота: "Привет! Как дела?"
5. Пишешь <code>!exit</code> — выход"#;
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 К помощи", "help")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_help_rag(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let text = r#"🧠 <b>RAG (Retrieval-Augmented Generation)</b>

Система долгосрочной памяти бота.

<b>Как работает:</b>
1. Сообщения преобразуются в векторные эмбеддинги
2. При ответе ищутся релевантные воспоминания
3. Найденный контекст добавляется к промпту
4. Модель генерирует ответ с учётом истории

<b>Компоненты:</b>
• <b>Эмбеддинги</b> — векторные представления текста
• <b>Чанки памяти</b> — фрагменты разговоров
• <b>Важность</b> — вес воспоминания (decay со временем)
• <b>Саммари</b> — сжатые версии старых диалогов

<b>Настройки:</b>
• <b>Глубина</b> — сколько сообщений учитывать
• <b>Вкл/Выкл</b> — использовать ли RAG

<b>Автосуммаризация:</b>
Старые разговоры автоматически сжимаются в краткие саммари для экономии контекста.

<b>Советы:</b>
• Больше глубина = лучше память, но медленнее
• Периодически очищайте RAG если бот "путается""#;
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 К помощи", "help")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

async fn edit_help_commands(bot: &Bot, chat_id: ChatId, msg_id: MessageId) -> ResponseResult<()> {
    let text = r#"📋 <b>Команды</b>

<b>Основные:</b>
/menu — главное меню
/status — быстрый статус
/help — справка

<b>Персоны:</b>
/create_persona название|промпт
/list_personas
/activate_persona ID
/update_persona ID|название|промпт
/delete_persona ID
/export_persona ID
/export_all_personas
/import_persona {json}

<b>Модель:</b>
/set_model название
/set_temperature 0.0-2.0
/set_max_tokens число
/models — список моделей

<b>RAG:</b>
/enable_rag, /disable_rag
/set_memory_depth 1-50

<b>Чат:</b>
/enable_auto_reply, /disable_auto_reply
/reply_to_all, /reply_to_mention
/set_cooldown секунды
/triggers слово1, слово2

<b>Ghost:</b>
/ghost on|off|status

<b>Утилиты:</b>
/broadcast текст
/stats — статистика очереди
/cancel — отмена wizard"#;
    
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 К помощи", "help")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

// === SECURITY MENU ===

async fn edit_security_menu(bot: &Bot, chat_id: ChatId, msg_id: MessageId, _state: &AppState) -> ResponseResult<()> {
    let text = "🛡️ <b>Безопасность</b>\n\n\
        <b>Защита от prompt injection:</b>\n\
        • Санитизация пользовательского ввода\n\
        • Детекция подозрительных паттернов\n\
        • Адаптивный rate limiting\n\
        • Временные блокировки\n\n\
        <b>Настройки:</b>\n\
        • Порог страйка: 30 risk score\n\
        • Страйков до блока: 3\n\
        • Длительность блока: 5 мин\n\n\
        <b>Команды:</b>\n\
        <code>/block &lt;user_id&gt; [мин]</code>\n\
        <code>/unblock &lt;user_id&gt;</code>\n\
        <code>/security_status &lt;user_id&gt;</code>";

    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 Назад", "tools")],
    ]);
    
    bot.edit_message_text(chat_id, msg_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}

// === PUBLIC MENU SENDER (for /menu command) ===

pub async fn send_main_menu_new(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let kb = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🎭 Персоны", "personas"),
            InlineKeyboardButton::callback("⚙️ Конфиг", "config"),
        ],
        vec![
            InlineKeyboardButton::callback("💬 Чат", "chat"),
            InlineKeyboardButton::callback("👻 Ghost", "ghost"),
        ],
        vec![
            InlineKeyboardButton::callback("🛠️ Инструменты", "tools"),
            InlineKeyboardButton::callback("📊 Статус", "status"),
        ],
        vec![InlineKeyboardButton::callback("❓ Помощь", "help")],
    ]);
    
    bot.send_message(chat_id, "🤖 <b>PersonaForge</b>\n\nВыберите раздел:")
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;
    Ok(())
}
