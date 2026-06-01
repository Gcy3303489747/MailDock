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
    pub credential_saved: bool,
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
    save_qq_authorization_code(account_id, &address, &authorization_code)?;

    let stored = db::upsert_messages(&app, account_id, "INBOX", &payload.messages)?;

    Ok(QqInboxSyncReport {
        account_id,
        address,
        folder: payload.report.folder,
        fetched: payload.messages.len(),
        stored,
        total_inbox_messages: payload.report.exists,
        credential_saved: true,
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

    let credential_key = CredentialKey::for_mailbox(ProviderKind::Qq, &account.address);
    let authorization_code = saved_qq_authorization_code(&credential_key, account.id)?;

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
        credential_saved: true,
    })
}

fn save_qq_authorization_code(
    account_id: i64,
    address: &str,
    authorization_code: &str,
) -> Result<(), String> {
    let primary_key = CredentialKey::for_mailbox(ProviderKind::Qq, address);
    let legacy_key = CredentialKey::legacy_account_id(ProviderKind::Qq, account_id);

    SystemCredentialService.set_secret(&primary_key, authorization_code)?;
    SystemCredentialService.set_secret(&legacy_key, authorization_code)?;

    let saved = saved_qq_authorization_code(&primary_key, account_id)?;
    if saved == authorization_code {
        return Ok(());
    }

    Err("QQ authorization code was accepted, but MailDock could not verify that it was saved. Please import the mailbox again.".into())
}

fn saved_qq_authorization_code(
    credential_key: &CredentialKey,
    account_id: i64,
) -> Result<String, String> {
    if let Some(secret) = SystemCredentialService.get_secret(credential_key)? {
        return Ok(secret);
    }

    let legacy_key = CredentialKey::legacy_account_id(ProviderKind::Qq, account_id);
    if let Some(secret) = SystemCredentialService.get_secret(&legacy_key)? {
        SystemCredentialService.set_secret(credential_key, &secret)?;
        return Ok(secret);
    }

    Err(
        "No saved QQ authorization code found. Import this mailbox again to enable automatic sync."
            .into(),
    )
}
