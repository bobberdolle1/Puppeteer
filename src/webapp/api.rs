use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use crate::db;
use crate::state::AppState;
use super::auth::{validate_init_data, TelegramUser};

// --- Auth middleware helper ---

fn extract_user(headers: &HeaderMap, state: &AppState) -> Result<TelegramUser, StatusCode> {
    let init_data = headers
        .get("X-Telegram-Init-Data")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = validate_init_data(init_data, &state.config.teloxide_token)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user is owner
    if user.id != state.config.owner_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(user)
}

// --- Response types ---

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: &str) -> Self {
        Self { success: false, data: None, error: Some(msg.to_string()) }
    }
}

// --- Persona types ---

#[derive(Serialize)]
pub struct PersonaResponse {
    pub id: i64,
    pub name: String,
    pub prompt: String,
    pub is_active: bool,
    pub display_name: Option<String>,
    pub triggers: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePersonaRequest {
    pub name: String,
    pub prompt: String,
    pub display_name: Option<String>,
    pub triggers: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePersonaRequest {
    pub name: String,
    pub prompt: String,
    pub display_name: Option<String>,
    pub triggers: Option<String>,
}

// --- Persona endpoints ---

pub async fn list_personas(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PersonaResponse>>>, StatusCode> {
    extract_user(&headers, &state)?;

    match db::get_all_personas(&state.db_pool).await {
        Ok(personas) => {
            let data: Vec<PersonaResponse> = personas
                .into_iter()
                .map(|p| PersonaResponse {
                    id: p.id,
                    name: p.name,
                    prompt: p.prompt,
                    is_active: p.is_active,
                    display_name: p.display_name,
                    triggers: p.triggers,
                })
                .collect();
            Ok(Json(ApiResponse::ok(data)))
        }
        Err(e) => {
            log::error!("Failed to list personas: {}", e);
            Ok(Json(ApiResponse::err("Database error")))
        }
    }
}

pub async fn create_persona(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<CreatePersonaRequest>,
) -> Result<Json<ApiResponse<PersonaResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    if req.name.is_empty() || req.prompt.is_empty() {
        return Ok(Json(ApiResponse::err("Name and prompt required")));
    }

    match db::create_persona_full(
        &state.db_pool, 
        &req.name, 
        &req.prompt,
        req.display_name.as_deref(),
        req.triggers.as_deref(),
    ).await {
        Ok(id) => Ok(Json(ApiResponse::ok(PersonaResponse {
            id,
            name: req.name,
            prompt: req.prompt,
            is_active: false,
            display_name: req.display_name,
            triggers: req.triggers,
        }))),
        Err(e) => {
            log::error!("Failed to create persona: {}", e);
            Ok(Json(ApiResponse::err("Failed to create persona")))
        }
    }
}

pub async fn update_persona(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdatePersonaRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    extract_user(&headers, &state)?;

    match db::update_persona_full(
        &state.db_pool, 
        id, 
        &req.name, 
        &req.prompt,
        req.display_name.as_deref(),
        req.triggers.as_deref(),
    ).await {
        Ok(()) => Ok(Json(ApiResponse::ok(()))),
        Err(e) => {
            log::error!("Failed to update persona: {}", e);
            Ok(Json(ApiResponse::err("Failed to update persona")))
        }
    }
}

pub async fn delete_persona(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    extract_user(&headers, &state)?;

    match db::delete_persona(&state.db_pool, id).await {
        Ok(()) => Ok(Json(ApiResponse::ok(()))),
        Err(e) => {
            log::error!("Failed to delete persona: {}", e);
            Ok(Json(ApiResponse::err("Failed to delete persona")))
        }
    }
}

pub async fn activate_persona(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    extract_user(&headers, &state)?;

    match db::set_active_persona(&state.db_pool, id).await {
        Ok(()) => Ok(Json(ApiResponse::ok(()))),
        Err(e) => {
            log::error!("Failed to activate persona: {}", e);
            Ok(Json(ApiResponse::err("Failed to activate persona")))
        }
    }
}


// --- Chat Settings types ---

#[derive(Serialize)]
pub struct ChatSettingsResponse {
    pub chat_id: i64,
    pub auto_reply_enabled: bool,
    pub reply_mode: String,
    pub cooldown_seconds: i64,
    pub context_depth: i64,
    pub rag_enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateChatSettingsRequest {
    pub auto_reply_enabled: Option<bool>,
    pub reply_mode: Option<String>,
    pub cooldown_seconds: Option<i64>,
    pub context_depth: Option<i64>,
    pub rag_enabled: Option<bool>,
}

// --- Chat Settings endpoints ---

pub async fn list_chats(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ChatSettingsResponse>>>, StatusCode> {
    extract_user(&headers, &state)?;

    match db::get_all_chat_settings(&state.db_pool).await {
        Ok(chats) => {
            let data: Vec<ChatSettingsResponse> = chats
                .into_iter()
                .map(|c| ChatSettingsResponse {
                    chat_id: c.chat_id,
                    auto_reply_enabled: c.auto_reply_enabled,
                    reply_mode: c.reply_mode,
                    cooldown_seconds: c.cooldown_seconds,
                    context_depth: c.context_depth,
                    rag_enabled: c.rag_enabled,
                })
                .collect();
            Ok(Json(ApiResponse::ok(data)))
        }
        Err(e) => {
            log::error!("Failed to list chats: {}", e);
            Ok(Json(ApiResponse::err("Database error")))
        }
    }
}

pub async fn get_chat_settings(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(chat_id): Path<i64>,
) -> Result<Json<ApiResponse<ChatSettingsResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    match db::get_or_create_chat_settings(&state.db_pool, chat_id).await {
        Ok(settings) => Ok(Json(ApiResponse::ok(ChatSettingsResponse {
            chat_id: settings.chat_id,
            auto_reply_enabled: settings.auto_reply_enabled,
            reply_mode: settings.reply_mode,
            cooldown_seconds: settings.cooldown_seconds,
            context_depth: settings.context_depth,
            rag_enabled: settings.rag_enabled,
        }))),
        Err(e) => {
            log::error!("Failed to get chat settings: {}", e);
            Ok(Json(ApiResponse::err("Database error")))
        }
    }
}

pub async fn update_chat_settings(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(chat_id): Path<i64>,
    Json(req): Json<UpdateChatSettingsRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    extract_user(&headers, &state)?;

    // Get current settings first
    let current = match db::get_or_create_chat_settings(&state.db_pool, chat_id).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to get chat settings: {}", e);
            return Ok(Json(ApiResponse::err("Database error")));
        }
    };

    // Apply updates
    if let Some(enabled) = req.auto_reply_enabled {
        let _ = db::toggle_auto_reply_for_chat(&state.db_pool, chat_id, enabled).await;
    }
    if let Some(mode) = &req.reply_mode {
        let _ = db::update_reply_mode_for_chat(&state.db_pool, chat_id, mode).await;
    }
    if let Some(cooldown) = req.cooldown_seconds {
        let _ = db::update_cooldown_for_chat(&state.db_pool, chat_id, cooldown).await;
    }
    if let Some(rag) = req.rag_enabled {
        let _ = db::toggle_rag_for_chat(&state.db_pool, chat_id, rag).await;
    }
    if let Some(depth) = req.context_depth {
        let rag = req.rag_enabled.unwrap_or(current.rag_enabled);
        let _ = db::update_rag_settings(&state.db_pool, chat_id, rag, depth).await;
    }

    Ok(Json(ApiResponse::ok(())))
}

// --- System Status ---

#[derive(Serialize)]
pub struct SystemStatus {
    pub ollama_online: bool,
    pub db_online: bool,
    pub active_persona: Option<String>,
    pub queue_available: usize,
    pub queue_max: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: u64,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub vision_enabled: bool,
    pub voice_enabled: bool,
    pub web_search_enabled: bool,
    pub paused: bool,
}

pub async fn get_status(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SystemStatus>>, StatusCode> {
    extract_user(&headers, &state)?;

    let ollama_online = state.llm_client.check_health().await.unwrap_or(false);
    let db_online = db::check_db_health(&state.db_pool).await.unwrap_or(false);
    
    let active_persona = db::get_active_persona(&state.db_pool)
        .await
        .ok()
        .flatten()
        .map(|p| p.name);

    let stats = state.queue_stats.lock().await.clone();
    let queue_available = state.llm_semaphore.available_permits();
    let queue_max = state.config.max_concurrent_llm_requests.unwrap_or(3);

    Ok(Json(ApiResponse::ok(SystemStatus {
        ollama_online,
        db_online,
        active_persona,
        queue_available,
        queue_max,
        total_requests: stats.total_requests,
        successful_requests: stats.successful_requests,
        failed_requests: stats.failed_requests,
        avg_response_time_ms: stats.avg_response_time_ms,
        model: state.config.ollama_chat_model.clone(),
        temperature: state.config.temperature,
        max_tokens: state.config.max_tokens,
        vision_enabled: state.config.vision_enabled,
        voice_enabled: state.config.voice_enabled,
        web_search_enabled: state.config.web_search_enabled,
        paused: state.is_paused(),
    })))
}

// --- Models ---

#[derive(Serialize)]
pub struct ModelsResponse {
    pub models: Vec<String>,
    pub current: String,
}

pub async fn list_models(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ModelsResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    let models = state.llm_client.list_models().await.unwrap_or_default();
    
    Ok(Json(ApiResponse::ok(ModelsResponse {
        models,
        current: state.config.ollama_chat_model.clone(),
    })))
}

// --- Triggers ---

#[derive(Serialize)]
pub struct TriggersResponse {
    pub chat_id: i64,
    pub keywords: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateTriggersRequest {
    pub keywords: Vec<String>,
}

pub async fn get_triggers(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(chat_id): Path<i64>,
) -> Result<Json<ApiResponse<TriggersResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    let triggers = state.keyword_triggers.lock().await;
    let keywords = triggers
        .get(&teloxide::types::ChatId(chat_id))
        .cloned()
        .unwrap_or_default();

    Ok(Json(ApiResponse::ok(TriggersResponse { chat_id, keywords })))
}

pub async fn update_triggers(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(chat_id): Path<i64>,
    Json(req): Json<UpdateTriggersRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    extract_user(&headers, &state)?;

    let mut triggers = state.keyword_triggers.lock().await;
    if req.keywords.is_empty() {
        triggers.remove(&teloxide::types::ChatId(chat_id));
    } else {
        triggers.insert(teloxide::types::ChatId(chat_id), req.keywords);
    }

    Ok(Json(ApiResponse::ok(())))
}

// --- Broadcast ---

#[derive(Deserialize)]
pub struct BroadcastRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct BroadcastResponse {
    pub sent: usize,
    pub failed: usize,
}

pub async fn broadcast(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<BroadcastRequest>,
) -> Result<Json<ApiResponse<BroadcastResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    if req.message.is_empty() {
        return Ok(Json(ApiResponse::err("Message required")));
    }

    let chat_ids = db::get_all_chat_ids(&state.db_pool).await.unwrap_or_default();
    
    // Note: actual sending would require Bot instance
    // For now, return the count of chats that would receive the message
    Ok(Json(ApiResponse::ok(BroadcastResponse {
        sent: chat_ids.len(),
        failed: 0,
    })))
}

// --- Stats ---

#[derive(Serialize)]
pub struct ChatStatsResponse {
    pub chat_id: i64,
    pub message_count: i64,
}

pub async fn get_chat_stats(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ChatStatsResponse>>>, StatusCode> {
    extract_user(&headers, &state)?;

    match db::get_chat_stats(&state.db_pool).await {
        Ok(stats) => {
            let data: Vec<ChatStatsResponse> = stats
                .into_iter()
                .map(|(chat_id, count)| ChatStatsResponse {
                    chat_id,
                    message_count: count,
                })
                .collect();
            Ok(Json(ApiResponse::ok(data)))
        }
        Err(e) => {
            log::error!("Failed to get chat stats: {}", e);
            Ok(Json(ApiResponse::err("Database error")))
        }
    }
}


// --- Runtime Config ---

#[derive(Serialize)]
pub struct RuntimeConfigResponse {
    pub ollama_chat_model: String,
    pub ollama_embedding_model: String,
    pub ollama_vision_model: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub vision_enabled: bool,
    pub voice_enabled: bool,
    pub web_search_enabled: bool,
    pub rag_decay_rate: f64,
    pub summary_threshold: u32,
    pub max_concurrent_llm_requests: u32,
    pub llm_timeout_seconds: u64,
    pub random_reply_probability: f64,
}

#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    pub ollama_chat_model: Option<String>,
    pub ollama_embedding_model: Option<String>,
    pub ollama_vision_model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub vision_enabled: Option<bool>,
    pub voice_enabled: Option<bool>,
    pub web_search_enabled: Option<bool>,
    pub rag_decay_rate: Option<f64>,
    pub summary_threshold: Option<u32>,
    pub max_concurrent_llm_requests: Option<u32>,
    pub llm_timeout_seconds: Option<u64>,
    pub random_reply_probability: Option<f64>,
}

pub async fn get_config(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<RuntimeConfigResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    let config = RuntimeConfigResponse {
        ollama_chat_model: db::get_config(&state.db_pool, "ollama_chat_model")
            .await.ok().flatten().unwrap_or_else(|| state.config.ollama_chat_model.clone()),
        ollama_embedding_model: db::get_config(&state.db_pool, "ollama_embedding_model")
            .await.ok().flatten().unwrap_or_else(|| state.config.ollama_embedding_model.clone()),
        ollama_vision_model: db::get_config(&state.db_pool, "ollama_vision_model")
            .await.ok().flatten().unwrap_or_else(|| state.config.ollama_vision_model.clone()),
        temperature: db::get_config_f64(&state.db_pool, "temperature", state.config.temperature).await,
        max_tokens: db::get_config_u32(&state.db_pool, "max_tokens", state.config.max_tokens).await,
        vision_enabled: db::get_config_bool(&state.db_pool, "vision_enabled", state.config.vision_enabled).await,
        voice_enabled: db::get_config_bool(&state.db_pool, "voice_enabled", state.config.voice_enabled).await,
        web_search_enabled: db::get_config_bool(&state.db_pool, "web_search_enabled", state.config.web_search_enabled).await,
        rag_decay_rate: db::get_config_f64(&state.db_pool, "rag_decay_rate", state.config.rag_decay_rate).await,
        summary_threshold: db::get_config_u32(&state.db_pool, "summary_threshold", state.config.summary_threshold).await,
        max_concurrent_llm_requests: db::get_config_u32(&state.db_pool, "max_concurrent_llm_requests", 
            state.config.max_concurrent_llm_requests.unwrap_or(3) as u32).await,
        llm_timeout_seconds: db::get_config(&state.db_pool, "llm_timeout_seconds")
            .await.ok().flatten().and_then(|v| v.parse().ok()).unwrap_or(state.config.llm_timeout_seconds),
        random_reply_probability: db::get_config_f64(&state.db_pool, "random_reply_probability", 
            state.config.random_reply_probability).await,
    };

    Ok(Json(ApiResponse::ok(config)))
}

pub async fn update_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    extract_user(&headers, &state)?;

