use crate::{
    bot::AddAccountDialogue,
    db::AccountRepository,
    AppState,
};
use anyhow::Result;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

/// Main menu keyboard
pub fn main_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("👥 Manage Accounts", "menu:accounts")],
        vec![InlineKeyboardButton::callback("⚙️ Global Settings", "menu:settings")],
        vec![InlineKeyboardButton::callback("📊 Statistics", "menu:stats")],
    ])
}

/// Account list keyboard
pub async fn accounts_keyboard(state: &AppState) -> Result<InlineKeyboardMarkup> {
    let accounts = AccountRepository::list_all(&state.db_pool).await?;
    
    let mut buttons = vec![];
    
    for account in accounts {
        let is_running = state.is_userbot_running(account.id).await;
        let status = if is_running { "🟢" } else { "🔴" };
        let label = format!("{} {} (ID: {})", status, account.phone_number, account.id);
        buttons.push(vec![InlineKeyboardButton::callback(
            label,
            format!("account:{}", account.id),
        )]);
    }
    
    buttons.push(vec![InlineKeyboardButton::callback("➕ Add Account", "account:add")]);
    buttons.push(vec![InlineKeyboardButton::callback("🔙 Back", "menu:main")]);
    
    Ok(InlineKeyboardMarkup::new(buttons))
}

/// Account control panel keyboard
pub fn account_control_keyboard(account_id: i64, is_running: bool) -> InlineKeyboardMarkup {
    let start_stop = if is_running {
        InlineKeyboardButton::callback("🔴 Stop", format!("acc:stop:{}", account_id))
    } else {
        InlineKeyboardButton::callback("🟢 Start", format!("acc:start:{}", account_id))
    };
    
    InlineKeyboardMarkup::new(vec![
        vec![start_stop],
        vec![InlineKeyboardButton::callback(
            "📝 Edit Prompt",
            format!("acc:prompt:{}", account_id),
        )],
        vec![InlineKeyboardButton::callback(
            "🎲 Set Probability",
            format!("acc:prob:{}", account_id),
        )],
        vec![InlineKeyboardButton::callback(
            "💬 Manage Chats",
            format!("acc:chats:{}", account_id),
        )],
        vec![InlineKeyboardButton::callback(
            "🗑 Delete Account",
            format!("acc:delete:{}", account_id),
        )],
        vec![InlineKeyboardButton::callback("🔙 Back", "menu:accounts")],
    ])
}

/// Handle callback queries
pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    dialogue: AddAccountDialogue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(data) = &q.data {
        let parts: Vec<&str> = data.split(':').collect();
        
        match parts[0] {
            "menu" => handle_menu_callback(&bot, &q, &state, parts).await?,
            "account" => handle_account_list_callback(&bot, &q, &state, parts).await?,
            "acc" => handle_account_control_callback(&bot, &q, &state, &dialogue, parts).await?,
            _ => {}
        }
    }
    
    // Answer callback to remove loading state
    bot.answer_callback_query(&q.id).await?;
    Ok(())
}

async fn handle_menu_callback(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    parts: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message = match &q.message {
        Some(msg) => msg,
        None => return Ok(()),
    };
    
    let chat_id = message.chat().id;
    let message_id = message.id();
    
    match parts.get(1) {
        Some(&"main") => {
            bot.edit_message_text(
                chat_id,
                message_id,
                "🎭 <b>Puppeteer Admin Panel</b>\n\nSelect an option:",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(main_menu_keyboard())
            .await?;
        }
        Some(&"accounts") => {
            let keyboard = accounts_keyboard(state).await?;
            bot.edit_message_text(
                chat_id,
                message_id,
                "👥 <b>Account Management</b>\n\nSelect an account to manage:",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
        }
        Some(&"settings") => {
            bot.edit_message_text(
                chat_id,
                message_id,
                "⚙️ <b>Global Settings</b>\n\n🚧 Coming soon...",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback("🔙 Back", "menu:main")],
            ]))
            .await?;
        }
        Some(&"stats") => {
            let active_count = state.active_userbot_count().await;
            let all_accounts = AccountRepository::list_all(&state.db_pool).await?;
            
            let text = format!(
                "📊 <b>Statistics</b>\n\n\
                🤖 Active Userbots: {}\n\
                📱 Total Accounts: {}\n",
                active_count,
                all_accounts.len()
            );
            
            bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(InlineKeyboardMarkup::new(vec![
                    vec![InlineKeyboardButton::callback("🔙 Back", "menu:main")],
                ]))
                .await?;
        }
        _ => {}
    }
    
    Ok(())
}

