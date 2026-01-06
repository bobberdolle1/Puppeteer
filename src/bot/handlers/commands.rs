use crate::db;
use crate::state::AppState;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::net::Download;

pub async fn handle_command(bot: Bot, msg: Message, state: AppState) -> ResponseResult<()> {
    let text = msg.text().unwrap_or_default();
    let chat_id = msg.chat.id;
    let user_id = msg.from.as_ref().map(|u| u.id.0);
    let username = msg.from.as_ref().map(|u| u.first_name.as_str()).unwrap_or("Unknown");

    // Parse command and check if it's for this bot
    let cmd_full = text.split_whitespace().next().unwrap_or("");
    let (cmd, target_bot) = if let Some(at_pos) = cmd_full.find('@') {
        (&cmd_full[..at_pos], Some(&cmd_full[at_pos + 1..]))
    } else {
        (cmd_full, None)
    };

    // In group chats, ignore commands targeted at other bots
    if !msg.chat.is_private() {
        if let Some(target) = target_bot {
            let bot_username = state.get_bot_username().await;
            if bot_username.as_ref().map(|u| u.as_str()) != Some(target) {
                // Command is for another bot, ignore silently
                return Ok(());
            }
        }
    }

    // List of known commands
    let known_commands = [
        "/start", "/cancel", "/create_persona", "/list_personas", "/activate_persona",
        "/update_persona", "/delete_persona", "/set_model", "/set_temperature", "/set_max_tokens",
        "/enable_rag", "/disable_rag", "/set_memory_depth", "/status", "/enable_auto_reply",
        "/disable_auto_reply", "/reply_to_all", "/reply_to_mention", "/set_cooldown",
        "/menu", "/settings", "/help", "/triggers", "/keywords", "/broadcast",
        "/queue_stats", "/stats", "/models", "/export_persona", "/export_all_personas",
        "/import_persona", "/block", "/unblock", "/security_status"
    ];

    // In group chats, ignore unknown commands (they might be for other bots)
    if !msg.chat.is_private() && !known_commands.contains(&cmd) {
        return Ok(());
    }

    log::info!("⚡ Command from {} ({}): {}", username, user_id.unwrap_or(0), text);

    // /start доступен всем
    if cmd == "/start" {
        return handle_start(bot, msg, &state).await;
    }

    // Остальные команды только для владельца
    if user_id != Some(state.config.owner_id) {
        // In groups, don't spam about permissions for unknown users
        if msg.chat.is_private() {
            bot.send_message(chat_id, "❌ У вас нет прав для выполнения этой команды.").await?;
        }
        return Ok(());
    }

    match cmd {
        "/cancel" => handle_cancel(bot, msg, &state).await,
        "/create_persona" => handle_create_persona(bot, msg, &state).await,
        "/list_personas" => handle_list_personas(bot, msg, &state).await,
        "/activate_persona" => handle_activate_persona(bot, msg, &state).await,
        "/update_persona" => handle_update_persona(bot, msg, &state).await,
        "/delete_persona" => handle_delete_persona(bot, msg, &state).await,
        "/set_model" => handle_set_model(bot, msg).await,
        "/set_temperature" => handle_set_temperature(bot, msg).await,
        "/set_max_tokens" => handle_set_max_tokens(bot, msg).await,
        "/enable_rag" => handle_enable_rag(bot, msg, &state).await,
        "/disable_rag" => handle_disable_rag(bot, msg, &state).await,
        "/set_memory_depth" => handle_set_memory_depth(bot, msg, &state).await,
        "/status" => handle_status(bot, msg, &state).await,
        "/enable_auto_reply" => handle_enable_auto_reply(bot, msg, &state).await,
        "/disable_auto_reply" => handle_disable_auto_reply(bot, msg, &state).await,
        "/reply_to_all" => handle_reply_to_all(bot, msg, &state).await,
        "/reply_to_mention" => handle_reply_to_mention(bot, msg, &state).await,
        "/set_cooldown" => handle_set_cooldown(bot, msg, &state).await,
        "/menu" => {
            crate::bot::handlers::callbacks::send_main_menu_new(&bot, chat_id, &state).await?;
            Ok(())
        }
        "/settings" => send_settings_menu(bot, chat_id).await,
        "/help" => send_help_message(bot, chat_id).await,
        "/triggers" | "/keywords" => handle_set_triggers(bot, msg, &state).await,
        "/broadcast" => handle_broadcast(bot, msg, &state).await,
        "/queue_stats" | "/stats" => handle_queue_stats(bot, msg, &state).await,
        "/models" => handle_list_models(bot, msg, &state).await,
        "/export_persona" => handle_export_persona(bot, msg, &state).await,
        "/export_all_personas" => handle_export_all_personas(bot, msg, &state).await,
        "/import_persona" => handle_import_persona(bot, msg, &state).await,
        // Security commands
        "/block" => handle_block_user(bot, msg, &state).await,
        "/unblock" => handle_unblock_user(bot, msg, &state).await,
        "/security_status" => handle_security_status(bot, msg, &state).await,
        _ => {
            // Only respond to unknown commands in private chats
            if msg.chat.is_private() {
                bot.send_message(chat_id, "❌ Неизвестная команда. /help").await?;
            }
            Ok(())
        }
    }
}

