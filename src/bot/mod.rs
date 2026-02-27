pub mod dialogues;
pub mod handlers;
pub mod middleware;
pub mod group_commands;

use crate::AppState;
use anyhow::Result;
use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateFilterExt},
    prelude::*,
    types::Update,
};

pub use dialogues::{AddAccountDialogue, AddAccountState};

/// Start the admin bot
pub async fn run_admin_bot(state: AppState) -> Result<()> {
    tracing::info!("Starting admin bot...");

    let bot = Bot::new(&state.config.bot_token);

    // Create dialogue storage
    let storage = InMemStorage::<AddAccountState>::new();

    // Build the dispatcher with owner filter
    let handler = Update::filter_message()
        .branch(
            dptree::filter(move |msg: Message, state: AppState| {
                msg.from()
                    .map(|user| state.config.is_owner(user.id.0 as i64))
                    .unwrap_or(false)
            })
            .branch(
                dptree::entry()
                    .filter_command::<handlers::Command>()
                    .endpoint(handlers::handle_command),
            )
            .branch(
                dptree::case![AddAccountState::ReceivePhone]
                    .endpoint(dialogues::receive_phone),
            )
            .branch(
                dptree::case![AddAccountState::ReceiveAuthCode { phone, client, worker }]
                    .endpoint(dialogues::receive_auth_code),
            )
            .branch(
                dptree::case![AddAccountState::Receive2FA { phone, client, worker }]
                    .endpoint(dialogues::receive_2fa),
            )
            .branch(
                dptree::case![AddAccountState::ReceivePrompt { account_id }]
                    .endpoint(dialogues::receive_prompt),
            ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state, storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
