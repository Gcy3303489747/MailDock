use mailparse::{addrparse, addrparse_header, parse_mail, MailAddr, MailHeaderMap, ParsedMail};
use serde::{Deserialize, Serialize};

use crate::mail_text::{clean_text, normalize_body_text, normalize_display_text};
use crate::models::MailMessage;

const QQ_IMAP_HOST: &str = "imap.qq.com";
const QQ_IMAP_PORT: u16 = 993;
const DEFAULT_SYNC_LIMIT: u32 = 50;
const MAX_SYNC_LIMIT: u32 = 100;

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
                "(UID FLAGS INTERNALDATE BODY.PEEK[]<0.65536>)",
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
    let received_at = fetch
        .internal_date()
        .map(|date| date.to_rfc3339())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".into());
    let is_seen = fetch
        .flags()
        .iter()
        .any(|flag| matches!(flag, imap::types::Flag::Seen));

    mail_message_from_raw(email, remote_id, fetch.body()?, received_at, !is_seen)
}

fn mail_message_from_raw(
    email: &str,
    remote_id: u32,
    raw_message: &[u8],
    fallback_received_at: String,
    is_unread: bool,
) -> Option<MailMessage> {
    let parsed = parse_mail(raw_message).ok()?;
    let subject = decoded_header(&parsed, "Subject")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "(No subject)".into());
    let from = decoded_from(&parsed).unwrap_or_else(|| "(Unknown sender)".into());
    let received_at = parsed
        .headers
        .get_first_value("Date")
        .map(clean_text)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback_received_at);
    let body = decoded_text_body(&parsed)
        .map(normalize_body_text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            "MailDock could not load a plain text body for this message yet.".into()
        });
    let preview = preview_from_body(&body);
    let has_attachments = parsed.parts().any(is_attachment_part);

    Some(MailMessage {
        id: format!("qq:{email}:{remote_id}"),
        from,
        subject,
        received_at,
        preview,
        body,
        has_attachments,
        is_unread,
    })
}

fn decoded_header(parsed: &ParsedMail<'_>, key: &str) -> Option<String> {
    let header = parsed.headers.get_first_header(key)?;
    let raw_value = String::from_utf8_lossy(header.get_value_raw());
    let decoded = normalize_display_text(&raw_value);

    if decoded.contains("=?") {
        return Some(normalize_display_text(header.get_value()));
    }

    Some(decoded)
}

fn decoded_from(parsed: &ParsedMail<'_>) -> Option<String> {
    let header = parsed.headers.get_first_header("From")?;
    let parsed_addresses = match addrparse_header(header) {
        Ok(addresses) => addresses,
        Err(_) => decoded_header(parsed, "From").and_then(|decoded| addrparse(&decoded).ok())?,
    };
    let first = parsed_addresses.first()?;

    match first {
        MailAddr::Single(info) => Some(match &info.display_name {
            Some(display_name) if !display_name.trim().is_empty() => {
                format!("{} <{}>", normalize_display_text(display_name), info.addr)
            }
            _ => info.addr.clone(),
        }),
        MailAddr::Group(group) => group.addrs.first().map(|info| match &info.display_name {
            Some(display_name) if !display_name.trim().is_empty() => {
                format!("{} <{}>", normalize_display_text(display_name), info.addr)
            }
            _ => info.addr.clone(),
        }),
    }
}

fn decoded_text_body(parsed: &ParsedMail<'_>) -> Option<String> {
    for part in parsed
        .parts()
        .filter(|part| part.ctype.mimetype.eq_ignore_ascii_case("text/plain"))
    {
        let body = part.get_body().ok()?;
        if !looks_like_multipart_preamble(&body) {
            return Some(body);
        }
    }

    for part in parsed
        .parts()
        .filter(|part| part.ctype.mimetype.eq_ignore_ascii_case("text/html"))
    {
        let body = strip_html_tags(&part.get_body().ok()?);
        if !looks_like_multipart_preamble(&body) {
            return Some(body);
        }
    }

    None
}

