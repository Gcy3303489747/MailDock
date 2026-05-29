use crate::db;
use crate::models::{MailAccount, MailMessage};
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