async fn handle_create_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    if parts.len() < 2 {
        bot.send_message(chat_id, "❌ Формат: /create_persona название|описание").await?;
        return Ok(());
    }

    let data: Vec<&str> = parts[1].splitn(2, '|').collect();
    if data.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /create_persona название|описание").await?;
        return Ok(());
    }

    let (name, prompt) = (data[0].trim(), data[1].trim());
    if name.is_empty() || prompt.is_empty() {
        bot.send_message(chat_id, "❌ Название и описание не могут быть пустыми.").await?;
        return Ok(());
    }

    match db::create_persona(&state.db_pool, name, prompt).await {
        Ok(id) => { bot.send_message(chat_id, format!("✅ Персона создана с ID: {}", id)).await?; }
        Err(e) => { log::error!("Create persona error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

pub async fn handle_list_personas(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match db::get_all_personas(&state.db_pool).await {
        Ok(personas) if !personas.is_empty() => {
            let mut text = "📋 <b>Персоны:</b>\n\n".to_string();
            for p in personas {
                let status = if p.is_active { "🟢" } else { "⚪" };
                let preview = if p.prompt.len() > 80 { format!("{}...", &p.prompt[..80]) } else { p.prompt.clone() };
                text.push_str(&format!("{} <b>{}</b> (ID: {})\n<i>{}</i>\n\n", status, p.name, p.id, preview));
            }
            bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
        }
        _ => { bot.send_message(chat_id, "📋 Нет персон.").await?; }
    }
    Ok(())
}

async fn handle_activate_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /activate_persona ID").await?;
        return Ok(());
    }

    let id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => { bot.send_message(chat_id, "❌ ID должен быть числом.").await?; return Ok(()); }
    };

    match db::set_active_persona(&state.db_pool, id).await {
        Ok(()) => { bot.send_message(chat_id, format!("✅ Персона {} активирована.", id)).await?; }
        Err(e) => { log::error!("Activate error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

async fn handle_update_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    
    if parts.len() < 2 {
        bot.send_message(chat_id, "❌ Формат: /update_persona ID|название|описание").await?;
        return Ok(());
    }

    let data: Vec<&str> = parts[1].splitn(3, '|').collect();
    if data.len() != 3 {
        bot.send_message(chat_id, "❌ Формат: /update_persona ID|название|описание").await?;
        return Ok(());
    }

    let id = match data[0].parse::<i64>() {
        Ok(id) => id,
        Err(_) => { bot.send_message(chat_id, "❌ ID должен быть числом.").await?; return Ok(()); }
    };

    let (name, prompt) = (data[1].trim(), data[2].trim());
    match db::update_persona(&state.db_pool, id, name, prompt).await {
        Ok(()) => { bot.send_message(chat_id, format!("✅ Персона {} обновлена.", id)).await?; }
        Err(e) => { log::error!("Update error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

async fn handle_delete_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /delete_persona ID").await?;
        return Ok(());
    }

    let id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => { bot.send_message(chat_id, "❌ ID должен быть числом.").await?; return Ok(()); }
    };

    match db::delete_persona(&state.db_pool, id).await {
        Ok(()) => { bot.send_message(chat_id, format!("✅ Персона {} удалена.", id)).await?; }
        Err(e) => { log::error!("Delete error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

async fn handle_set_model(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    
    if parts.len() != 2 || parts[1].trim().is_empty() {
        bot.send_message(chat_id, "❌ Формат: /set_model название").await?;
        return Ok(());
    }
    bot.send_message(chat_id, format!("✅ Модель: {}", parts[1].trim())).await?;
    Ok(())
}

async fn handle_set_temperature(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /set_temperature 0.0-2.0").await?;
        return Ok(());
    }

    match parts[1].trim().parse::<f64>() {
        Ok(t) if (0.0..=2.0).contains(&t) => { bot.send_message(chat_id, format!("✅ Температура: {}", t)).await?; }
        _ => { bot.send_message(chat_id, "❌ Значение должно быть 0.0-2.0").await?; }
    }
    Ok(())
}

async fn handle_set_max_tokens(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /set_max_tokens число").await?;
        return Ok(());
    }

    match parts[1].trim().parse::<u32>() {
        Ok(t) if t > 0 => { bot.send_message(chat_id, format!("✅ Макс. токенов: {}", t)).await?; }
        _ => { bot.send_message(chat_id, "❌ Должно быть положительным числом").await?; }
    }
    Ok(())
}


pub async fn handle_enable_rag(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match db::toggle_rag_for_chat(&state.db_pool, chat_id.0, true).await {
        Ok(()) => { bot.send_message(chat_id, "✅ RAG включен.").await?; }
        Err(e) => { log::error!("RAG error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

pub async fn handle_disable_rag(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match db::toggle_rag_for_chat(&state.db_pool, chat_id.0, false).await {
        Ok(()) => { bot.send_message(chat_id, "✅ RAG отключен.").await?; }
        Err(e) => { log::error!("RAG error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

async fn handle_set_memory_depth(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /set_memory_depth 1-50").await?;
        return Ok(());
    }

    let depth = match parts[1].trim().parse::<u32>() {
        Ok(d) if d > 0 && d <= 50 => d,
        _ => { bot.send_message(chat_id, "❌ Значение 1-50").await?; return Ok(()); }
    };

    let settings = db::get_or_create_chat_settings(&state.db_pool, chat_id.0).await
        .unwrap_or(db::ChatSettings { chat_id: chat_id.0, auto_reply_enabled: true, reply_mode: "mention_only".into(), cooldown_seconds: 5, context_depth: 10, rag_enabled: true });

    match db::update_rag_settings(&state.db_pool, chat_id.0, settings.rag_enabled, depth as i64).await {
        Ok(()) => { bot.send_message(chat_id, format!("✅ Глубина памяти: {}", depth)).await?; }
        Err(e) => { log::error!("Memory depth error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

pub async fn handle_status(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    
    let ollama = if state.llm_client.check_health().await.unwrap_or(false) { "🟢" } else { "🔴" };
    let db_ok = if db::check_db_health(&state.db_pool).await.unwrap_or(false) { "🟢" } else { "🔴" };
    let persona = match db::get_active_persona(&state.db_pool).await {
        Ok(Some(p)) => p.name,
        _ => "Не выбрана".into(),
    };
    let stats = state.queue_stats.lock().await;

    let text = format!(
r#"📊 <b>Статус</b>

<b>Сервисы:</b> Ollama {} | БД {}
<b>Персона:</b> {}
<b>Очередь:</b> {}/{} | Запросов: {} (✅{} ❌{})
<b>Модель:</b> {}
<b>Температура:</b> {} | Токены: {}"#,
        ollama, db_ok, persona,
        state.llm_semaphore.available_permits(),
        state.config.max_concurrent_llm_requests.unwrap_or(3),
        stats.total_requests, stats.successful_requests, stats.failed_requests,
        state.config.ollama_chat_model,
        state.config.temperature, state.config.max_tokens
    );

    bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
    Ok(())
}

pub async fn handle_enable_auto_reply(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, true).await {
        Ok(()) => { bot.send_message(chat_id, "✅ Автоответы включены.").await?; }
        Err(e) => { log::error!("Auto-reply error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

pub async fn handle_disable_auto_reply(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match db::toggle_auto_reply_for_chat(&state.db_pool, chat_id.0, false).await {
        Ok(()) => { bot.send_message(chat_id, "✅ Автоответы отключены.").await?; }
        Err(e) => { log::error!("Auto-reply error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

pub async fn handle_reply_to_all(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "all_messages").await {
        Ok(()) => { bot.send_message(chat_id, "✅ Режим: все сообщения.").await?; }
        Err(e) => { log::error!("Reply mode error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

pub async fn handle_reply_to_mention(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match db::update_reply_mode_for_chat(&state.db_pool, chat_id.0, "mention_only").await {
        Ok(()) => { bot.send_message(chat_id, "✅ Режим: только упоминания.").await?; }
        Err(e) => { log::error!("Reply mode error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}

async fn handle_set_cooldown(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();
    
    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /set_cooldown секунды").await?;
        return Ok(());
    }

    let cooldown = match parts[1].trim().parse::<u32>() {
        Ok(c) if c <= 300 => c,
        _ => { bot.send_message(chat_id, "❌ Значение 0-300").await?; return Ok(()); }
    };

    match db::update_cooldown_for_chat(&state.db_pool, chat_id.0, cooldown as i64).await {
        Ok(()) => { bot.send_message(chat_id, format!("✅ Cooldown: {}с", cooldown)).await?; }
        Err(e) => { log::error!("Cooldown error: {}", e); bot.send_message(chat_id, "❌ Ошибка.").await?; }
    }
    Ok(())
}


async fn handle_set_triggers(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    use crate::state::WizardState;
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();

    match parts.get(1) {
        Some(&"clear") => {
            state.keyword_triggers.lock().await.remove(&chat_id);
            bot.send_message(chat_id, "✅ Триггеры удалены.").await?;
        }
        Some(kw) if !kw.is_empty() => {
            let keywords: Vec<String> = kw.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
            if keywords.is_empty() {
                bot.send_message(chat_id, "❌ Введите слова через запятую.").await?;
            } else {
                state.keyword_triggers.lock().await.insert(chat_id, keywords.clone());
                bot.send_message(chat_id, format!("✅ Триггеры: {}", keywords.join(", "))).await?;
            }
        }
        _ => {
            let current = state.keyword_triggers.lock().await.get(&chat_id).cloned();
            match current {
                Some(kw) if !kw.is_empty() => {
                    bot.send_message(chat_id, format!("🔑 Триггеры: {}\n\n/triggers clear - удалить", kw.join(", "))).await?;
                }
                _ => {
                    state.set_wizard_state(chat_id, WizardState::SettingKeywords).await;
                    bot.send_message(chat_id, "🔑 Введите ключевые слова через запятую:\n\n/cancel для отмены").await?;
                }
            }
        }
    }
    Ok(())
}

async fn handle_broadcast(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.splitn(2, ' ').collect();

    match parts.get(1) {
        Some(message) if !message.is_empty() => {
            let chats = db::get_all_chat_ids(&state.db_pool).await.unwrap_or_default();
            if chats.is_empty() {
                bot.send_message(chat_id, "❌ Нет чатов.").await?;
                return Ok(());
            }

            let (mut ok, mut err) = (0, 0);
            for target in &chats {
                match bot.send_message(ChatId(*target), *message).await {
                    Ok(_) => ok += 1,
                    Err(_) => err += 1,
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            bot.send_message(chat_id, format!("📢 Рассылка: ✅{} ❌{}", ok, err)).await?;
        }
        _ => {
            bot.send_message(chat_id, "📢 Формат: /broadcast текст").await?;
        }
    }
    Ok(())
}

async fn handle_queue_stats(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let stats = state.queue_stats.lock().await.clone();
    let available = state.llm_semaphore.available_permits();
    let max = state.config.max_concurrent_llm_requests.unwrap_or(3);

    let text = format!(
r#"📊 <b>Очередь LLM</b>

Слотов: {}/{}
Запросов: {}
✅ Успешных: {}
❌ Ошибок: {}
⏱️ Таймаутов: {}
⚡ Среднее время: {}мс"#,
        available, max, stats.total_requests, stats.successful_requests,
        stats.failed_requests, stats.queue_timeouts, stats.avg_response_time_ms
    );

    bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
    Ok(())
}

async fn handle_list_models(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    match state.llm_client.list_models().await {
        Ok(models) if !models.is_empty() => {
            let list = models.iter().map(|m| format!("• {}", m)).collect::<Vec<_>>().join("\n");
            bot.send_message(chat_id, format!("🤖 <b>Модели:</b>\n\n{}\n\nТекущая: {}", list, state.config.ollama_chat_model))
                .parse_mode(ParseMode::Html).await?;
        }
        _ => { bot.send_message(chat_id, "❌ Модели не найдены.").await?; }
    }
    Ok(())
}

async fn handle_export_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() != 2 {
        bot.send_message(chat_id, "❌ Формат: /export_persona ID").await?;
        return Ok(());
    }

    let id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => { bot.send_message(chat_id, "❌ ID должен быть числом.").await?; return Ok(()); }
    };

    match db::export_persona(&state.db_pool, id).await {
        Ok(Some(json)) => {
            // Send as document
            let filename = format!("persona_{}.json", id);
            let doc = teloxide::types::InputFile::memory(json.into_bytes()).file_name(filename);
            bot.send_document(chat_id, doc)
                .caption("📤 Экспорт персоны")
                .await?;
        }
        Ok(None) => { bot.send_message(chat_id, "❌ Персона не найдена.").await?; }
        Err(e) => { log::error!("Export error: {}", e); bot.send_message(chat_id, "❌ Ошибка экспорта.").await?; }
    }
    Ok(())
}

async fn handle_export_all_personas(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match db::export_all_personas(&state.db_pool).await {
        Ok(json) => {
            let doc = teloxide::types::InputFile::memory(json.into_bytes()).file_name("personas_export.json");
            bot.send_document(chat_id, doc)
                .caption("📤 Экспорт всех персон")
                .await?;
        }
        Err(e) => { log::error!("Export error: {}", e); bot.send_message(chat_id, "❌ Ошибка экспорта.").await?; }
    }
    Ok(())
}

async fn handle_import_persona(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    
    // Check if message has a document attached
    if let Some(doc) = msg.document() {
        let file = bot.get_file(doc.file.id.clone()).await?;
        let mut buffer = Vec::new();
        bot.download_file(&file.path, &mut buffer).await?;
        
        let json = String::from_utf8_lossy(&buffer);
        
        // Try to import as array first, then as single
        match db::import_personas(&state.db_pool, &json).await {
            Ok(ids) if !ids.is_empty() => {
                bot.send_message(chat_id, format!("✅ Импортировано {} персон: {:?}", ids.len(), ids)).await?;
            }
            Ok(_) => {
                // Try single import
                match db::import_persona(&state.db_pool, &json).await {
                    Ok(id) => { bot.send_message(chat_id, format!("✅ Персона импортирована с ID: {}", id)).await?; }
                    Err(e) => { bot.send_message(chat_id, format!("❌ Ошибка импорта: {}", e)).await?; }
                }
            }
            Err(e) => { bot.send_message(chat_id, format!("❌ Ошибка импорта: {}", e)).await?; }
        }
    } else {
        // Check for JSON in message text
        let text = msg.text().unwrap_or_default();
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        
        if parts.len() < 2 || parts[1].trim().is_empty() {
            bot.send_message(chat_id, "📥 <b>Импорт персоны</b>\n\nОтправьте JSON-файл или:\n/import_persona {\"name\":\"...\",\"prompt\":\"...\"}").parse_mode(ParseMode::Html).await?;
            return Ok(());
        }

        let json = parts[1].trim();
        match db::import_persona(&state.db_pool, json).await {
            Ok(id) => { bot.send_message(chat_id, format!("✅ Персона импортирована с ID: {}", id)).await?; }
            Err(e) => { bot.send_message(chat_id, format!("❌ Ошибка: {}", e)).await?; }
        }
    }
    Ok(())
}

pub async fn send_main_menu(bot: Bot, chat_id: ChatId) -> ResponseResult<()> {
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("👤 Персоны", "personas_menu")],
        vec![InlineKeyboardButton::callback("⚙️ Модель", "model_settings")],
        vec![InlineKeyboardButton::callback("🧠 RAG", "rag_settings")],
        vec![InlineKeyboardButton::callback("💬 Чат", "chat_settings")],
        vec![InlineKeyboardButton::callback("📊 Статус", "system_status")],
        vec![InlineKeyboardButton::callback("ℹ️ Помощь", "help_info")],
    ]);
    bot.send_message(chat_id, "🤖 <b>PersonaForge</b>").parse_mode(ParseMode::Html).reply_markup(kb).await?;
    Ok(())
}

pub async fn send_settings_menu(bot: Bot, chat_id: ChatId) -> ResponseResult<()> {
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
    let kb = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🎭 Персона", "personas_menu")],
        vec![InlineKeyboardButton::callback("🧠 Память", "memory_settings")],
        vec![InlineKeyboardButton::callback("⚙️ Модель", "model_params")],
        vec![InlineKeyboardButton::callback("🔙 Назад", "main_menu")],
    ]);
    bot.send_message(chat_id, "🔧 <b>Настройки</b>").parse_mode(ParseMode::Html).reply_markup(kb).await?;
    Ok(())
}

pub async fn send_help_message(bot: Bot, chat_id: ChatId) -> ResponseResult<()> {
    let text = r#"🤖 <b>PersonaForge</b>

<b>👤 Персоны:</b>
/create_persona название|описание
/list_personas
/activate_persona ID
/update_persona ID|название|описание
/delete_persona ID
/export_persona ID
/export_all_personas
/import_persona (+ JSON файл)

<b>⚙️ Модель:</b>
/set_model, /set_temperature, /set_max_tokens
/models - список моделей

<b>🧠 RAG:</b>
/enable_rag, /disable_rag
/set_memory_depth 1-50

<b>💬 Чат:</b>
/enable_auto_reply, /disable_auto_reply
/reply_to_all, /reply_to_mention
/set_cooldown, /triggers

<b>📊 Система:</b>
/status, /stats, /broadcast

<b>🛡️ Безопасность:</b>
/block, /unblock, /security_status

<b>🎛️ Меню:</b>
/menu, /settings

<b>💡 Треды:</b>
Бот поддерживает треды в супергруппах"#;

    bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
    Ok(())
}

// ============================================================================
// Security commands
// ============================================================================

/// Block a user manually: /block <user_id> [minutes]
async fn handle_block_user(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() < 2 {
        bot.send_message(chat_id, "❌ Формат: /block <user_id> [минуты]\nПример: /block 123456789 30").await?;
        return Ok(());
    }

    let user_id: u64 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Неверный user_id").await?;
            return Ok(());
        }
    };

    // Don't allow blocking owner
    if user_id == state.config.owner_id {
        bot.send_message(chat_id, "❌ Нельзя заблокировать владельца").await?;
        return Ok(());
    }

    let minutes: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(30);
    let duration = std::time::Duration::from_secs(minutes * 60);

    state.security_tracker.block_user(user_id, duration).await;

    bot.send_message(
        chat_id,
        format!("🔒 Пользователь {} заблокирован на {} минут", user_id, minutes)
    ).await?;

    Ok(())
}

/// Unblock a user: /unblock <user_id>
async fn handle_unblock_user(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() < 2 {
        bot.send_message(chat_id, "❌ Формат: /unblock <user_id>").await?;
        return Ok(());
    }

    let user_id: u64 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Неверный user_id").await?;
            return Ok(());
        }
    };

    state.security_tracker.unblock_user(user_id).await;

    bot.send_message(
        chat_id,
        format!("🔓 Пользователь {} разблокирован", user_id)
    ).await?;

    Ok(())
}

/// Check security status for a user: /security_status [user_id]
async fn handle_security_status(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() < 2 {
        // Show general security info
        let response = r#"🛡️ <b>Система безопасности</b>

<b>Настройки:</b>
• Порог страйка: 30 risk score
• Страйков до блока: 3
• Длительность блока: 5 мин
• Окно страйков: 1 час

<b>Команды:</b>
• /block &lt;user_id&gt; [мин] - заблокировать
• /unblock &lt;user_id&gt; - разблокировать
• /security_status &lt;user_id&gt; - статус пользователя"#;

        bot.send_message(chat_id, response).parse_mode(ParseMode::Html).await?;
        return Ok(());
    }

    let user_id: u64 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(chat_id, "❌ Неверный user_id").await?;
            return Ok(());
        }
    };

    let response = if let Some((strikes, total_violations, is_blocked)) = 
        state.security_tracker.get_user_stats(user_id).await 
    {
        let status = if is_blocked { "🔒 Заблокирован" } else { "✅ Активен" };
        format!(
            "🛡️ <b>Пользователь {}</b>\n\n\
            Статус: {}\n\
            Текущие страйки: {}/3\n\
            Всего нарушений: {}",
            user_id, status, strikes, total_violations
        )
    } else {
        format!("✅ Пользователь {} не имеет нарушений", user_id)
    };

    bot.send_message(chat_id, response).parse_mode(ParseMode::Html).await?;
    Ok(())
}

async fn handle_start(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let user_name = msg.from.as_ref()
        .map(|u| u.first_name.as_str())
        .unwrap_or("друг");

    let welcome = format!(
        "👋 <b>Привет, {}!</b>\n\n\
        Я <b>PersonaForge</b> — AI-бот с кастомными персонами.\n\n\
        🎭 <b>Что умею:</b>\n\
        • Общаться в разных стилях\n\
        • Помнить контекст разговора\n\
        • Распознавать голосовые сообщения\n\
        • Анализировать изображения\n\n\
        Просто напиши мне что-нибудь! 💬",
        user_name
    );

    bot.send_message(chat_id, welcome)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

async fn handle_cancel(bot: Bot, msg: Message, state: &AppState) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    
    if state.get_wizard_state(chat_id).await.is_some() {
        state.clear_wizard_state(chat_id).await;
        bot.send_message(chat_id, "✅ Операция отменена.").await?;
    } else {
        bot.send_message(chat_id, "ℹ️ Нечего отменять.").await?;
    }
    Ok(())
}