fn is_attachment_part(part: &ParsedMail<'_>) -> bool {
    let disposition = part
        .headers
        .get_first_value("Content-Disposition")
        .unwrap_or_default()
        .to_lowercase();
    disposition.contains("attachment")
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

fn looks_like_multipart_preamble(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("this is a multi-part message in mime format")
        || normalized.contains("this is a multipart message in mime format")
}

fn strip_html_tags(value: &str) -> String {
    let mut output = String::new();
    let mut inside_tag = false;

    for character in value.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                output.push(' ');
            }
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }

    output
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn preview_from_body(body: &str) -> String {
    let preview = body.split_whitespace().collect::<Vec<_>>().join(" ");

    if preview.chars().count() <= 160 {
        return preview;
    }

    preview.chars().take(160).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_mime_headers_and_quoted_printable_body() {
        let raw_message = concat!(
            "From: =?UTF-8?B?6YKu5Lu25pyN5Yqh?= <service@qq.com>\r\n",
            "Subject: =?UTF-8?B?5rWL6K+V6YKu5Lu2?=\r\n",
            "Date: Fri, 29 May 2026 09:30:00 +0800\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "Content-Transfer-Encoding: quoted-printable\r\n",
            "\r\n",
            "=E8=BF=99=E6=98=AF=E4=B8=80=E5=B0=81QQ=E9=82=AE=E4=BB=B6"
        );

        let message = mail_message_from_raw(
            "student@qq.com",
            42,
            raw_message.as_bytes(),
            "2026-05-29T09:30:00+08:00".into(),
            true,
        )
        .expect("parse message");

        assert_eq!(message.subject, "测试邮件");
        assert_eq!(message.from, "邮件服务 <service@qq.com>");
        assert_eq!(message.body, "这是一封QQ邮件");
        assert!(!message.body.contains("=E8"));
    }

    #[test]
    fn decodes_folded_lowercase_rfc2047_subject() {
        let raw_message = concat!(
            "From: <service@qq.com>\r\n",
            "Subject: =?utf-8?B?5rWL6K+V?=\r\n",
            " =?utf-8?B?6YKu5Lu2?=\r\n",
            "Date: Fri, 29 May 2026 09:30:00 +0800\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "hello"
        );

        let message = mail_message_from_raw(
            "student@qq.com",
            43,
            raw_message.as_bytes(),
            "2026-05-29T09:30:00+08:00".into(),
            true,
        )
        .expect("parse message");

        assert_eq!(message.subject, "测试邮件");
    }

    #[test]
    fn decodes_wrapped_base64_payload_subject() {
        let raw_message = concat!(
            "From: <service@qq.com>\r\n",
            "Subject: =?utf-8?B?5rWL\r\n",
            " 6K+V6YKu5Lu2?=\r\n",
            "Date: Fri, 29 May 2026 09:30:00 +0800\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "hello"
        );

        let message = mail_message_from_raw(
            "student@qq.com",
            44,
            raw_message.as_bytes(),
            "2026-05-29T09:30:00+08:00".into(),
            true,
        )
        .expect("parse message");

        assert_eq!(message.subject, "测试邮件");
        assert!(!message.subject.contains("=?"));
    }

    #[test]
    fn decodes_mixed_plain_text_and_q_encoded_subject() {
        let raw_message = concat!(
            "From: <service@qq.com>\r\n",
            "Subject: www.donkX.net邮箱验=?utf-8?Q?=E8=AF=81=E7=A0=81?=\r\n",
            "Date: Fri, 29 May 2026 09:30:00 +0800\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "hello"
        );

        let message = mail_message_from_raw(
            "student@qq.com",
            45,
            raw_message.as_bytes(),
            "2026-05-29T09:30:00+08:00".into(),
            true,
        )
        .expect("parse message");

        assert_eq!(message.subject, "www.donkX.net邮箱验证码");
        assert!(!message.subject.contains("=?"));
    }

    #[test]
    fn preserves_body_paragraph_breaks() {
        let raw_message = concat!(
            "From: <service@qq.com>\r\n",
            "Subject: Paragraphs\r\n",
            "Date: Fri, 29 May 2026 09:30:00 +0800\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "First paragraph\r\n",
            "\r\n",
            "> Quoted reply\r\n",
            "\r\n",
            "Second paragraph"
        );

        let message = mail_message_from_raw(
            "student@qq.com",
            46,
            raw_message.as_bytes(),
            "2026-05-29T09:30:00+08:00".into(),
            true,
        )
        .expect("parse message");

        assert_eq!(
            message.body,
            "First paragraph\n\n> Quoted reply\n\nSecond paragraph"
        );
    }

    #[test]
    fn skips_multipart_preamble_body() {
        let raw_message = concat!(
            "From: <service@qq.com>\r\n",
            "Subject: Multipart\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "This is a multi-part message in MIME format.\r\n",
            "------=_NextPart_001"
        );

        let message = mail_message_from_raw(
            "student@qq.com",
            47,
            raw_message.as_bytes(),
            "2026-05-29T09:30:00+08:00".into(),
            true,
        )
        .expect("parse message");

        assert!(!message.body.contains("multi-part message"));
    }
}
