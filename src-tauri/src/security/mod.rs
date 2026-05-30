use crate::models::ProviderKind;
use keyring::{Entry, Error};

const SERVICE_NAME: &str = "MailDock";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CredentialKey {
    pub account_id: i64,
    pub provider: ProviderKind,
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

fn credential_entry(key: &CredentialKey) -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, &credential_account_name(key))
        .map_err(|error| format!("Failed to open system credential store: {error}"))
}

fn credential_account_name(key: &CredentialKey) -> String {
    format!("{}:{}", provider_key(&key.provider), key.account_id)
}

fn provider_key(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Qq => "qq",
        ProviderKind::Fudan => "fudan",
        ProviderKind::Gmail => "gmail",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_stable_credential_account_name() {
        let key = CredentialKey {
            account_id: 42,
            provider: ProviderKind::Qq,
        };

        assert_eq!(credential_account_name(&key), "qq:42");
    }
}
