use crate::models::ProviderKind;

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

#[allow(dead_code)]
pub struct NotConfiguredCredentialService;

impl CredentialService for NotConfiguredCredentialService {
    fn get_secret(&self, _key: &CredentialKey) -> Result<Option<String>, String> {
        Err("Credential storage is not implemented yet.".into())
    }

    fn set_secret(&self, _key: &CredentialKey, _secret: &str) -> Result<(), String> {
        Err("Credential storage is not implemented yet.".into())
    }

    fn delete_secret(&self, _key: &CredentialKey) -> Result<(), String> {
        Err("Credential storage is not implemented yet.".into())
    }
}
