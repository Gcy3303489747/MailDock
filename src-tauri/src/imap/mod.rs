use serde::{Deserialize, Serialize};

const QQ_IMAP_HOST: &str = "imap.qq.com";
const QQ_IMAP_PORT: u16 = 993;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QqImapConnectionInput {
    pub email: String,
    pub authorization_code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImapConnectionReport {
    pub provider: String,
    pub host: String,
    pub port: u16,
    pub folder: String,
    pub exists: u32,
    pub recent: u32,
    pub unseen: Option<u32>,
}

pub fn test_qq_connection(input: QqImapConnectionInput) -> Result<ImapConnectionReport, String> {
    validate_input(&input)?;

    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|error| format!("Failed to create TLS connector: {error}"))?;

    let client = imap::connect((QQ_IMAP_HOST, QQ_IMAP_PORT), QQ_IMAP_HOST, &tls)
        .map_err(|error| format!("Failed to connect to QQ IMAP: {error}"))?;

    let mut session = client
        .login(input.email.trim(), input.authorization_code.trim())
        .map_err(|(error, _client)| format!("QQ IMAP login failed: {error}"))?;

    let mailbox = session
        .examine("INBOX")
        .map_err(|error| format!("Failed to open INBOX in read-only mode: {error}"))?;

    let _ = session.logout();

    Ok(ImapConnectionReport {
        provider: "qq".into(),
        host: QQ_IMAP_HOST.into(),
        port: QQ_IMAP_PORT,
        folder: "INBOX".into(),
        exists: mailbox.exists,
        recent: mailbox.recent,
        unseen: mailbox.unseen,
    })
}

fn validate_input(input: &QqImapConnectionInput) -> Result<(), String> {
    if input.email.trim().is_empty() {
        return Err("Email address is required.".into());
    }

    if input.authorization_code.trim().is_empty() {
        return Err("QQ Mail authorization code is required.".into());
    }

    if !input.email.contains('@') {
        return Err("Email address should include @.".into());
    }

    Ok(())
}
