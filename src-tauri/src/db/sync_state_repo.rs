use crate::models::SyncState;
use rusqlite::{params, Connection, OptionalExtension};

pub(crate) fn get_sync_state(
    connection: &Connection,
    account_id: i64,
    folder: &str,
) -> Result<SyncState, String> {
    let state = connection
        .query_row(
            "SELECT account_id, folder, last_attempt_at, last_success_at, last_error, is_syncing
             FROM sync_state
             WHERE account_id = ?1 AND folder = ?2",
            params![account_id, folder],
            |row| {
                Ok(SyncState {
                    account_id: row.get(0)?,
                    folder: row.get(1)?,
                    last_attempt_at: row.get(2)?,
                    last_success_at: row.get(3)?,
                    last_error: row.get(4)?,
                    is_syncing: row.get::<_, i64>(5)? != 0,
                })
            },
        )
        .optional()
        .map_err(|error| format!("Failed to read sync state: {error}"))?;

    Ok(state.unwrap_or_else(|| SyncState {
        account_id,
        folder: folder.to_owned(),
        last_attempt_at: None,
        last_success_at: None,
        last_error: None,
        is_syncing: false,
    }))
}

pub(crate) fn mark_sync_started(
    connection: &Connection,
    account_id: i64,
    folder: &str,
) -> Result<String, String> {
    let timestamp = current_timestamp(connection)?;
    connection
        .execute(
            "INSERT INTO sync_state(account_id, folder, last_attempt_at, is_syncing)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(account_id, folder) DO UPDATE SET
                last_attempt_at = excluded.last_attempt_at,
                last_error = NULL,
                is_syncing = 1",
            params![account_id, folder, timestamp],
        )
        .map_err(|error| format!("Failed to mark sync started: {error}"))?;

    Ok(timestamp)
}

pub(crate) fn mark_sync_finished(
    connection: &Connection,
    account_id: i64,
    folder: &str,
    last_uid: Option<&str>,
) -> Result<String, String> {
    let timestamp = current_timestamp(connection)?;
    connection
        .execute(
            "INSERT INTO sync_state(account_id, folder, last_synced_at, last_success_at, last_uid, last_error, is_syncing)
             VALUES (?1, ?2, ?3, ?3, ?4, NULL, 0)
             ON CONFLICT(account_id, folder) DO UPDATE SET
                last_synced_at = excluded.last_synced_at,
                last_success_at = excluded.last_success_at,
                last_uid = excluded.last_uid,
                last_error = NULL,
                is_syncing = 0",
            params![account_id, folder, timestamp, last_uid],
        )
        .map_err(|error| format!("Failed to mark sync finished: {error}"))?;

    Ok(timestamp)
}

pub(crate) fn mark_sync_failed(
    connection: &Connection,
    account_id: i64,
    folder: &str,
    message: &str,
) -> Result<String, String> {
    let timestamp = current_timestamp(connection)?;
    connection
        .execute(
            "INSERT INTO sync_state(account_id, folder, last_attempt_at, last_error, is_syncing)
             VALUES (?1, ?2, ?3, ?4, 0)
             ON CONFLICT(account_id, folder) DO UPDATE SET
                last_attempt_at = excluded.last_attempt_at,
                last_error = excluded.last_error,
                is_syncing = 0",
            params![account_id, folder, timestamp, message],
        )
        .map_err(|error| format!("Failed to mark sync failed: {error}"))?;

    Ok(timestamp)
}

fn current_timestamp(connection: &Connection) -> Result<String, String> {
    connection
        .query_row("SELECT datetime('now')", [], |row| row.get(0))
        .map_err(|error| format!("Failed to read current database timestamp: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::account_repo::upsert_qq_account;
    use crate::db::connection::initialize_connection;

    #[test]
    fn tracks_sync_state_lifecycle() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        let account_id =
            upsert_qq_account(&connection, "student@qq.com").expect("upsert QQ account");

        mark_sync_started(&connection, account_id, "INBOX").expect("start sync");
        let started = get_sync_state(&connection, account_id, "INBOX").expect("read state");
        assert!(started.is_syncing);
        assert!(started.last_error.is_none());

        mark_sync_finished(&connection, account_id, "INBOX", Some("10")).expect("finish sync");
        let finished = get_sync_state(&connection, account_id, "INBOX").expect("read state");
        assert!(!finished.is_syncing);
        assert!(finished.last_success_at.is_some());

        mark_sync_failed(&connection, account_id, "INBOX", "offline").expect("fail sync");
        let failed = get_sync_state(&connection, account_id, "INBOX").expect("read state");
        assert!(!failed.is_syncing);
        assert_eq!(failed.last_error, Some("offline".into()));
    }
}
