use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Qq,
    Fudan,
    Gmail,
}

impl ProviderKind {
    pub fn from_db(value: String) -> Result<Self, String> {
        match value.as_str() {
            "qq" => Ok(Self::Qq),
            "fudan" => Ok(Self::Fudan),
            "gmail" => Ok(Self::Gmail),
            unknown => Err(format!("Unknown provider kind: {unknown}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    AuthorizationCode,
    Oauth,
}

impl AuthType {
    pub fn from_db(value: String) -> Result<Self, String> {
        match value.as_str() {
            "authorization_code" => Ok(Self::AuthorizationCode),
            "oauth" => Ok(Self::Oauth),
            unknown => Err(format!("Unknown auth type: {unknown}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MailAccount {
    pub id: i64,
    pub provider: ProviderKind,
    pub address: String,
    pub display_name: String,
    pub auth_type: AuthType,
    pub imap_host: String,
    pub imap_port: i64,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MailMessage {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub received_at: String,
    pub preview: String,
    pub body: String,
    pub has_attachments: bool,
    pub is_unread: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    pub account_id: i64,
    pub folder: String,
    pub last_attempt_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_error: Option<String>,
    pub is_syncing: bool,
}
