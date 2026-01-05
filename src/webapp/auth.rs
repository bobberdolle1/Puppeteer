use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

/// Validate Telegram WebApp initData
/// https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
pub fn validate_init_data(init_data: &str, bot_token: &str) -> Option<TelegramUser> {
    let params: HashMap<String, String> = init_data
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            Some((
                urlencoding::decode(parts.next()?).ok()?.into_owned(),
                urlencoding::decode(parts.next()?).ok()?.into_owned(),
            ))
        })
        .collect();

    let hash = params.get("hash")?;
    
    // Build data-check-string
    let mut check_pairs: Vec<_> = params
        .iter()
        .filter(|(k, _)| *k != "hash")
        .collect();
    check_pairs.sort_by(|a, b| a.0.cmp(b.0));
    
    let data_check_string: String = check_pairs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    // Calculate secret key: HMAC-SHA256(bot_token, "WebAppData")
    let mut secret_mac = HmacSha256::new_from_slice(b"WebAppData").ok()?;
    secret_mac.update(bot_token.as_bytes());
    let secret_key = secret_mac.finalize().into_bytes();

    // Calculate hash: HMAC-SHA256(data_check_string, secret_key)
    let mut mac = HmacSha256::new_from_slice(&secret_key).ok()?;
    mac.update(data_check_string.as_bytes());
    let calculated_hash = hex::encode(mac.finalize().into_bytes());

    if calculated_hash != *hash {
        log::warn!("Invalid WebApp hash");
        return None;
    }

    // Check auth_date (not older than 24 hours)
    if let Some(auth_date) = params.get("auth_date") {
        if let Ok(timestamp) = auth_date.parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            if now - timestamp > 86400 {
                log::warn!("WebApp auth_date too old");
                return None;
            }
        }
    }

    // Parse user data
    let user_json = params.get("user")?;
    serde_json::from_str(user_json).ok()
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TelegramUser {
    pub id: u64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub language_code: Option<String>,
    pub is_premium: Option<bool>,
}
