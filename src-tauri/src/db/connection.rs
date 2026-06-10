use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const INITIAL_SCHEMA: &str = include_str!("../../migrations/001_initial.sql");

pub(crate) fn open_database(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    let connection = Connection::open(path)
        .map_err(|error| format!("Failed to open MailDock database: {error}"))?;

    initialize_connection(&connection)?;
    Ok(connection)
}

pub(crate) fn initialize_connection(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(INITIAL_SCHEMA)
        .map_err(|error| format!("Failed to initialize database schema: {error}"))?;

    apply_account_compat_migrations(connection)?;
    apply_message_compat_migrations(connection)?;
    apply_sync_state_compat_migrations(connection)?;
    backfill_message_sort_columns(connection)?;
    refresh_message_indexes(connection)?;
    Ok(())
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve app data directory: {error}"))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|error| format!("Failed to create app data directory: {error}"))?;

    Ok(app_data_dir.join("maildock.sqlite3"))
}

fn apply_account_compat_migrations(connection: &Connection) -> Result<(), String> {
    add_column_if_missing(
        connection,
        "accounts",
        "auth_type",
        "ALTER TABLE accounts ADD COLUMN auth_type TEXT NOT NULL DEFAULT 'authorization_code'",
    )?;
    add_column_if_missing(
        connection,
        "accounts",
        "imap_host",
        "ALTER TABLE accounts ADD COLUMN imap_host TEXT NOT NULL DEFAULT 'imap.qq.com'",
    )?;
    add_column_if_missing(
        connection,
        "accounts",
        "imap_port",
        "ALTER TABLE accounts ADD COLUMN imap_port INTEGER NOT NULL DEFAULT 993",
    )?;
    add_column_if_missing(
        connection,
        "accounts",
        "is_enabled",
        "ALTER TABLE accounts ADD COLUMN is_enabled INTEGER NOT NULL DEFAULT 1",
    )?;

    Ok(())
}

fn apply_message_compat_migrations(connection: &Connection) -> Result<(), String> {
    add_column_if_missing(
        connection,
        "messages",
        "provider_message_id",
        "ALTER TABLE messages ADD COLUMN provider_message_id TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        connection,
        "messages",
        "uid",
        "ALTER TABLE messages ADD COLUMN uid TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        connection,
        "messages",
        "received_at_epoch",
        "ALTER TABLE messages ADD COLUMN received_at_epoch INTEGER NOT NULL DEFAULT 0",
    )?;

    Ok(())
}

fn apply_sync_state_compat_migrations(connection: &Connection) -> Result<(), String> {
    add_column_if_missing(
        connection,
        "sync_state",
        "last_attempt_at",
        "ALTER TABLE sync_state ADD COLUMN last_attempt_at TEXT",
    )?;
    add_column_if_missing(
        connection,
        "sync_state",
        "last_success_at",
        "ALTER TABLE sync_state ADD COLUMN last_success_at TEXT",
    )?;
    add_column_if_missing(
        connection,
        "sync_state",
        "last_error",
        "ALTER TABLE sync_state ADD COLUMN last_error TEXT",
    )?;
    add_column_if_missing(
        connection,
        "sync_state",
        "is_syncing",
        "ALTER TABLE sync_state ADD COLUMN is_syncing INTEGER NOT NULL DEFAULT 0",
    )?;

    Ok(())
}

fn backfill_message_sort_columns(connection: &Connection) -> Result<(), String> {
    let mut statement = connection
        .prepare("SELECT id, received_at FROM messages WHERE received_at_epoch = 0 OR uid = ''")
        .map_err(|error| format!("Failed to prepare message backfill query: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("Failed to read message backfill rows: {error}"))?;

    for row in rows {
        let (id, received_at) =
            row.map_err(|error| format!("Failed to map message backfill row: {error}"))?;
        let uid = id.rsplit(':').next().unwrap_or(&id).to_owned();
        let received_at_epoch = crate::time_utils::received_at_sort_key(&received_at);

        connection
            .execute(
                "UPDATE messages
                 SET provider_message_id = CASE WHEN provider_message_id = '' THEN ?2 ELSE provider_message_id END,
                     uid = CASE WHEN uid = '' THEN ?3 ELSE uid END,
                     received_at_epoch = CASE WHEN received_at_epoch = 0 THEN ?4 ELSE received_at_epoch END
                 WHERE id = ?1",
                rusqlite::params![id, id, uid, received_at_epoch],
            )
            .map_err(|error| format!("Failed to backfill message sort columns: {error}"))?;
    }

    Ok(())
}

fn refresh_message_indexes(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DROP INDEX IF EXISTS idx_messages_account_folder_received;
             CREATE INDEX IF NOT EXISTS idx_messages_account_folder_received
             ON messages(account_id, folder, received_at_epoch DESC);",
        )
        .map_err(|error| format!("Failed to refresh message indexes: {error}"))
}

fn add_column_if_missing(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    alter_statement: &str,
) -> Result<(), String> {
    if has_column(connection, table_name, column_name)? {
        return Ok(());
    }

    connection
        .execute(alter_statement, [])
        .map_err(|error| format!("Failed to add {table_name}.{column_name}: {error}"))?;

    Ok(())
}

fn has_column(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|error| format!("Failed to inspect {table_name} columns: {error}"))?;

    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to read {table_name} columns: {error}"))?;

    for column in columns {
        if column.map_err(|error| format!("Failed to map {table_name} column: {error}"))?
            == column_name
        {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    #[test]
    fn migrates_legacy_schema_without_clearing_cached_messages() {
        let connection = Connection::open_in_memory().expect("open test database");
        connection
            .execute_batch(
                "
                CREATE TABLE accounts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    provider TEXT NOT NULL,
                    address TEXT NOT NULL UNIQUE,
                    display_name TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE messages (
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
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE sync_state (
                    account_id INTEGER NOT NULL,
                    folder TEXT NOT NULL,
                    last_synced_at TEXT,
                    last_uid TEXT,
                    PRIMARY KEY (account_id, folder)
                );
                ",
            )
            .expect("create legacy schema");
        connection
            .execute(
                "INSERT INTO accounts(provider, address, display_name) VALUES ('qq', 'student@qq.com', 'QQ Mail')",
                [],
            )
            .expect("insert account");
        connection
            .execute(
                "INSERT INTO messages(id, account_id, folder, from_address, subject, received_at, preview, body)
                 VALUES ('qq:student@qq.com:7', 1, 'INBOX', 'Sender', 'Subject', ?1, 'Preview', 'Body')",
                params!["2026-05-29T10:00:00+08:00"],
            )
            .expect("insert message");

        initialize_connection(&connection).expect("migrate schema");

        assert!(has_column(&connection, "messages", "received_at_epoch").expect("column exists"));
        assert!(has_column(&connection, "sync_state", "last_success_at").expect("column exists"));

        let migrated: (String, i64) = connection
            .query_row(
                "SELECT uid, received_at_epoch FROM messages WHERE id = 'qq:student@qq.com:7'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read migrated message");
        assert_eq!(migrated.0, "7");
        assert!(migrated.1 > 0);
    }
}
