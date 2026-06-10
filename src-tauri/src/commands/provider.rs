use crate::imap::QqInboxSyncInput;
use crate::mail_service::{self, QqInboxSyncReport};
use serde::Deserialize;
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedQqInboxSyncInput {
    pub account_id: i64,
}

#[tauri::command]
pub fn sync_qq_inbox(app: AppHandle, input: QqInboxSyncInput) -> Result<QqInboxSyncReport, String> {
    mail_service::import_qq_account(&app, input)
}

#[tauri::command]
pub fn sync_saved_qq_inbox(
    app: AppHandle,
    input: SavedQqInboxSyncInput,
) -> Result<QqInboxSyncReport, String> {
    mail_service::sync_account_now(&app, input.account_id)
}

#[tauri::command]
pub fn sync_account_now(
    app: AppHandle,
    input: SavedQqInboxSyncInput,
) -> Result<QqInboxSyncReport, String> {
    mail_service::sync_account_now(&app, input.account_id)
}
