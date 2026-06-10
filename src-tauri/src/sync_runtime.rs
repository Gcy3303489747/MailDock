use crate::{db, mail_service};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;

const AUTO_SYNC_INTERVAL: Duration = Duration::from_secs(5 * 60);
const STARTUP_SYNC_DELAY: Duration = Duration::from_secs(3);

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        thread::sleep(STARTUP_SYNC_DELAY);

        loop {
            sync_all_enabled_accounts(&app);
            thread::sleep(AUTO_SYNC_INTERVAL);
        }
    });
}

fn sync_all_enabled_accounts(app: &AppHandle) {
    let accounts = match db::list_accounts(app) {
        Ok(accounts) => accounts,
        Err(error) => {
            eprintln!("Failed to list accounts for background sync: {error}");
            return;
        }
    };

    for account in accounts {
        if let Err(error) = mail_service::sync_account_now(app, account.id) {
            if error != "Sync is already running for this account." {
                eprintln!("Background sync failed for {}: {error}", account.address);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_five_minute_auto_sync_interval() {
        assert_eq!(AUTO_SYNC_INTERVAL, Duration::from_secs(300));
    }
}
