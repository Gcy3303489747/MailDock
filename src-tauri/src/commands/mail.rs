use crate::db;
use crate::models::{MailAccount, MailMessage, SyncState};
use tauri::AppHandle;

#[tauri::command]
pub fn list_accounts(app: AppHandle) -> Result<Vec<MailAccount>, String> {
    db::list_accounts(&app)
}

#[tauri::command]
pub fn list_messages(
    app: AppHandle,
    account_id: i64,
    folder: String,
) -> Result<Vec<MailMessage>, String> {
    db::list_messages(&app, account_id, &folder)
}

#[tauri::command]
pub fn get_sync_state(
    app: AppHandle,
    account_id: i64,
    folder: String,
) -> Result<SyncState, String> {
    db::get_sync_state(&app, account_id, &folder)
}
