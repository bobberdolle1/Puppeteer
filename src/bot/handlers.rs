use crate::{
    bot::{AddAccountDialogue, AddAccountState},
    db::{AccountRepository, MessageRepository},
    AppState,
};
use anyhow::Result;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Admin commands:")]
pub enum Command {
    #[command(description = "Show bot status and statistics")]
    Start,
    #[command(description = "Add a new MTProto userbot account")]
    AddAccount,
    #[command(description = "List all connected accounts")]
    List,
    #[command(description = "Update system prompt for an account (usage: /set_prompt <id>)")]
    SetPrompt,
    #[command(description = "Set reply probability 0-100 (usage: /set_prob <id> <0-100>)")]
    SetProb,
    #[command(description = "Add chat to whitelist (usage: /allow_chat <id> <chat_id>)")]
    AllowChat,
    #[command(description = "Remove chat from whitelist (usage: /remove_chat <id> <chat_id>)")]
    RemoveChat,
    #[command(description = "Stop a running userbot (usage: /stop <id>)")]
    Stop,
    #[command(description = "Delete an account from database (usage: /delete <id>)")]
    Delete,
    #[command(description = "Show help message")]
    Help,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: AppState,
    dialogue: AddAccountDialogue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        Command::Start => handle_start(bot, msg, state).await?,
        Command::AddAccount => handle_add_account(bot, msg, dialogue).await?,
        Command::List => handle_list(bot, msg, state).await?,
        Command::SetPrompt => handle_set_prompt(bot, msg, state, dialogue).await?,
        Command::SetProb => handle_set_prob(bot, msg, state).await?,
        Command::AllowChat => handle_allow_chat(bot, msg, state).await?,
        Command::RemoveChat => handle_remove_chat(bot, msg, state).await?,
        Command::Stop => handle_stop(bot, msg, state).await?,
        Command::Delete => handle_delete(bot, msg, state).await?,
        Command::Help => handle_help(bot, msg).await?,
    }
    Ok(())
}

async fn handle_start(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let active_count = state.active_userbot_count().await;
    let all_accounts = AccountRepository::list_all(&state.db_pool).await?;
    
    let mut total_messages = 0i64;
    for account in &all_accounts {
        total_messages += MessageRepository::count_by_account(&state.db_pool, account.id).await?;
    }

    let status_text = format!(
        "🤖 <b>Puppeteer Admin Bot</b>\n\n\
        📊 <b>Statistics:</b>\n\
        • Active Userbots: {}\n\
        • Total Accounts: {}\n\
        • Total Messages: {}\n\n\
        Use /help to see available commands.",
        active_count,
        all_accounts.len(),
        total_messages
    );

    bot.send_message(msg.chat.id, status_text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}

async fn handle_add_account(
    bot: Bot,
    msg: Message,
    dialogue: AddAccountDialogue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.send_message(
        msg.chat.id,
        "📱 <b>Add New Userbot Account</b>\n\n\
        Please send the phone number in international format (e.g., +1234567890).\n\n\
        Send /cancel to abort.",
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;

    dialogue.update(AddAccountState::ReceivePhone).await?;
    Ok(())
}

async fn handle_list(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let accounts = AccountRepository::list_all(&state.db_pool).await?;

    if accounts.is_empty() {
        bot.send_message(msg.chat.id, "No accounts found. Use /add_account to add one.")
            .await?;
        return Ok(());
    }

    let mut response = String::from("📋 <b>Registered Accounts:</b>\n\n");

    for account in accounts {
        let is_running = state.is_userbot_running(account.id).await;
        let status_emoji = if is_running { "🟢" } else { "🔴" };
        let active_text = if account.is_active { "Active" } else { "Inactive" };
        
        let msg_count = MessageRepository::count_by_account(&state.db_pool, account.id).await?;
        let allowed_chats = account.get_allowed_chats();
        let chats_text = if allowed_chats.is_empty() {
            "All".to_string()
        } else {
            format!("{} chats", allowed_chats.len())
        };
        
        response.push_str(&format!(
            "{} <b>ID:</b> {} | <b>Phone:</b> {}\n\
            └ Status: {} | Prob: {}% | Chats: {} | Msgs: {}\n\n",
            status_emoji,
            account.id,
            account.phone_number,
            active_text,
            account.reply_probability,
            chats_text,
            msg_count
        ));
    }

    bot.send_message(msg.chat.id, response)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}

async fn handle_set_prompt(
    bot: Bot,
    msg: Message,
    state: AppState,
    dialogue: AddAccountDialogue,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse account ID from message text
    let text = msg.text().unwrap_or("");
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() < 2 {
        bot.send_message(msg.chat.id, "❌ Usage: /set_prompt <account_id>")
            .await?;
        return Ok(());
    }
    
    let account_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid account ID. Usage: /set_prompt <id>")
                .await?;
            return Ok(());
        }
    };

    // Check if account exists
    match AccountRepository::get_by_id(&state.db_pool, account_id).await? {
        Some(account) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "✏️ <b>Update System Prompt</b>\n\n\
                    Account: {} ({})\n\
                    Current prompt: <i>{}</i>\n\n\
                    Please send the new system prompt.\n\
                    Send /cancel to abort.",
                    account.id,
                    account.phone_number,
                    account.system_prompt
                ),
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;

            dialogue.update(AddAccountState::ReceivePrompt { account_id }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, format!("❌ Account {} not found.", account_id))
                .await?;
        }
    }

    Ok(())
}

