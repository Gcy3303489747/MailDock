mod account_repo;
pub(crate) mod connection;
mod message_repo;
pub(crate) mod seed;
mod sync_state_repo;

use crate::models::{MailAccount, MailMessage, SyncState};
use tauri::AppHandle;

pub fn get_account(app: &AppHandle, account_id: i64) -> Result<MailAccount, String> {
    let connection = initialized_database(app)?;
    account_repo::get_account(&connection, account_id)
}

pub fn list_accounts(app: &AppHandle) -> Result<Vec<MailAccount>, String> {
    let connection = initialized_database(app)?;
    account_repo::list_accounts(&connection)
}

pub fn list_messages(
    app: &AppHandle,
    account_id: i64,
    folder: &str,
) -> Result<Vec<MailMessage>, String> {
    let connection = initialized_database(app)?;
    message_repo::list_messages(&connection, account_id, folder)
}

pub fn upsert_qq_account(app: &AppHandle, address: &str) -> Result<i64, String> {
    let connection = initialized_database(app)?;
    account_repo::upsert_qq_account(&connection, address)
}

pub fn upsert_messages(
    app: &AppHandle,
    account_id: i64,
    folder: &str,
    messages: &[MailMessage],
) -> Result<usize, String> {
    let connection = initialized_database(app)?;
    message_repo::upsert_messages(&connection, account_id, folder, messages)
}

pub fn last_message_uid(messages: &[MailMessage]) -> Option<String> {
    message_repo::last_message_uid(messages)
}

pub fn get_sync_state(app: &AppHandle, account_id: i64, folder: &str) -> Result<SyncState, String> {
    let connection = initialized_database(app)?;
    sync_state_repo::get_sync_state(&connection, account_id, folder)
}

pub fn mark_sync_started(app: &AppHandle, account_id: i64, folder: &str) -> Result<String, String> {
    let connection = initialized_database(app)?;
    sync_state_repo::mark_sync_started(&connection, account_id, folder)
}

pub fn mark_sync_finished(
    app: &AppHandle,
    account_id: i64,
    folder: &str,
    last_uid: Option<&str>,
) -> Result<String, String> {
    let connection = initialized_database(app)?;
    sync_state_repo::mark_sync_finished(&connection, account_id, folder, last_uid)
}

pub fn mark_sync_failed(
    app: &AppHandle,
    account_id: i64,
    folder: &str,
    message: &str,
) -> Result<String, String> {
    let connection = initialized_database(app)?;
    sync_state_repo::mark_sync_failed(&connection, account_id, folder, message)
}

fn initialized_database(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    let connection = connection::open_database(app)?;
    seed::cleanup_legacy_seed_data(&connection)?;
    Ok(connection)
}
