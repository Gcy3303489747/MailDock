use crate::models::ProviderKind;
use keyring::{Entry, Error};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SERVICE_NAME: &str = "MailDock";
const LOCAL_CREDENTIAL_HEADER: &str = "maildock-local-credential-v1";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CredentialKey {
    pub provider: ProviderKind,
    pub account_id: Option<i64>,
    pub address: Option<String>,
}

impl CredentialKey {
    pub fn for_mailbox(provider: ProviderKind, address: &str) -> Self {
        Self {
            provider,
            account_id: None,
            address: Some(address.trim().to_lowercase()),
        }
    }

    pub fn legacy_account_id(provider: ProviderKind, account_id: i64) -> Self {
        Self {
            provider,
            account_id: Some(account_id),
            address: None,
        }
    }
}

#[allow(dead_code)]
pub trait CredentialService {
    fn get_secret(&self, key: &CredentialKey) -> Result<Option<String>, String>;
    fn set_secret(&self, key: &CredentialKey, secret: &str) -> Result<(), String>;
    fn delete_secret(&self, key: &CredentialKey) -> Result<(), String>;
}

pub struct SystemCredentialService;

impl CredentialService for SystemCredentialService {
    fn get_secret(&self, key: &CredentialKey) -> Result<Option<String>, String> {
        match credential_entry(key)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("Failed to read saved mailbox credential: {error}")),
        }
    }

    fn set_secret(&self, key: &CredentialKey, secret: &str) -> Result<(), String> {
        credential_entry(key)?
            .set_password(secret)
            .map_err(|error| format!("Failed to save mailbox credential: {error}"))
    }

    fn delete_secret(&self, key: &CredentialKey) -> Result<(), String> {
        match credential_entry(key)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("Failed to delete mailbox credential: {error}")),
        }
    }
}

pub struct AppCredentialService {
    local_credentials_dir: PathBuf,
}

impl AppCredentialService {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("Failed to resolve app data directory: {error}"))?;
        let local_credentials_dir = app_data_dir.join("credentials");

        fs::create_dir_all(&local_credentials_dir)
            .map_err(|error| format!("Failed to create local credential directory: {error}"))?;

        Ok(Self {
            local_credentials_dir,
        })
    }

    fn local_secret_path(&self, key: &CredentialKey) -> PathBuf {
        self.local_credentials_dir
            .join(format!("{}.credential", safe_key_name(key)))
    }
}

impl CredentialService for AppCredentialService {
    fn get_secret(&self, key: &CredentialKey) -> Result<Option<String>, String> {
        match SystemCredentialService.get_secret(key) {
            Ok(Some(secret)) => return Ok(Some(secret)),
            Ok(None) => {}
            Err(error) => {
                eprintln!("System credential read failed; trying local fallback: {error}");
            }
        }

        read_local_secret(self.local_secret_path(key))
    }

    fn set_secret(&self, key: &CredentialKey, secret: &str) -> Result<(), String> {
        match SystemCredentialService.set_secret(key, secret) {
            Ok(()) => {
                if matches!(SystemCredentialService.get_secret(key), Ok(Some(saved)) if saved == secret)
                {
                    return Ok(());
                }
            }
            Err(error) => {
                eprintln!("System credential save failed; using local fallback: {error}");
            }
        }

        write_local_secret(self.local_secret_path(key), secret)
    }

    fn delete_secret(&self, key: &CredentialKey) -> Result<(), String> {
        let _ = SystemCredentialService.delete_secret(key);
        match fs::remove_file(self.local_secret_path(key)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "Failed to delete local mailbox credential: {error}"
            )),
        }
    }
}

fn credential_entry(key: &CredentialKey) -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, &credential_account_name(key))
        .map_err(|error| format!("Failed to open system credential store: {error}"))
}

fn credential_account_name(key: &CredentialKey) -> String {
    if let Some(address) = key.address.as_deref().filter(|value| !value.is_empty()) {
        return format!("{}:{}", provider_key(&key.provider), address);
    }

    match key.account_id {
        Some(account_id) => format!("{}:{}", provider_key(&key.provider), account_id),
        None => format!("{}:unknown", provider_key(&key.provider)),
    }
}

fn provider_key(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Qq => "qq",
        ProviderKind::Fudan => "fudan",
        ProviderKind::Gmail => "gmail",
    }
}

fn safe_key_name(key: &CredentialKey) -> String {
    credential_account_name(key)
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn read_local_secret(path: PathBuf) -> Result<Option<String>, String> {
    let value = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Failed to read local mailbox credential: {error}")),
    };

    let mut lines = value.lines();
    if lines.next() != Some(LOCAL_CREDENTIAL_HEADER) {
        return Err("Local mailbox credential file is not a MailDock credential.".into());
    }

    Ok(Some(lines.collect::<Vec<_>>().join("\n")))
}

fn write_local_secret(path: PathBuf, secret: &str) -> Result<(), String> {
    fs::write(path, format!("{LOCAL_CREDENTIAL_HEADER}\n{secret}"))
        .map_err(|error| format!("Failed to save local mailbox credential: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_stable_address_credential_account_name() {
        let key = CredentialKey::for_mailbox(ProviderKind::Qq, "Student@qq.com");

        assert_eq!(credential_account_name(&key), "qq:student@qq.com");
    }

    #[test]
    fn builds_legacy_credential_account_name() {
        let key = CredentialKey::legacy_account_id(ProviderKind::Qq, 42);

        assert_eq!(credential_account_name(&key), "qq:42");
    }

    #[test]
    fn builds_filesystem_safe_key_name() {
        let key = CredentialKey::for_mailbox(ProviderKind::Qq, "Student@qq.com");

        assert_eq!(safe_key_name(&key), "qq_student_qq_com");
    }
}
