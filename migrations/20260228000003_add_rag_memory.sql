-- Add RAG Memory table for semantic long-term memory
CREATE TABLE IF NOT EXISTS long_term_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    chat_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Index for fast lookups by account and chat
CREATE INDEX IF NOT EXISTS idx_memory_account_chat ON long_term_memory(account_id, chat_id);

-- Index for timestamp-based queries
CREATE INDEX IF NOT EXISTS idx_memory_created_at ON long_term_memory(created_at);