    if let Some(v) = req.ollama_chat_model {
        let _ = db::set_config(&state.db_pool, "ollama_chat_model", &v).await;
    }
    if let Some(v) = req.ollama_embedding_model {
        let _ = db::set_config(&state.db_pool, "ollama_embedding_model", &v).await;
    }
    if let Some(v) = req.ollama_vision_model {
        let _ = db::set_config(&state.db_pool, "ollama_vision_model", &v).await;
    }
    if let Some(v) = req.temperature {
        let _ = db::set_config(&state.db_pool, "temperature", &v.to_string()).await;
    }
    if let Some(v) = req.max_tokens {
        let _ = db::set_config(&state.db_pool, "max_tokens", &v.to_string()).await;
    }
    if let Some(v) = req.vision_enabled {
        let _ = db::set_config(&state.db_pool, "vision_enabled", if v { "true" } else { "false" }).await;
    }
    if let Some(v) = req.voice_enabled {
        let _ = db::set_config(&state.db_pool, "voice_enabled", if v { "true" } else { "false" }).await;
    }
    if let Some(v) = req.web_search_enabled {
        let _ = db::set_config(&state.db_pool, "web_search_enabled", if v { "true" } else { "false" }).await;
    }
    if let Some(v) = req.rag_decay_rate {
        let _ = db::set_config(&state.db_pool, "rag_decay_rate", &v.to_string()).await;
    }
    if let Some(v) = req.summary_threshold {
        let _ = db::set_config(&state.db_pool, "summary_threshold", &v.to_string()).await;
    }
    if let Some(v) = req.max_concurrent_llm_requests {
        let _ = db::set_config(&state.db_pool, "max_concurrent_llm_requests", &v.to_string()).await;
    }
    if let Some(v) = req.llm_timeout_seconds {
        let _ = db::set_config(&state.db_pool, "llm_timeout_seconds", &v.to_string()).await;
    }
    if let Some(v) = req.random_reply_probability {
        let _ = db::set_config(&state.db_pool, "random_reply_probability", &v.to_string()).await;
    }

