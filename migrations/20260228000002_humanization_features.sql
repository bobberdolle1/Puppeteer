-- Add humanization settings to accounts table
ALTER TABLE accounts ADD COLUMN min_response_delay_sec INTEGER NOT NULL DEFAULT 2;
ALTER TABLE accounts ADD COLUMN max_response_delay_sec INTEGER NOT NULL DEFAULT 15;
ALTER TABLE accounts ADD COLUMN typing_speed_cpm INTEGER NOT NULL DEFAULT 200; -- characters per minute
ALTER TABLE accounts ADD COLUMN use_reply_probability INTEGER NOT NULL DEFAULT 70; -- 0-100, chance to use reply vs regular message
ALTER TABLE accounts ADD COLUMN ignore_old_messages_sec INTEGER NOT NULL DEFAULT 300; -- ignore messages older than 5 min
ALTER TABLE accounts ADD COLUMN always_respond_in_pm INTEGER NOT NULL DEFAULT 1; -- always respond in private chats

-- Create bot groups table for coordinated actions
CREATE TABLE IF NOT EXISTS bot_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create bot group members table
CREATE TABLE IF NOT EXISTS bot_group_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES bot_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    UNIQUE(group_id, account_id)
);

-- Create spam campaigns table for coordinated attacks
CREATE TABLE IF NOT EXISTS spam_campaigns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    group_id INTEGER,
    target_type TEXT NOT NULL, -- 'chat', 'user', 'group'
    target_id INTEGER NOT NULL,
    message_text TEXT,
    media_path TEXT,
    media_type TEXT, -- 'photo', 'video', 'gif', 'document'
    repeat_count INTEGER NOT NULL DEFAULT 1,
    delay_between_ms INTEGER NOT NULL DEFAULT 1000,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'running', 'completed', 'stopped'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES bot_groups(id) ON DELETE SET NULL
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_bot_group_members_group ON bot_group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_bot_group_members_account ON bot_group_members(account_id);
CREATE INDEX IF NOT EXISTS idx_spam_campaigns_status ON spam_campaigns(status);
