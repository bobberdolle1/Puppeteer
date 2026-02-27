use crate::{
    db::{BotGroupRepository, SpamCampaignRepository, NewBotGroup, NewSpamCampaign},
    AppState,
};
use anyhow::Result;
use teloxide::{prelude::*, types::InputFile};

/// Create a new bot group
/// Usage: /create_group <name> [description]
pub async fn handle_create_group(
    bot: Bot,
    msg: Message,
    state: AppState,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.is_empty() {
        bot.send_message(msg.chat.id, "❌ Usage: /create_group <name> [description]")
            .await?;
        return Ok(());
    }

    let name = args[0].clone();
    let description = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    let new_group = NewBotGroup { name, description };
    let group = BotGroupRepository::create(&state.db_pool, new_group).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Created bot group '{}' with ID {}", group.name, group.id),
    )
    .await?;

    Ok(())
}

/// List all bot groups
pub async fn handle_list_groups(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let groups = BotGroupRepository::list_all(&state.db_pool).await?;

    if groups.is_empty() {
        bot.send_message(msg.chat.id, "📋 No bot groups found.").await?;
        return Ok(());
    }

    let mut text = String::from("📋 <b>Bot Groups:</b>\n\n");
    
    for group in groups {
        let members = BotGroupRepository::get_members(&state.db_pool, group.id).await?;
        text.push_str(&format!(
            "🔹 <b>{}</b> (ID: {})\n",
            group.name, group.id
        ));
        if let Some(desc) = &group.description {
            text.push_str(&format!("   {}\n", desc));
        }
        text.push_str(&format!("   Members: {}\n\n", members.len()));
    }

    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}

/// Add account to bot group
/// Usage: /add_to_group <group_id> <account_id>
pub async fn handle_add_to_group(
    bot: Bot,
    msg: Message,
    state: AppState,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.len() < 2 {
        bot.send_message(msg.chat.id, "❌ Usage: /add_to_group <group_id> <account_id>")
            .await?;
        return Ok(());
    }

    let group_id: i64 = args[0].parse()?;
    let account_id: i64 = args[1].parse()?;

    BotGroupRepository::add_member(&state.db_pool, group_id, account_id).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Added account {} to group {}", account_id, group_id),
    )
    .await?;

    Ok(())
}

/// Create spam campaign
/// Usage: /spam <group_id|all> <target_type> <target_id> <repeat> <delay_ms> <text>
pub async fn handle_create_spam(
    bot: Bot,
    msg: Message,
    state: AppState,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.len() < 6 {
        bot.send_message(
            msg.chat.id,
            "❌ Usage: /spam <group_id|all> <target_type> <target_id> <repeat> <delay_ms> <text>\n\n\
            Example: /spam 1 chat -1001234567890 5 1000 Hello from bots!",
        )
        .await?;
        return Ok(());
    }

    let group_id = if args[0] == "all" {
        None
    } else {
        Some(args[0].parse::<i64>()?)
    };

    let target_type = args[1].clone();
    let target_id: i64 = args[2].parse()?;
    let repeat_count: i64 = args[3].parse()?;
    let delay_between_ms: i64 = args[4].parse()?;
    let message_text = Some(args[5..].join(" "));

    let new_campaign = NewSpamCampaign {
        name: format!("Campaign_{}", chrono::Utc::now().timestamp()),
        group_id,
        target_type,
        target_id,
        message_text,
        media_path: None,
        media_type: None,
        repeat_count,
        delay_between_ms,
    };

    let campaign = SpamCampaignRepository::create(&state.db_pool, new_campaign).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Created spam campaign ID {}\n\
            Target: {}\n\
            Repeats: {}\n\
            Delay: {}ms\n\n\
            Campaign will start automatically.",
            campaign.id, campaign.target_id, campaign.repeat_count, campaign.delay_between_ms
        ),
    )
    .await?;

    Ok(())
}

/// Create spam campaign with media
/// Usage: /spam_media <group_id|all> <target_type> <target_id> <repeat> <delay_ms> <media_type>
/// Then send the media file
pub async fn handle_create_spam_media(
    bot: Bot,
    msg: Message,
    state: AppState,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.len() < 6 {
        bot.send_message(
            msg.chat.id,
            "❌ Usage: /spam_media <group_id|all> <target_type> <target_id> <repeat> <delay_ms> <media_type> [caption]\n\n\
            Media types: photo, video, gif, document\n\
            After sending this command, send the media file.",
        )
        .await?;
        return Ok(());
    }

    // Store campaign parameters in state for next message
    // For now, just show instructions
    bot.send_message(
        msg.chat.id,
        "📎 Now send the media file (photo/video/gif/document) with optional caption.",
    )
    .await?;

    Ok(())
}

/// List spam campaigns
pub async fn handle_list_campaigns(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let campaigns = SpamCampaignRepository::list_all(&state.db_pool).await?;

    if campaigns.is_empty() {
        bot.send_message(msg.chat.id, "📋 No spam campaigns found.").await?;
        return Ok(());
    }

    let mut text = String::from("📋 <b>Spam Campaigns:</b>\n\n");
    
    for campaign in campaigns {
        text.push_str(&format!(
            "🔹 <b>{}</b> (ID: {})\n\
            Status: {}\n\
            Target: {}\n\
            Repeats: {}\n\
            Delay: {}ms\n\n",
            campaign.name,
            campaign.id,
            campaign.status,
            campaign.target_id,
            campaign.repeat_count,
            campaign.delay_between_ms
        ));
    }

    bot.send_message(msg.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}

/// Stop a running spam campaign
/// Usage: /stop_campaign <campaign_id>
pub async fn handle_stop_campaign(
    bot: Bot,
    msg: Message,
    state: AppState,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.is_empty() {
        bot.send_message(msg.chat.id, "❌ Usage: /stop_campaign <campaign_id>")
            .await?;
        return Ok(());
    }

    let campaign_id: i64 = args[0].parse()?;
    SpamCampaignRepository::update_status(&state.db_pool, campaign_id, "stopped").await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Stopped campaign {}", campaign_id),
    )
    .await?;

    Ok(())
}

/// Send a message to a user from a specific bot
/// Usage: /dm <account_id> <user_id> <text>
pub async fn handle_dm(
    bot: Bot,
    msg: Message,
    state: AppState,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.len() < 3 {
        bot.send_message(msg.chat.id, "❌ Usage: /dm <account_id> <user_id> <text>")
            .await?;
        return Ok(());
    }

    let account_id: i64 = args[0].parse()?;
    let user_id: i64 = args[1].parse()?;
    let text = args[2..].join(" ");

    // Get userbot handle
    let handle = match state.get_userbot(account_id).await {
        Some(h) => h,
        None => {
            bot.send_message(msg.chat.id, "❌ Userbot not running").await?;
            return Ok(());
        }
    };

    // Send message
    use rust_tdlib::types::*;
    let client_lock = handle.client.lock().await;
    
    let input_message = InputMessageContent::InputMessageText(
        InputMessageText::builder()
            .text(FormattedText::builder().text(text.clone()).build())
            .build()
    );

    let send_message = SendMessage::builder()
        .chat_id(user_id)
        .input_message_content(input_message)
        .build();

    match client_lock.send_message(&send_message).await {
        Ok(_) => {
            bot.send_message(msg.chat.id, "✅ Message sent").await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed: {}", e)).await?;
        }
    }

    Ok(())
}
