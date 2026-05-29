mod commands;
mod db;
mod imap;
mod models;
mod security;

use commands::mail::{list_accounts, list_messages};
use commands::provider::test_qq_imap_connection;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_accounts,
            list_messages,
            test_qq_imap_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running MailDock");
}
