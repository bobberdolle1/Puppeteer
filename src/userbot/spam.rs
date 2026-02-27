use crate::{
    db::{BotGroupRepository, SpamCampaignRepository, SpamCampaign},
    state::AppState,
};
use anyhow::{Context, Result};
use rust_tdlib::{
    client::tdlib_client::TdJson,
    types::*,
};
use std::sync::Arc;
use tokio::sync::Mutex;

type TdClient = rust_tdlib::client::Client<TdJson>;

/// Execute a spam campaign
pub async fn execute_spam_campaign(
    state: &AppState,
    campaign: &SpamCampaign,
) -> Result<()> {
    tracing::info!("Starting spam campaign: {}", campaign.name);

    // Update status to running
    SpamCampaignRepository::update_status(&state.db_pool, campaign.id, "running").await?;

    // Get accounts to use
    let accounts = if let Some(group_id) = campaign.group_id {
        // Use bot group
        BotGroupRepository::get_members(&state.db_pool, group_id).await?
    } else {
        // Use all active accounts
        crate::db::AccountRepository::list_active(&state.db_pool).await?
    };

    if accounts.is_empty() {
        tracing::warn!("No accounts available for spam campaign");
        SpamCampaignRepository::update_status(&state.db_pool, campaign.id, "completed").await?;
        return Ok(());
    }

    tracing::info!("Using {} accounts for spam campaign", accounts.len());

    // Execute campaign
    for repeat in 0..campaign.repeat_count {
        tracing::info!("Spam campaign iteration {}/{}", repeat + 1, campaign.repeat_count);

        for account in &accounts {
            // Get userbot handle
            let handle = match state.get_userbot(account.id).await {
                Some(h) => h,
                None => {
                    tracing::warn!("Userbot {} not running, skipping", account.id);
                    continue;
                }
            };

            // Send message
            if let Err(e) = send_spam_message(&handle.client, campaign).await {
                tracing::error!("Failed to send spam message from account {}: {}", account.id, e);
                continue;
            }

            // Delay between messages from same account
            tokio::time::sleep(tokio::time::Duration::from_millis(campaign.delay_between_ms as u64)).await;
        }

        // Delay between iterations
        if repeat < campaign.repeat_count - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(campaign.delay_between_ms as u64 * 2)).await;
        }
    }

    // Update status to completed
    SpamCampaignRepository::update_status(&state.db_pool, campaign.id, "completed").await?;

    tracing::info!("Completed spam campaign: {}", campaign.name);
    Ok(())
}

/// Send a single spam message
async fn send_spam_message(
    client: &Arc<Mutex<TdClient>>,
    campaign: &SpamCampaign,
) -> Result<()> {
    let client_lock = client.lock().await;

    // Prepare message content
    let input_message = if let Some(media_path) = &campaign.media_path {
        // Send media
        match campaign.media_type.as_deref() {
            Some("photo") => {
                let photo = InputFileLocal::builder()
                    .path(media_path.clone())
                    .build();
                
                let caption = campaign.message_text.as_ref()
                    .map(|text| FormattedText::builder().text(text.clone()).build());

                InputMessageContent::InputMessagePhoto(
                    InputMessagePhoto::builder()
                        .photo(InputFile::Local(photo))
                        .caption(caption.unwrap_or_default())
                        .build()
                )
            }
            Some("video") => {
                let video = InputFileLocal::builder()
                    .path(media_path.clone())
                    .build();
                
                let caption = campaign.message_text.as_ref()
                    .map(|text| FormattedText::builder().text(text.clone()).build());

                InputMessageContent::InputMessageVideo(
                    InputMessageVideo::builder()
                        .video(InputFile::Local(video))
                        .caption(caption.unwrap_or_default())
                        .build()
                )
            }
            Some("gif") => {
                let animation = InputFileLocal::builder()
                    .path(media_path.clone())
                    .build();
                
                let caption = campaign.message_text.as_ref()
                    .map(|text| FormattedText::builder().text(text.clone()).build());

                InputMessageContent::InputMessageAnimation(
                    InputMessageAnimation::builder()
                        .animation(InputFile::Local(animation))
                        .caption(caption.unwrap_or_default())
                        .build()
                )
            }
            Some("document") => {
                let document = InputFileLocal::builder()
                    .path(media_path.clone())
                    .build();
                
                let caption = campaign.message_text.as_ref()
                    .map(|text| FormattedText::builder().text(text.clone()).build());

                InputMessageContent::InputMessageDocument(
                    InputMessageDocument::builder()
                        .document(InputFile::Local(document))
                        .caption(caption.unwrap_or_default())
                        .build()
                )
            }
            _ => {
                // Fallback to text
                let text = campaign.message_text.as_deref().unwrap_or("");
                InputMessageContent::InputMessageText(
                    InputMessageText::builder()
                        .text(FormattedText::builder().text(text.to_string()).build())
                        .build()
                )
            }
        }
    } else {
        // Send text only
        let text = campaign.message_text.as_deref().unwrap_or("");
        InputMessageContent::InputMessageText(
            InputMessageText::builder()
                .text(FormattedText::builder().text(text.to_string()).build())
                .build()
        )
    };

    // Send message
    let send_message = SendMessage::builder()
        .chat_id(campaign.target_id)
        .input_message_content(input_message)
        .build();

    client_lock.send_message(&send_message).await
        .context("Failed to send spam message")?;

    drop(client_lock);
    Ok(())
}

/// Monitor and execute pending spam campaigns
pub async fn spam_campaign_worker(state: AppState) {
    tracing::info!("Spam campaign worker started");

    loop {
        // Check for pending campaigns every 5 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Get pending campaigns
        let campaigns = match SpamCampaignRepository::list_pending(&state.db_pool).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to fetch pending campaigns: {}", e);
                continue;
            }
        };

        // Execute each campaign
        for campaign in campaigns {
            let state_clone = state.clone();
            let campaign_clone = campaign.clone();
            
            tokio::spawn(async move {
                if let Err(e) = execute_spam_campaign(&state_clone, &campaign_clone).await {
                    tracing::error!("Spam campaign {} failed: {}", campaign_clone.id, e);
                    
                    // Update status to stopped on error
                    if let Err(update_err) = SpamCampaignRepository::update_status(
                        &state_clone.db_pool,
                        campaign_clone.id,
                        "stopped"
                    ).await {
                        tracing::error!("Failed to update campaign status: {}", update_err);
                    }
                }
            });
        }
    }
}
