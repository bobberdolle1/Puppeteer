pub mod worker;
pub mod spam;

pub use worker::{spawn_userbot, DEFAULT_SYSTEM_PROMPT};
pub use spam::{execute_spam_campaign, spam_campaign_worker};
