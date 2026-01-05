-- Add summaries table for memory summarization
CREATE TABLE IF NOT EXISTS chat_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    summary_text TEXT NOT NULL,
    messages_from INTEGER NOT NULL, -- First message ID included
    messages_to INTEGER NOT NULL,   -- Last message ID included
    message_count INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add importance score to memory_chunks for time-decay
ALTER TABLE memory_chunks ADD COLUMN importance_score REAL DEFAULT 1.0;

-- Index for faster summary lookups
CREATE INDEX IF NOT EXISTS idx_chat_summaries_chat_id ON chat_summaries(chat_id);
CREATE INDEX IF NOT EXISTS idx_chat_summaries_created_at ON chat_summaries(created_at);
