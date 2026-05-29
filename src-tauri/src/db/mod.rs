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

fn initialized_database(app: &AppHandle) -> Result<rusqlite::Connection, String> {
    let connection = connection::open_database(app)?;
    seed::seed_database(&connection)?;
    Ok(connection)
}
