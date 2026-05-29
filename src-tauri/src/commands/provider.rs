use crate::db;
use crate::imap::{self, ImapConnectionReport, QqImapConnectionInput, QqInboxSyncInput};
use serde::Serialize;
use tauri::AppHandle;

#[tauri::command]
pub fn test_qq_imap_connection(
    input: QqImapConnectionInput,
) -> Result<ImapConnectionReport, String> {
    imap::test_qq_connection(input)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QqInboxSyncReport {
    pub account_id: i64,
    pub address: String,
    pub folder: String,
    pub fetched: usize,
    pub stored: usize,
    pub total_inbox_messages: u32,
}

#[tauri::command]
pub fn sync_qq_inbox(app: AppHandle, input: QqInboxSyncInput) -> Result<QqInboxSyncReport, String> {
    let address = input.email.trim().to_lowercase();
    let payload = imap::sync_qq_inbox(input)?;
    let account_id = db::upsert_qq_account(&app, &address)?;
    let stored = db::upsert_messages(&app, account_id, "INBOX", &payload.messages)?;

    Ok(QqInboxSyncReport {
        account_id,
        address,
        folder: payload.report.folder,
        fetched: payload.messages.len(),
        stored,
        total_inbox_messages: payload.report.exists,
    })
}
