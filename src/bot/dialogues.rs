use crate::{
    db::{AccountRepository, NewAccount},
    userbot,
    AppState,
};
use anyhow::Result;
use rust_tdlib::{
    client::{tdlib_client::TdJson, Client, ConsoleAuthStateHandler, Worker},
    types::{
        AuthorizationState, CheckAuthenticationCode, CheckAuthenticationPassword,
        GetAuthorizationState, SetAuthenticationPhoneNumber, TdlibParameters,
    },
};
use std::sync::Arc;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
use tokio::sync::Mutex;

type TdClient = Client<TdJson>;
type TdWorker = Worker<ConsoleAuthStateHandler, TdJson>;

pub type AddAccountDialogue = Dialogue<AddAccountState, InMemStorage<AddAccountState>>;

#[derive(Clone)]
pub enum AddAccountState {
    ReceivePhone,
    ReceiveAuthCode {
        phone: String,
        client: TdClient,
        worker: Arc<Mutex<TdWorker>>,
    },
    Receive2FA {
        phone: String,
        client: TdClient,
        worker: Arc<Mutex<TdWorker>>,
    },
    ReceivePrompt {
        account_id: i64,
    },
}

impl Default for AddAccountState {
    fn default() -> Self {
        Self::ReceivePhone
    }
}

pub async fn receive_phone(
    bot: Bot,
    msg: Message,
    dialogue: AddAccountDialogue,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match msg.text() {
        Some(t) => t.trim(),
        None => {
            bot.send_message(msg.chat.id, "❌ Please send a text message with the phone number.")
                .await?;
            return Ok(());
        }
    };

    if text == "/cancel" {
        dialogue.exit().await?;
        bot.send_message(msg.chat.id, "❌ Operation cancelled.").await?;
        return Ok(());
    }

    if !text.starts_with('+') || text.len() < 10 {
        bot.send_message(
            msg.chat.id,
            "❌ Invalid phone format. Please use international format (e.g., +1234567890).",
        )
        .await?;
        return Ok(());
    }

    let phone = text.to_string();

    if AccountRepository::get_by_phone(&state.db_pool, &phone).await?.is_some() {
        bot.send_message(
            msg.chat.id,
            format!("❌ Account with phone {} already exists.", phone),
        )
        .await?;
        dialogue.exit().await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, "⏳ Connecting to Telegram...").await?;

    let (client, worker) = match create_tdlib_client(&state, &phone).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create tdlib client: {}", e);
            bot.send_message(
                msg.chat.id,
                format!("❌ Failed to connect to Telegram: {}", e),
            )
            .await?;
            dialogue.exit().await?;
            return Ok(());
        }
    };

    let set_phone = SetAuthenticationPhoneNumber::builder()
        .phone_number(phone.clone())
        .build();
    
    if let Err(e) = client.set_authentication_phone_number(&set_phone).await {
        tracing::error!("Failed to send phone number: {}", e);
        bot.send_message(
            msg.chat.id,
            format!("❌ Failed to request login code: {}", e),
        )
        .await?;
        dialogue.exit().await?;
        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Login code sent to {}.\n\nPlease send the code you received (e.g., 12345).\nSend /cancel to abort.",
            phone
        ),
    )
    .await?;

    dialogue
        .update(AddAccountState::ReceiveAuthCode { phone, client, worker })
        .await?;

    Ok(())
}

