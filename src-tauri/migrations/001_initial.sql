CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL,
    address TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    auth_type TEXT NOT NULL DEFAULT 'authorization_code',
    imap_host TEXT NOT NULL DEFAULT '',
    imap_port INTEGER NOT NULL DEFAULT 993,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    account_id INTEGER NOT NULL,
    folder TEXT NOT NULL,
    from_address TEXT NOT NULL,
    subject TEXT NOT NULL,
    received_at TEXT NOT NULL,
    preview TEXT NOT NULL,
    body TEXT NOT NULL,
    has_attachments INTEGER NOT NULL DEFAULT 0,
    is_unread INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE INDEX IF NOT EXISTS idx_messages_account_folder_received
ON messages(account_id, folder, received_at DESC);

CREATE TABLE IF NOT EXISTS sync_state (
    account_id INTEGER NOT NULL,
    folder TEXT NOT NULL,
    last_synced_at TEXT,
    last_uid TEXT,
    PRIMARY KEY (account_id, folder),
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);
