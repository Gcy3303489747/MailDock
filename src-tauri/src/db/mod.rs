mod account_repo;
pub(crate) mod connection;
mod message_repo;
pub(crate) mod seed;

use crate::models::{MailAccount, MailMessage};
use tauri::AppHandle;

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

fn initialized_database(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    let connection = connection::open_database(app)?;
    seed::cleanup_legacy_seed_data(&connection)?;
    Ok(connection)
}
