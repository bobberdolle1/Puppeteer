use axum::{
    Router,
    routing::{get, post, put},
    http::{header, Method, StatusCode, Uri},
    response::IntoResponse,
};
use rust_embed::Embed;
use tower_http::cors::{Any, CorsLayer};
use crate::state::AppState;
use super::api;

#[derive(Embed)]
#[folder = "src/webapp/static/"]
struct Assets;

/// Serve embedded static files
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => {
            // Fallback to index.html for SPA routing
            match Assets::get("index.html") {
                Some(content) => {
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/html")],
                        content.data.into_owned(),
                    )
                        .into_response()
                }
                None => (StatusCode::NOT_FOUND, "Not found").into_response(),
            }
        }
    }
}

/// Create the webapp router
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::HeaderName::from_static("x-telegram-init-data")]);

    let api_routes = Router::new()
        // Personas
        .route("/personas", get(api::list_personas).post(api::create_persona))
        .route("/personas/{id}", put(api::update_persona))
        .route("/personas/{id}/delete", post(api::delete_persona))
        .route("/personas/{id}/activate", post(api::activate_persona))
        // Chat settings
        .route("/chats", get(api::list_chats))
        .route("/chats/{chat_id}", get(api::get_chat_settings).put(api::update_chat_settings))
        // Triggers
        .route("/chats/{chat_id}/triggers", get(api::get_triggers).put(api::update_triggers))
        // Security
        .route("/security", get(api::get_security_config))
        .route("/security/users/{user_id}", get(api::get_user_security_status))
        .route("/security/users/{user_id}/block", post(api::block_user))
        .route("/security/users/{user_id}/unblock", post(api::unblock_user))
        // System
        .route("/status", get(api::get_status))
        .route("/models", get(api::list_models))
        .route("/stats", get(api::get_chat_stats))
        .route("/broadcast", post(api::broadcast))
        .route("/config", get(api::get_config).put(api::update_config))
        // Pause/Resume
        .route("/pause", get(api::get_pause_status).post(api::toggle_pause));

    Router::new()
        .nest("/api", api_routes)
        .fallback(static_handler)
        .layer(cors)
        .with_state(state)
}

/// Start the webapp server
pub async fn start_webapp_server(state: AppState, port: u16) {
    let app = create_router(state.clone());
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    
    log::info!("🌐 WebApp server starting on http://0.0.0.0:{}", port);
    log::info!("   └─ API: http://localhost:{}/api/status", port);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
