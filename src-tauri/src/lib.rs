mod commands;
mod db;
mod imap;
mod mail_service;
mod mail_text;
mod models;
mod providers;
mod security;
mod sync_runtime;
mod time_utils;

use commands::mail::{get_sync_state, list_accounts, list_messages};
use commands::provider::{sync_account_now, sync_qq_inbox, sync_saved_qq_inbox};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            sync_runtime::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sync_state,
            list_accounts,
            list_messages,
            sync_account_now,
            sync_qq_inbox,
            sync_saved_qq_inbox
        ])
        .run(tauri::generate_context!())
        .expect("error while running MailDock");
}
