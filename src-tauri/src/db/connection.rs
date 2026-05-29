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
