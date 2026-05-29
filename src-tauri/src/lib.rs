mod commands;
mod db;
mod models;
mod security;

use commands::mail::{list_accounts, list_messages};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_accounts, list_messages])
        .run(tauri::generate_context!())
        .expect("error while running MailDock");
}