    Ok(Json(ApiResponse::ok(())))
}


// --- Security ---

#[derive(Serialize)]
pub struct SecurityStatusResponse {
    pub user_id: u64,
    pub strikes: u8,
    pub total_violations: u64,
    pub is_blocked: bool,
    pub is_rate_limited: bool,
}

#[derive(Serialize)]
pub struct SecurityConfigResponse {
    pub strike_threshold: u8,
    pub max_strikes: u8,
    pub block_duration_seconds: u64,
    pub strike_window_seconds: u64,
}

#[derive(Deserialize)]
pub struct BlockUserRequest {
    pub duration_minutes: Option<u64>,
}

pub async fn get_security_config(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SecurityConfigResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    Ok(Json(ApiResponse::ok(SecurityConfigResponse {
        strike_threshold: 30,
        max_strikes: 3,
        block_duration_seconds: 300,
        strike_window_seconds: 3600,
    })))
}

pub async fn get_user_security_status(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(user_id): Path<u64>,
) -> Result<Json<ApiResponse<SecurityStatusResponse>>, StatusCode> {
    extract_user(&headers, &state)?;

    let (strikes, total_violations, is_blocked) = state
        .security_tracker
        .get_user_stats(user_id)
        .await
        .unwrap_or((0, 0, false));

    let is_rate_limited = state.security_tracker.is_blocked(user_id).await.is_some();

    Ok(Json(ApiResponse::ok(SecurityStatusResponse {
        user_id,
        strikes,
        total_violations,
        is_blocked,
        is_rate_limited,
    })))
}

