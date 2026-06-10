use crate::models::ProviderKind;
use base64::prelude::*;
use keyring::{Entry, Error};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use windows::core::w;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

const SERVICE_NAME: &str = "MailDock";
const LOCAL_CREDENTIAL_HEADER: &str = "maildock-local-credential-dpapi-v1";
const LEGACY_LOCAL_CREDENTIAL_HEADER: &str = "maildock-local-credential-v1";

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
    let value = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Failed to read local mailbox credential: {error}")),
    };

    let mut lines = value.lines();
    match lines.next() {
        Some(LOCAL_CREDENTIAL_HEADER) => {
            let encoded = lines.collect::<Vec<_>>().join("");
            let encrypted = BASE64_STANDARD
                .decode(encoded)
                .map_err(|error| format!("Failed to decode local mailbox credential: {error}"))?;
            decrypt_with_dpapi(&encrypted).map(Some)
        }
        Some(LEGACY_LOCAL_CREDENTIAL_HEADER) => {
            let secret = lines.collect::<Vec<_>>().join("\n");
            write_local_secret(path.clone(), &secret)?;
            Ok(Some(secret))
        }
        _ => Err("Local mailbox credential file is not a MailDock credential.".into()),
    }
}

fn write_local_secret(path: PathBuf, secret: &str) -> Result<(), String> {
    let encrypted = encrypt_with_dpapi(secret.as_bytes())?;
    let encoded = BASE64_STANDARD.encode(encrypted);
    fs::write(path, format!("{LOCAL_CREDENTIAL_HEADER}\n{encoded}"))
        .map_err(|error| format!("Failed to save local mailbox credential: {error}"))
}

fn encrypt_with_dpapi(value: &[u8]) -> Result<Vec<u8>, String> {
    let mut input = CRYPT_INTEGER_BLOB {
        cbData: value.len() as u32,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptProtectData(
            &mut input,
            w!("MailDock mailbox credential"),
            None,
            None,
            None,
            0,
            &mut output,
        )
        .map_err(|error| format!("Failed to encrypt local mailbox credential: {error}"))?;
    }

    blob_to_vec_and_free(output)
}

fn decrypt_with_dpapi(value: &[u8]) -> Result<String, String> {
    let mut input = CRYPT_INTEGER_BLOB {
        cbData: value.len() as u32,
        pbData: value.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(&mut input, None, None, None, None, 0, &mut output)
            .map_err(|error| format!("Failed to decrypt local mailbox credential: {error}"))?;
    }

    let decrypted = blob_to_vec_and_free(output)?;
    String::from_utf8(decrypted)
        .map_err(|error| format!("Local mailbox credential is not valid UTF-8: {error}"))
}

fn blob_to_vec_and_free(blob: CRYPT_INTEGER_BLOB) -> Result<Vec<u8>, String> {
    if blob.pbData.is_null() {
        return Err("Local mailbox credential file is not a MailDock credential.".into());
    }

    let bytes = unsafe { std::slice::from_raw_parts(blob.pbData, blob.cbData as usize).to_vec() };
    unsafe {
        let _ = LocalFree(Some(HLOCAL(blob.pbData.cast())));
    }
    Ok(bytes)
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

    #[test]
    fn encrypts_and_decrypts_local_secret_with_dpapi() {
        let encrypted = encrypt_with_dpapi(b"qq-secret").expect("encrypt secret");

        assert_ne!(encrypted, b"qq-secret");
        assert_eq!(
            decrypt_with_dpapi(&encrypted).expect("decrypt secret"),
            "qq-secret"
        );
    }

    #[test]
    fn migrates_legacy_plaintext_local_secret() {
        let path = std::env::temp_dir().join(format!(
            "maildock-legacy-credential-{}.credential",
            std::process::id()
        ));
        fs::write(
            &path,
            format!("{LEGACY_LOCAL_CREDENTIAL_HEADER}\nlegacy-secret"),
        )
        .expect("write legacy credential");

        let secret = read_local_secret(path.clone())
            .expect("read local secret")
            .expect("secret exists");
        let migrated = fs::read_to_string(&path).expect("read migrated credential");
        let _ = fs::remove_file(&path);

        assert_eq!(secret, "legacy-secret");
        assert!(migrated.starts_with(LOCAL_CREDENTIAL_HEADER));
        assert!(!migrated.contains("legacy-secret"));
    }
}
