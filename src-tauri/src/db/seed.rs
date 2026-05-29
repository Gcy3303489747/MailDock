use rusqlite::{params, Connection};

pub(crate) struct SeedMessage {
    pub id: &'static str,
    pub from: &'static str,
    pub subject: &'static str,
    pub received_at: &'static str,
    pub preview: &'static str,
    pub body: &'static str,
    pub has_attachments: bool,
    pub is_unread: bool,
}

pub(crate) const SEED_ACCOUNT_ADDRESS: &str = "learning@qq.com";

pub(crate) const SEED_MESSAGES: &[SeedMessage] = &[
    SeedMessage {
        id: "qq-001",
        from: "QQ Mail Team <service@qq.com>",
        subject: "Welcome to MailDock",
        received_at: "2026-05-26T08:30:00.000Z",
        preview: "This mock message now lives in SQLite instead of only in React state.",
        body: "MailDock now initializes a local SQLite database, seeds learning-friendly messages, and reads the inbox through a Tauri command.",
        has_attachments: false,
        is_unread: true,
    },
    SeedMessage {
        id: "qq-002",
        from: "Learning Log <notes@maildock.local>",
        subject: "Week 02: SQLite local cache",
        received_at: "2026-05-25T13:05:00.000Z",
        preview: "The app can keep mail metadata locally before real IMAP sync exists.",
        body: "SQLite gives MailDock a durable local cache. The same table shape will later store messages fetched from QQ Mail.",
        has_attachments: false,
        is_unread: true,
    },
    SeedMessage {
        id: "qq-003",
        from: "Tauri Backend <backend@maildock.local>",
        subject: "React is now reading from Rust and SQLite",
        received_at: "2026-05-24T21:18:00.000Z",
        preview: "The frontend calls list_messages, and Rust owns the database boundary.",
        body: "Keeping SQLite access in Rust helps protect future account data and keeps the React layer focused on interface state.",
        has_attachments: false,
        is_unread: false,
    },
    SeedMessage {
        id: "qq-004",
        from: "SQLite Notes <db@maildock.local>",
        subject: "Schema: accounts, messages, sync_state",
        received_at: "2026-05-23T16:42:00.000Z",
        preview: "The first schema is small but ready for real provider sync.",
        body: "The accounts table identifies a mailbox, messages stores inbox rows, and sync_state will later remember the last QQ Mail sync cursor.",
        has_attachments: true,
        is_unread: false,
    },
    SeedMessage {
        id: "qq-005",
        from: "Security Reminder <security@maildock.local>",
        subject: "SQLite is not for plaintext authorization codes",
        received_at: "2026-05-22T10:10:00.000Z",
        preview: "The database can store mail content, but secrets need a credential vault.",
        body: "A later milestone should use the Windows credential store or a Tauri-supported secret storage option for QQ Mail authorization codes.",
        has_attachments: false,
        is_unread: false,
    },
];

pub(crate) fn seed_database(connection: &Connection) -> Result<(), String> {
    seed_account(connection)?;
    let account_id = seeded_account_id(connection)?;
    seed_messages(connection, account_id)?;
    seed_sync_state(connection, account_id)?;
    Ok(())
}

fn seed_account(connection: &Connection) -> Result<(), String> {
    connection
        .execute(
            "INSERT OR IGNORE INTO accounts(
                provider,
                address,
                display_name,
                auth_type,
                imap_host,
                imap_port,
                is_enabled
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                "qq",
                SEED_ACCOUNT_ADDRESS,
                "QQ Mail",
                "authorization_code",
                "imap.qq.com",
                993,
                1
            ],
        )
        .map_err(|error| format!("Failed to seed account: {error}"))?;

    connection
        .execute(
            "UPDATE accounts
             SET provider = ?1,
                 display_name = ?2,
                 auth_type = ?3,
                 imap_host = ?4,
                 imap_port = ?5,
                 is_enabled = ?6
             WHERE address = ?7",
            params![
                "qq",
                "QQ Mail",
                "authorization_code",
                "imap.qq.com",
                993,
                1,
                SEED_ACCOUNT_ADDRESS
            ],
        )
        .map_err(|error| format!("Failed to refresh seeded account: {error}"))?;

    Ok(())
}

fn seeded_account_id(connection: &Connection) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT id FROM accounts WHERE address = ?1",
            params![SEED_ACCOUNT_ADDRESS],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to read seeded account: {error}"))
}

fn seed_messages(connection: &Connection, account_id: i64) -> Result<(), String> {
    for message in SEED_MESSAGES {
        connection
            .execute(
                "INSERT OR IGNORE INTO messages(
                    id,
                    account_id,
                    folder,
                    from_address,
                    subject,
                    received_at,
                    preview,
                    body,
                    has_attachments,
                    is_unread
                 )
                 VALUES (?1, ?2, 'INBOX', ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    message.id,
                    account_id,
                    message.from,
                    message.subject,
                    message.received_at,
                    message.preview,
                    message.body,
                    message.has_attachments as i64,
                    message.is_unread as i64,
                ],
            )
            .map_err(|error| format!("Failed to seed message {}: {error}", message.id))?;
    }

    Ok(())
}

fn seed_sync_state(connection: &Connection, account_id: i64) -> Result<(), String> {
    connection
        .execute(
            "INSERT OR IGNORE INTO sync_state(account_id, folder, last_synced_at, last_uid)
             VALUES (?1, 'INBOX', NULL, NULL)",
            params![account_id],
        )
        .map_err(|error| format!("Failed to seed sync state: {error}"))?;

    Ok(())
}
