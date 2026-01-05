use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    pub teloxide_token: String,
    pub database_url: String,
    pub owner_id: u64,
    #[serde(default = "default_ollama_chat_model")]
    pub ollama_chat_model: String,
    #[serde(default = "default_ollama_embedding_model")]
    pub ollama_embedding_model: String,
    #[serde(default = "default_ollama_vision_model")]
    pub ollama_vision_model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_bot_name")]
    pub bot_name: String,
    /// Maximum concurrent LLM requests (queue limit)
    #[serde(default = "default_max_concurrent_llm")]
    pub max_concurrent_llm_requests: Option<usize>,
    /// Timeout for LLM requests in seconds
    #[serde(default = "default_llm_timeout")]
    pub llm_timeout_seconds: u64,
    /// Queue wait timeout in seconds
    #[serde(default = "default_queue_timeout")]
    pub queue_timeout_seconds: u64,
    /// Enable vision (image analysis) support
    #[serde(default = "default_vision_enabled")]
    pub vision_enabled: bool,
    /// Random reply probability (0.0-1.0) for group chats
    #[serde(default = "default_random_reply_probability")]
    pub random_reply_probability: f64,
    /// Enable web search for current information
    #[serde(default = "default_web_search_enabled")]
    pub web_search_enabled: bool,
    /// Enable voice message transcription
    #[serde(default = "default_voice_enabled")]
    pub voice_enabled: bool,
    /// Whisper API URL for voice transcription
    #[serde(default = "default_whisper_url")]
    pub whisper_url: String,
    /// Time decay rate for RAG (0.0 = no decay, 1.0 = fast decay)
    #[serde(default = "default_rag_decay_rate")]
    pub rag_decay_rate: f64,
    /// Number of messages before auto-summarization
    #[serde(default = "default_summary_threshold")]
    pub summary_threshold: u32,
    /// WebApp server port
    #[serde(default = "default_webapp_port")]
    pub webapp_port: u16,
}

fn default_ollama_url() -> String {
    "http://host.docker.internal:11434".to_string()
}

fn default_ollama_chat_model() -> String {
    "gemini-3-flash-preview:cloud".to_string()
}

fn default_ollama_embedding_model() -> String {
    "nomic-embed-text".to_string()
}

fn default_ollama_vision_model() -> String {
    "llava:latest".to_string()
}

fn default_temperature() -> f64 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

fn default_bot_name() -> String {
    "PersonaForge".to_string()
}

fn default_max_concurrent_llm() -> Option<usize> {
    Some(3)
}

fn default_llm_timeout() -> u64 {
    120
}

fn default_queue_timeout() -> u64 {
    30
}

fn default_vision_enabled() -> bool {
    false
}

fn default_random_reply_probability() -> f64 {
    0.0
}

fn default_web_search_enabled() -> bool {
    true
}

fn default_voice_enabled() -> bool {
    false
}

fn default_whisper_url() -> String {
    "http://localhost:8080".to_string()
}

fn default_rag_decay_rate() -> f64 {
    0.1 // Slow decay by default
}

fn default_summary_threshold() -> u32 {
    50 // Summarize every 50 messages
}

fn default_webapp_port() -> u16 {
    8080
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Config>()
    }
}