async fn handle_account_list_callback(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    parts: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message = match &q.message {
        Some(msg) => msg,
        None => return Ok(()),
    };
    
    let chat_id = message.chat().id;
    
    if parts.get(1) == Some(&"add") {
        bot.send_message(
            chat_id,
            "📱 <b>Add New Userbot Account</b>\n\n\
            Please send the phone number in international format (e.g., +1234567890).\n\n\
            Send /cancel to abort.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }
    
    if let Some(account_id_str) = parts.get(1) {
        if let Ok(account_id) = account_id_str.parse::<i64>() {
            if let Some(account) = AccountRepository::get_by_id(&state.db_pool, account_id).await? {
                let is_running = state.is_userbot_running(account_id).await;
                let status = if is_running { "🟢 Running" } else { "🔴 Stopped" };
                
                let text = format!(
                    "📱 <b>Account: {}</b>\n\n\
                    ID: {}\n\
                    Status: {}\n\
                    Reply Probability: {}%\n\n\
                    <i>System Prompt:</i>\n<code>{}</code>",
                    account.phone_number,
                    account.id,
                    status,
                    account.reply_probability,
                    account.system_prompt
                );
                
                bot.edit_message_text(chat_id, message.id(), text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(account_control_keyboard(account_id, is_running))
                    .await?;
            }
        }
    }
    
    Ok(())
}

async fn handle_account_control_callback(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    dialogue: &AddAccountDialogue,
    parts: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message = match &q.message {
        Some(msg) => msg,
        None => return Ok(()),
    };
    
    let chat_id = message.chat().id;
    let message_id = message.id();
    
    if parts.len() < 3 {
        return Ok(());
    }
    
    let action = parts[1];
    let account_id: i64 = parts[2].parse()?;
    
    match action {
        "start" => {
            if !state.is_userbot_running(account_id).await {
                crate::userbot::spawn_userbot(state.clone(), account_id).await?;
                AccountRepository::set_active(&state.db_pool, account_id, true).await?;
                
                bot.answer_callback_query(&q.id)
                    .text("✅ Userbot started!")
                    .await?;
            }
        }
        "stop" => {
            if state.is_userbot_running(account_id).await {
                state.shutdown_userbot(account_id).await?;
                AccountRepository::set_active(&state.db_pool, account_id, false).await?;
                
                bot.answer_callback_query(&q.id)
                    .text("✅ Userbot stopped!")
                    .await?;
            }
        }
        "delete" => {
            if state.is_userbot_running(account_id).await {
                state.shutdown_userbot(account_id).await?;
            }
            AccountRepository::delete(&state.db_pool, account_id).await?;
            
            bot.answer_callback_query(&q.id)
                .text("✅ Account deleted!")
                .await?;
            
            let keyboard = accounts_keyboard(state).await?;
            bot.edit_message_text(
                chat_id,
                message_id,
                "👥 <b>Account Management</b>\n\nAccount deleted. Select another account:",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
            
            return Ok(());
        }
        "prompt" => {
            bot.send_message(
                chat_id,
                format!("📝 <b>Edit System Prompt</b>\n\nSend the new system prompt for account {}.\n\nSend /cancel to abort.", account_id),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
        "prob" => {
            bot.send_message(
                chat_id,
                format!("🎲 <b>Set Reply Probability</b>\n\nSend a number between 0-100 for account {}.\n\nSend /cancel to abort.", account_id),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
        _ => {}
    }
    
    // Refresh the account panel
    if let Some(account) = AccountRepository::get_by_id(&state.db_pool, account_id).await? {
        let is_running = state.is_userbot_running(account_id).await;
        let status = if is_running { "🟢 Running" } else { "🔴 Stopped" };
        
        let text = format!(
            "📱 <b>Account: {}</b>\n\n\
            ID: {}\n\
            Status: {}\n\
            Reply Probability: {}%\n\n\
            <i>System Prompt:</i>\n<code>{}</code>",
            account.phone_number,
            account.id,
            status,
            account.reply_probability,
            account.system_prompt
        );
        
        bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(account_control_keyboard(account_id, is_running))
            .await?;
    }
    
    Ok(())
}
