use crate::db;
use crate::imap::QqInboxSyncInput;
use crate::models::{MailAccount, ProviderKind};
use crate::providers::{MailProvider, QqImapProvider};
use crate::security::{AppCredentialService, CredentialKey, CredentialService};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

const INBOX_FOLDER: &str = "INBOX";

static SYNC_LOCKS: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStartedEvent {
    pub account_id: i64,
    pub folder: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncFinishedEvent {
    pub account_id: i64,
    pub folder: String,
    pub stored: usize,
    pub last_success_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncFailedEvent {
    pub account_id: i64,
    pub folder: String,
    pub message: String,
    pub last_attempt_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessagesChangedEvent {
    pub account_id: i64,
    pub folder: String,
}

pub fn import_qq_account(
    app: &AppHandle,
    input: QqInboxSyncInput,
) -> Result<QqInboxSyncReport, String> {
    let address = input.email.trim().to_lowercase();
    let authorization_code = input.authorization_code.trim().to_owned();
    let provider = QqImapProvider;
    let payload = provider.sync_inbox(&address, &authorization_code, input.limit)?;
    let account_id = db::upsert_qq_account(app, &address)?;
    let credentials = AppCredentialService::new(app)?;
    save_authorization_code(
        &credentials,
        ProviderKind::Qq,
        account_id,
        &address,
        &authorization_code,
    )?;

    let stored = db::upsert_messages(app, account_id, &payload.folder, &payload.messages)?;
    let last_uid = db::last_message_uid(&payload.messages);
    let last_success_at =
        db::mark_sync_finished(app, account_id, &payload.folder, last_uid.as_deref())?;
    emit_sync_finished(app, account_id, &payload.folder, stored, &last_success_at);
    emit_messages_changed(app, account_id, &payload.folder);

    Ok(QqInboxSyncReport {
        account_id,
        address,
        folder: payload.folder,
        fetched: payload.messages.len(),
        stored,
        total_inbox_messages: payload.total_messages,
        credential_saved: true,
    })
}

pub fn sync_account_now(app: &AppHandle, account_id: i64) -> Result<QqInboxSyncReport, String> {
    let _guard = AccountSyncGuard::acquire(account_id)?;
    let account = db::get_account(app, account_id)?;
    emit_sync_started(app, account.id, INBOX_FOLDER);
    db::mark_sync_started(app, account.id, INBOX_FOLDER)?;

    match sync_account_with_saved_secret(app, &account) {
        Ok(report) => Ok(report),
        Err(error) => {
            let last_attempt_at = db::mark_sync_failed(app, account.id, INBOX_FOLDER, &error)?;
            emit_sync_failed(app, account.id, INBOX_FOLDER, &error, &last_attempt_at);
            Err(error)
        }
    }
}

fn sync_account_with_saved_secret(
    app: &AppHandle,
    account: &MailAccount,
) -> Result<QqInboxSyncReport, String> {
    if account.provider != ProviderKind::Qq {
        return Err("Saved sync is currently only available for QQ Mail accounts.".into());
    }

    let provider = QqImapProvider;
    let credentials = AppCredentialService::new(app)?;
    let credential_key = CredentialKey::for_mailbox(provider.kind(), &account.address);
    let authorization_code =
        saved_authorization_code(&credentials, provider.kind(), &credential_key, account.id)?;
    let payload = provider.sync_inbox(&account.address, &authorization_code, Some(50))?;
    let stored = db::upsert_messages(app, account.id, &payload.folder, &payload.messages)?;
    let last_uid = db::last_message_uid(&payload.messages);
    let last_success_at =
        db::mark_sync_finished(app, account.id, &payload.folder, last_uid.as_deref())?;

    emit_sync_finished(app, account.id, &payload.folder, stored, &last_success_at);
    emit_messages_changed(app, account.id, &payload.folder);

    Ok(QqInboxSyncReport {
        account_id: account.id,
        address: account.address.clone(),
        folder: payload.folder,
        fetched: payload.messages.len(),
        stored,
        total_inbox_messages: payload.total_messages,
        credential_saved: true,
    })
}

fn save_authorization_code(
    credentials: &impl CredentialService,
    provider: ProviderKind,
    account_id: i64,
    address: &str,
    authorization_code: &str,
) -> Result<(), String> {
    let primary_key = CredentialKey::for_mailbox(provider.clone(), address);
    let legacy_key = CredentialKey::legacy_account_id(provider.clone(), account_id);

    credentials.set_secret(&primary_key, authorization_code)?;
    credentials.set_secret(&legacy_key, authorization_code)?;

    let saved = saved_authorization_code(credentials, provider, &primary_key, account_id)?;
    if saved == authorization_code {
        return Ok(());
    }

    Err("Mailbox authorization code was accepted, but MailDock could not verify that it was saved. Please import the mailbox again.".into())
}

fn saved_authorization_code(
    credentials: &impl CredentialService,
    provider: ProviderKind,
    credential_key: &CredentialKey,
    account_id: i64,
) -> Result<String, String> {
    if let Some(secret) = credentials.get_secret(credential_key)? {
        return Ok(secret);
    }

    let legacy_key = CredentialKey::legacy_account_id(provider, account_id);
    if let Some(secret) = credentials.get_secret(&legacy_key)? {
        credentials.set_secret(credential_key, &secret)?;
        return Ok(secret);
    }

    Err(
        "No saved QQ authorization code found. Import this mailbox again to enable automatic sync."
            .into(),
    )
}

fn emit_sync_started(app: &AppHandle, account_id: i64, folder: &str) {
    let _ = app.emit(
        "maildock:sync-started",
        SyncStartedEvent {
            account_id,
            folder: folder.to_owned(),
        },
    );
}

fn emit_sync_finished(
    app: &AppHandle,
    account_id: i64,
    folder: &str,
    stored: usize,
    last_success_at: &str,
) {
    let _ = app.emit(
        "maildock:sync-finished",
        SyncFinishedEvent {
            account_id,
            folder: folder.to_owned(),
            stored,
            last_success_at: last_success_at.to_owned(),
        },
    );
}

fn emit_sync_failed(app: &AppHandle, account_id: i64, folder: &str, message: &str, at: &str) {
    let _ = app.emit(
        "maildock:sync-failed",
        SyncFailedEvent {
            account_id,
            folder: folder.to_owned(),
            message: message.to_owned(),
            last_attempt_at: at.to_owned(),
        },
    );
}

fn emit_messages_changed(app: &AppHandle, account_id: i64, folder: &str) {
    let _ = app.emit(
        "maildock:messages-changed",
        MessagesChangedEvent {
            account_id,
            folder: folder.to_owned(),
        },
    );
}

struct AccountSyncGuard {
    account_id: i64,
}

impl AccountSyncGuard {
    fn acquire(account_id: i64) -> Result<Self, String> {
        let locks = SYNC_LOCKS.get_or_init(|| Mutex::new(HashSet::new()));
        let mut locks = locks
            .lock()
            .map_err(|_| "Failed to lock sync runtime state.".to_string())?;

        if locks.contains(&account_id) {
            return Err("Sync is already running for this account.".into());
        }

        locks.insert(account_id);
        Ok(Self { account_id })
    }
}

impl Drop for AccountSyncGuard {
    fn drop(&mut self) {
        if let Some(locks) = SYNC_LOCKS.get() {
            if let Ok(mut locks) = locks.lock() {
                locks.remove(&self.account_id);
            }
        }
    }
}
