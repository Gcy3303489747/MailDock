use crate::db;
use crate::imap::{self, QqInboxSyncInput};
use crate::models::ProviderKind;
use crate::security::{CredentialKey, CredentialService, SystemCredentialService};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedQqInboxSyncInput {
    pub account_id: i64,
    pub limit: Option<u32>,
}

#[tauri::command]
pub fn sync_qq_inbox(app: AppHandle, input: QqInboxSyncInput) -> Result<QqInboxSyncReport, String> {
    let address = input.email.trim().to_lowercase();
    let authorization_code = input.authorization_code.trim().to_owned();
    let payload = imap::sync_qq_inbox(input)?;
    let account_id = db::upsert_qq_account(&app, &address)?;
    let credential_key = CredentialKey {
        account_id,
        provider: ProviderKind::Qq,
    };

    SystemCredentialService.set_secret(&credential_key, &authorization_code)?;

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

#[tauri::command]
pub fn sync_saved_qq_inbox(
    app: AppHandle,
    input: SavedQqInboxSyncInput,
) -> Result<QqInboxSyncReport, String> {
    let account = db::get_account(&app, input.account_id)?;
    if account.provider != ProviderKind::Qq {
        return Err("Saved sync is currently only available for QQ Mail accounts.".into());
    }

    let credential_key = CredentialKey {
        account_id: account.id,
        provider: ProviderKind::Qq,
    };
    let authorization_code = SystemCredentialService
        .get_secret(&credential_key)?
        .ok_or_else(|| "No saved QQ authorization code found. Import this mailbox again to enable automatic sync.".to_string())?;

    let payload = imap::sync_qq_inbox(QqInboxSyncInput {
        email: account.address.clone(),
        authorization_code,
        limit: input.limit,
    })?;
    let stored = db::upsert_messages(&app, account.id, "INBOX", &payload.messages)?;

    Ok(QqInboxSyncReport {
        account_id: account.id,
        address: account.address,
        folder: payload.report.folder,
        fetched: payload.messages.len(),
        stored,
        total_inbox_messages: payload.report.exists,
    })
}
