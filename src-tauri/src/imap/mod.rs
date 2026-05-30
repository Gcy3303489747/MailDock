use base64::{engine::general_purpose, Engine as _};
use encoding_rs::Encoding;
use mailparse::{addrparse_header, parse_mail, MailAddr, MailHeaderMap, ParsedMail};
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
        .map(normalize_display_text)
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
    let decoded = decode_rfc2047_words(&raw_value).unwrap_or_else(|| header.get_value());
    Some(normalize_display_text(decoded))
}

fn decoded_from(parsed: &ParsedMail<'_>) -> Option<String> {
    let header = parsed.headers.get_first_header("From")?;
    let parsed_addresses = addrparse_header(header).ok()?;
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

fn normalize_display_text(value: impl AsRef<str>) -> String {
    repair_utf8_decoded_as_gbk(&clean_text(value))
}

fn clean_text(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_rfc2047_words(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut remainder = value;
    let mut decoded_any = false;
    let mut previous_was_encoded = false;

    while let Some(start) = remainder.find("=?") {
        let before = &remainder[..start];
        if !(previous_was_encoded && before.trim().is_empty()) {
            output.push_str(before);
        }

        let encoded_remainder = &remainder[start + 2..];
        let Some(end) = encoded_remainder.find("?=") else {
            output.push_str(&remainder[start..]);
            return decoded_any.then_some(output);
        };

        let encoded_word = &encoded_remainder[..end];
        if let Some(decoded_word) = decode_rfc2047_word(encoded_word) {
            output.push_str(&decoded_word);
            decoded_any = true;
            previous_was_encoded = true;
        } else {
            output.push_str("=?");
            output.push_str(encoded_word);
            output.push_str("?=");
            previous_was_encoded = false;
        }

        remainder = &encoded_remainder[end + 2..];
    }

    output.push_str(remainder);
    decoded_any.then_some(output)
}

fn decode_rfc2047_word(encoded_word: &str) -> Option<String> {
    let mut parts = encoded_word.splitn(3, '?');
    let charset = parts.next()?.trim();
    let encoding = parts.next()?.trim();
    let payload = parts.next()?.trim();

    let bytes = match encoding.to_ascii_lowercase().as_str() {
        "b" => general_purpose::STANDARD.decode(payload).ok()?,
        "q" => decode_rfc2047_q_payload(payload)?,
        _ => return None,
    };

    decode_charset(charset, &bytes)
}

fn decode_rfc2047_q_payload(payload: &str) -> Option<Vec<u8>> {
    let bytes = payload.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'_' => {
                decoded.push(b' ');
                index += 1;
            }
            b'=' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
                decoded.push(u8::from_str_radix(hex, 16).ok()?);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    Some(decoded)
}

fn decode_charset(charset: &str, bytes: &[u8]) -> Option<String> {
    let encoding = Encoding::for_label(charset.as_bytes())?;
    let (decoded, _, had_errors) = encoding.decode(bytes);

    if had_errors {
        return None;
    }

    Some(decoded.into_owned())
}

fn repair_utf8_decoded_as_gbk(value: &str) -> String {
    let (encoded, _, had_errors) = encoding_rs::GBK.encode(value);
    if had_errors {
        return value.to_owned();
    }

    String::from_utf8(encoded.into_owned()).unwrap_or_else(|_| value.to_owned())
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
            44,
            raw_message.as_bytes(),
            "2026-05-29T09:30:00+08:00".into(),
            true,
        )
        .expect("parse message");

        assert!(!message.body.contains("multi-part message"));
    }
}
