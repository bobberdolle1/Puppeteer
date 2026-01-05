-- Runtime configuration that can be changed without restart
CREATE TABLE IF NOT EXISTS runtime_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default values from env (will be overwritten by actual env on first run)
INSERT OR IGNORE INTO runtime_config (key, value) VALUES 
    ('ollama_chat_model', 'gemma2:2b'),
    ('ollama_embedding_model', 'nomic-embed-text'),
    ('ollama_vision_model', 'llava:latest'),
    ('temperature', '0.7'),
    ('max_tokens', '2048'),
    ('vision_enabled', 'false'),
    ('voice_enabled', 'false'),
    ('web_search_enabled', 'true'),
    ('rag_decay_rate', '0.1'),
    ('summary_threshold', '50'),
    ('max_concurrent_llm_requests', '3'),
    ('llm_timeout_seconds', '120'),
    ('random_reply_probability', '0.0');
