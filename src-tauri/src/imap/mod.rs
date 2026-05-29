use serde::{Deserialize, Serialize};

use crate::models::MailMessage;

const QQ_IMAP_HOST: &str = "imap.qq.com";
const QQ_IMAP_PORT: u16 = 993;
const DEFAULT_SYNC_LIMIT: u32 = 50;
const MAX_SYNC_LIMIT: u32 = 100;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QqImapConnectionInput {
    pub email: String,
    pub authorization_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QqInboxSyncInput {
    pub email: String,
    pub authorization_code: String,
    pub limit: Option<u32>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QqInboxSyncPayload {
    pub report: ImapConnectionReport,
    pub messages: Vec<MailMessage>,
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

pub fn sync_qq_inbox(input: QqInboxSyncInput) -> Result<QqInboxSyncPayload, String> {
    validate_credentials(input.email.trim(), input.authorization_code.trim())?;

    let limit = input
        .limit
        .unwrap_or(DEFAULT_SYNC_LIMIT)
        .clamp(1, MAX_SYNC_LIMIT);
    let email = input.email.trim().to_lowercase();

    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|error| format!("Failed to create TLS connector: {error}"))?;

    let client = imap::connect((QQ_IMAP_HOST, QQ_IMAP_PORT), QQ_IMAP_HOST, &tls)
        .map_err(|error| format!("Failed to connect to QQ IMAP: {error}"))?;

    let mut session = client
        .login(email.as_str(), input.authorization_code.trim())
        .map_err(|(error, _client)| format!("QQ IMAP login failed: {error}"))?;

    let mailbox = session
        .examine("INBOX")
        .map_err(|error| format!("Failed to open INBOX in read-only mode: {error}"))?;

    let messages = if mailbox.exists == 0 {
        Vec::new()
    } else {
        let start = mailbox.exists.saturating_sub(limit).saturating_add(1);
        let sequence_set = format!("{start}:{}", mailbox.exists);
        let fetches = session
            .fetch(
                sequence_set,
                "(UID FLAGS INTERNALDATE ENVELOPE BODY.PEEK[TEXT]<0.8192>)",
            )
            .map_err(|error| format!("Failed to fetch QQ inbox messages: {error}"))?;

        fetches
            .iter()
            .filter_map(|fetch| mail_message_from_fetch(&email, fetch))
            .collect()
    };

    let _ = session.logout();

    Ok(QqInboxSyncPayload {
        report: ImapConnectionReport {
            provider: "qq".into(),
            host: QQ_IMAP_HOST.into(),
            port: QQ_IMAP_PORT,
            folder: "INBOX".into(),
            exists: mailbox.exists,
            recent: mailbox.recent,
            unseen: mailbox.unseen,
        },
        messages,
    })
}

fn mail_message_from_fetch(email: &str, fetch: &imap::types::Fetch) -> Option<MailMessage> {
    let remote_id = fetch.uid.unwrap_or(fetch.message);
    let envelope = fetch.envelope();
    let subject = envelope
        .and_then(|value| value.subject)
        .map(bytes_to_clean_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "(No subject)".into());
    let from = envelope
        .and_then(|value| value.from.as_ref())
        .and_then(|addresses| addresses.first())
        .map(|address| {
            let display_name = address.name.map(bytes_to_clean_string).unwrap_or_default();
            let mailbox = address
                .mailbox
                .map(bytes_to_clean_string)
                .unwrap_or_default();
            let host = address.host.map(bytes_to_clean_string).unwrap_or_default();
            let address_text = if mailbox.is_empty() || host.is_empty() {
                String::new()
            } else {
                format!("{mailbox}@{host}")
            };

            match (display_name.is_empty(), address_text.is_empty()) {
                (true, true) => "(Unknown sender)".into(),
                (true, false) => address_text,
                (false, true) => display_name,
                (false, false) => format!("{display_name} <{address_text}>"),
            }
        })
        .unwrap_or_else(|| "(Unknown sender)".into());
    let received_at = fetch
        .internal_date()
        .map(|date| date.to_rfc3339())
        .or_else(|| {
            envelope
                .and_then(|value| value.date)
                .map(bytes_to_clean_string)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".into());
    let body = fetch
        .text()
        .map(bytes_to_clean_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            "MailDock could not load a plain text body for this message yet.".into()
        });
    let preview = preview_from_body(&body);
    let is_seen = fetch
        .flags()
        .iter()
        .any(|flag| matches!(flag, imap::types::Flag::Seen));

    Some(MailMessage {
        id: format!("qq:{email}:{remote_id}"),
        from,
        subject,
        received_at,
        preview,
        body,
        has_attachments: false,
        is_unread: !is_seen,
    })
}

fn validate_input(input: &QqImapConnectionInput) -> Result<(), String> {
    validate_credentials(input.email.trim(), input.authorization_code.trim())
}

fn validate_credentials(email: &str, authorization_code: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email address is required.".into());
    }

    if authorization_code.is_empty() {
        return Err("QQ Mail authorization code is required.".into());
    }

    if !email.contains('@') {
        return Err("Email address should include @.".into());
    }

    Ok(())
}

fn bytes_to_clean_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn preview_from_body(body: &str) -> String {
    let preview = body.split_whitespace().collect::<Vec<_>>().join(" ");

    if preview.chars().count() <= 160 {
        return preview;
    }

    preview.chars().take(160).collect::<String>()
}