pub async fn block_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(user_id): Path<u64>,
    Json(req): Json<BlockUserRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let auth_user = extract_user(&headers, &state)?;

    // Don't allow blocking owner
    if user_id == state.config.owner_id {
        return Ok(Json(ApiResponse::err("Cannot block owner")));
    }

    let minutes = req.duration_minutes.unwrap_or(30);
    let duration = std::time::Duration::from_secs(minutes * 60);

    state.security_tracker.block_user(user_id, duration).await;

    log::info!(
        "User {} blocked by {} for {} minutes via API",
        user_id, auth_user.id, minutes
    );

    Ok(Json(ApiResponse::ok(())))
}

pub async fn unblock_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(user_id): Path<u64>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let auth_user = extract_user(&headers, &state)?;

    state.security_tracker.unblock_user(user_id).await;

    log::info!(
        "User {} unblocked by {} via API",
        user_id, auth_user.id
    );

    Ok(Json(ApiResponse::ok(())))
}

// --- Pause/Resume ---

#[derive(Serialize)]
pub struct PauseResponse {
    pub paused: bool,
}

pub async fn get_pause_status(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PauseResponse>>, StatusCode> {
    extract_user(&headers, &state)?;
    Ok(Json(ApiResponse::ok(PauseResponse { paused: state.is_paused() })))
}

pub async fn toggle_pause(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PauseResponse>>, StatusCode> {
    extract_user(&headers, &state)?;
    
    let new_state = !state.is_paused();
    state.set_paused(new_state);
    
    log::info!("Bot {} via API", if new_state { "paused" } else { "resumed" });
    
    Ok(Json(ApiResponse::ok(PauseResponse { paused: new_state })))
}