async fn handle_set_prob(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse arguments from message text
    let text = msg.text().unwrap_or("");
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() < 3 {
        bot.send_message(msg.chat.id, "❌ Usage: /set_prob <account_id> <0-100>")
            .await?;
        return Ok(());
    }
    
    let account_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid account ID. Usage: /set_prob <id> <0-100>")
                .await?;
            return Ok(());
        }
    };

    let probability = match parts[2].parse::<i64>() {
        Ok(p) if (0..=100).contains(&p) => p,
        _ => {
            bot.send_message(msg.chat.id, "❌ Probability must be between 0 and 100")
                .await?;
            return Ok(());
        }
    };

    // Check if account exists
    if AccountRepository::get_by_id(&state.db_pool, account_id).await?.is_none() {
        bot.send_message(msg.chat.id, format!("❌ Account {} not found.", account_id))
            .await?;
        return Ok(());
    }

    AccountRepository::update_reply_probability(&state.db_pool, account_id, probability).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Reply probability for account {} set to {}%", account_id, probability),
    )
    .await?;

    Ok(())
}

async fn handle_allow_chat(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse arguments from message text
    let text = msg.text().unwrap_or("");
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() < 3 {
        bot.send_message(msg.chat.id, "❌ Usage: /allow_chat <account_id> <chat_id>")
            .await?;
        return Ok(());
    }
    
    let account_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid account ID. Usage: /allow_chat <id> <chat_id>")
                .await?;
            return Ok(());
        }
    };

    let chat_id_num = match parts[2].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid chat ID")
                .await?;
            return Ok(());
        }
    };

    AccountRepository::add_allowed_chat(&state.db_pool, account_id, chat_id_num).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Chat {} added to whitelist for account {}", chat_id_num, account_id),
    )
    .await?;

    Ok(())
}

async fn handle_remove_chat(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse arguments from message text
    let text = msg.text().unwrap_or("");
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() < 3 {
        bot.send_message(msg.chat.id, "❌ Usage: /remove_chat <account_id> <chat_id>")
            .await?;
        return Ok(());
    }
    
    let account_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid account ID. Usage: /remove_chat <id> <chat_id>")
                .await?;
            return Ok(());
        }
    };

    let chat_id_num = match parts[2].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid chat ID")
                .await?;
            return Ok(());
        }
    };

    AccountRepository::remove_allowed_chat(&state.db_pool, account_id, chat_id_num).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Chat {} removed from whitelist for account {}", chat_id_num, account_id),
    )
    .await?;

    Ok(())
}

async fn handle_stop(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse account ID from message text
    let text = msg.text().unwrap_or("");
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() < 2 {
        bot.send_message(msg.chat.id, "❌ Usage: /stop <account_id>")
            .await?;
        return Ok(());
    }
    
    let account_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid account ID. Usage: /stop <id>")
                .await?;
            return Ok(());
        }
    };

    if !state.is_userbot_running(account_id).await {
        bot.send_message(msg.chat.id, format!("⚠️ Userbot {} is not running.", account_id))
            .await?;
        return Ok(());
    }

    state.shutdown_userbot(account_id).await?;
    AccountRepository::set_active(&state.db_pool, account_id, false).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Userbot {} stopped successfully.", account_id),
    )
    .await?;

    Ok(())
}

async fn handle_delete(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse account ID from message text
    let text = msg.text().unwrap_or("");
    let parts: Vec<&str> = text.split_whitespace().collect();
    
    if parts.len() < 2 {
        bot.send_message(msg.chat.id, "❌ Usage: /delete <account_id>")
            .await?;
        return Ok(());
    }
    
    let account_id = match parts[1].parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Invalid account ID. Usage: /delete <id>")
                .await?;
            return Ok(());
        }
    };

    // Check if account exists
    let account = match AccountRepository::get_by_id(&state.db_pool, account_id).await? {
        Some(acc) => acc,
        None => {
            bot.send_message(msg.chat.id, format!("❌ Account {} not found.", account_id))
                .await?;
            return Ok(());
        }
    };

    // Stop if running
    if state.is_userbot_running(account_id).await {
        state.shutdown_userbot(account_id).await?;
    }

    // Delete from database
    AccountRepository::delete(&state.db_pool, account_id).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Account {} ({}) deleted successfully.",
            account_id, account.phone_number
        ),
    )
    .await?;

    Ok(())
}

async fn handle_help(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let help_text = format!(
        "<b>🤖 Puppeteer Admin Bot</b>\n\n\
        <b>Available Commands:</b>\n\
        {}\n\n\
        <b>About:</b>\n\
        Puppeteer manages multiple AI-driven Telegram userbots with human-like behavior. \
        Each userbot can have its own personality (system prompt) and maintains conversation context.",
        Command::descriptions()
    );

    bot.send_message(msg.chat.id, help_text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}