pub async fn receive_auth_code(
    bot: Bot,
    msg: Message,
    dialogue: AddAccountDialogue,
    state: AppState,
    (phone, client, worker): (String, TdClient, Arc<Mutex<TdWorker>>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match msg.text() {
        Some(t) => t.trim(),
        None => {
            bot.send_message(msg.chat.id, "❌ Please send the auth code as text.")
                .await?;
            return Ok(());
        }
    };

    if text == "/cancel" {
        dialogue.exit().await?;
        bot.send_message(msg.chat.id, "❌ Operation cancelled.").await?;
        return Ok(());
    }

    let code = text.to_string();
    bot.send_message(msg.chat.id, "⏳ Verifying code...").await?;

    let check_code = CheckAuthenticationCode::builder().code(code).build();
    
    if let Err(e) = client.check_authentication_code(&check_code).await {
        tracing::error!("Failed to check auth code: {}", e);
        bot.send_message(msg.chat.id, format!("❌ Invalid code: {}", e))
            .await?;
        dialogue.exit().await?;
        return Ok(());
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let auth_state = client.get_authorization_state(&GetAuthorizationState::builder().build()).await?;

    match auth_state {
        AuthorizationState::WaitPassword(_) => {
            bot.send_message(
                msg.chat.id,
                "🔐 Two-factor authentication is enabled.\n\nPlease send your 2FA password.\nSend /cancel to abort.",
            )
            .await?;

            dialogue
                .update(AddAccountState::Receive2FA { phone, client, worker })
                .await?;
        }
        AuthorizationState::Ready(_) => {
            if let Err(e) = finalize_account(&bot, &msg, &dialogue, &state, phone, &client, &worker).await {
                tracing::error!("Failed to finalize account: {}", e);
                bot.send_message(msg.chat.id, format!("❌ Failed to save account: {}", e))
                    .await?;
                dialogue.exit().await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "❌ Unexpected authentication state")
                .await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

pub async fn receive_2fa(
    bot: Bot,
    msg: Message,
    dialogue: AddAccountDialogue,
    state: AppState,
    (phone, client, worker): (String, TdClient, Arc<Mutex<TdWorker>>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match msg.text() {
        Some(t) => t,
        None => {
            bot.send_message(msg.chat.id, "❌ Please send the password as text.")
                .await?;
            return Ok(());
        }
    };

    if text == "/cancel" {
        dialogue.exit().await?;
        bot.send_message(msg.chat.id, "❌ Operation cancelled.").await?;
        return Ok(());
    }

    let password = text.to_string();
    bot.send_message(msg.chat.id, "⏳ Verifying password...").await?;

    let check_password = CheckAuthenticationPassword::builder()
        .password(password)
        .build();
    
    if let Err(e) = client.check_authentication_password(&check_password).await {
        tracing::error!("2FA error: {}", e);
        bot.send_message(msg.chat.id, format!("❌ Invalid password: {}", e))
            .await?;
        dialogue.exit().await?;
        return Ok(());
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    if let Err(e) = finalize_account(&bot, &msg, &dialogue, &state, phone, &client, &worker).await {
        tracing::error!("Failed to finalize account: {}", e);
        bot.send_message(msg.chat.id, format!("❌ Failed to save account: {}", e))
            .await?;
        dialogue.exit().await?;
    }

    Ok(())
}

pub async fn receive_prompt(
    bot: Bot,
    msg: Message,
    dialogue: AddAccountDialogue,
    state: AppState,
    account_id: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match msg.text() {
        Some(t) => t.trim(),
        None => {
            bot.send_message(msg.chat.id, "❌ Please send the prompt as text.")
                .await?;
            return Ok(());
        }
    };

    if text == "/cancel" {
        dialogue.exit().await?;
        bot.send_message(msg.chat.id, "❌ Operation cancelled.").await?;
        return Ok(());
    }

    let new_prompt = text.to_string();
    AccountRepository::update_system_prompt(&state.db_pool, account_id, &new_prompt).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ System prompt updated for account {}.", account_id),
    )
    .await?;

    dialogue.exit().await?;
    Ok(())
}

async fn create_tdlib_client(
    state: &AppState,
    phone: &str,
) -> Result<(TdClient, Arc<Mutex<TdWorker>>)> {
    let mut worker = Worker::builder().build()?;
    worker.start();

    let tdlib_params = TdlibParameters::builder()
        .api_id(state.config.telegram_api_id)
        .api_hash(state.config.telegram_api_hash.clone())
        .database_directory(format!("./data/tdlib/{}", phone))
        .use_message_database(true)
        .use_secret_chats(false)
        .system_language_code("en".to_string())
        .device_model("Desktop".to_string())
        .application_version("1.0.0".to_string())
        .build();

    let client = Client::builder()
        .with_tdlib_parameters(tdlib_params)
        .build()?;

    let client = worker.bind_client(client).await?;

    Ok((client, Arc::new(Mutex::new(worker))))
}

async fn finalize_account(
    bot: &Bot,
    msg: &Message,
    dialogue: &AddAccountDialogue,
    state: &AppState,
    phone: String,
    _client: &TdClient,
    _worker: &Arc<Mutex<TdWorker>>,
) -> Result<()> {
    let session_data = vec![0u8];

    let new_account = NewAccount {
        phone_number: phone.clone(),
        session_data,
        system_prompt: get_default_system_prompt(),
    };

    let account = AccountRepository::create(&state.db_pool, new_account).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ <b>Account added successfully!</b>\n\nID: {}\nPhone: {}\n\nStarting userbot...",
            account.id, account.phone_number
        ),
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;

    if let Err(e) = userbot::spawn_userbot(state.clone(), account.id).await {
        tracing::error!("Failed to spawn userbot: {}", e);
        bot.send_message(
            msg.chat.id,
            format!("⚠️ Account saved but failed to start userbot: {}", e),
        )
        .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            format!("🟢 Userbot {} is now running!", account.id),
        )
        .await?;
    }

    dialogue.exit().await?;
    Ok(())
}

fn get_default_system_prompt() -> String {
    r#"[КОНТЕКСТ]
Ты участвуешь в неформальной переписке в Telegram. Ты обычный человек со своей жизнью, рутиной и интересами. Твоя задача — поддерживать диалог в рамках своего характера.

[ТВОЙ СТИЛЬ ОБЩЕНИЯ]
- Тексты короткие (обычно 1-2 предложения, как в реальном мессенджере).
- Пиши в нижнем регистре, можешь пропускать точки в конце предложений.
- Используй разговорные сокращения (щас, норм, спс, че, пон).
- Эмоции выражай через уместные эмодзи, скобочки ")" или сленг ("ахах", "жиза").
- Если не знаешь ответ на вопрос, так и скажи ("хз вообще", "без понятия").

[ПРАВИЛА]
1. Адаптируйся под тон собеседника.
2. Не предлагай помощь как ассистент. Ты просто общаешься.
3. Если собеседник пишет бессвязный бред, реагируй с недоумением.

[ПРИМЕРЫ]
User: привет, пойдешь сегодня гулять?
Assistant: ку. не, я сегодня пас, дел много(

User: *скидывает смешной мем*
Assistant: ахахах жиза

User: напиши код на питоне для калькулятора
Assistant: эээ ты тейком ошибся походу, я не прогер"#.to_string()
}
