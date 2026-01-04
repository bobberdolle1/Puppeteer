-- Add migration script here

-- Table to store different bot personas (system prompts)
CREATE TABLE IF NOT EXISTS personas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    prompt TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Table to store settings for each chat
CREATE TABLE IF NOT EXISTS chat_settings (
    chat_id INTEGER PRIMARY KEY, -- Telegram Chat ID
    auto_reply_enabled BOOLEAN NOT NULL DEFAULT 1,
    reply_mode TEXT NOT NULL DEFAULT 'mention_only', -- 'mention_only' or 'all_messages'
    cooldown_seconds INTEGER NOT NULL DEFAULT 5,
    context_depth INTEGER NOT NULL DEFAULT 10,
    rag_enabled BOOLEAN NOT NULL DEFAULT 1,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Table to store all messages for context and RAG
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    chat_id INTEGER NOT NULL,
    user_id INTEGER,
    username TEXT,
    text TEXT,
    sent_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Table for RAG memory chunks and their embeddings
-- For now, a chunk is a whole message. This can be expanded later.
CREATE TABLE IF NOT EXISTS memory_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    embedding BLOB, -- Storing vector as a blob
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (message_id) REFERENCES messages (id)
);

-- Indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_sent_at ON messages(sent_at);
CREATE INDEX IF NOT EXISTS idx_memory_chunks_message_id ON memory_chunks(message_id);

-- Insert a default persona
INSERT INTO personas (name, prompt, is_active) VALUES ('Default', 'You are a helpful AI assistant. Your name is PersonaForge. Keep your answers concise.', 1);

-- Insert default settings for a placeholder chat_id 0, which can be a template.
INSERT INTO chat_settings (chat_id) VALUES (0);