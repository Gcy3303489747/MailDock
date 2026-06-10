use crate::mail_text::normalize_display_text;
use crate::models::MailMessage;
use rusqlite::{params, Connection};

pub(crate) fn list_messages(
    connection: &Connection,
    account_id: i64,
    folder: &str,
) -> Result<Vec<MailMessage>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, from_address, subject, received_at, preview, body, has_attachments, is_unread
             FROM messages
             WHERE account_id = ?1 AND folder = ?2
             ORDER BY received_at_epoch DESC, received_at DESC
             LIMIT 100",
        )
        .map_err(|error| format!("Failed to prepare message query: {error}"))?;

    let rows = statement
        .query_map(params![account_id, folder], |row| {
            Ok(MailMessage {
                id: row.get(0)?,
                from: normalize_display_text(row.get::<_, String>(1)?),
                subject: normalize_display_text(row.get::<_, String>(2)?),
                received_at: row.get(3)?,
                preview: row.get(4)?,
                body: row.get(5)?,
                has_attachments: row.get::<_, i64>(6)? != 0,
                is_unread: row.get::<_, i64>(7)? != 0,
            })
        })
        .map_err(|error| format!("Failed to read messages: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to map messages: {error}"))
}

pub(crate) fn upsert_messages(
    connection: &Connection,
    account_id: i64,
    folder: &str,
    messages: &[MailMessage],
) -> Result<usize, String> {
    let mut stored_count = 0;

    for message in messages {
        connection
            .execute(
                "INSERT INTO messages(
                    id,
                    account_id,
                    folder,
                    provider_message_id,
                    uid,
                    from_address,
                    subject,
                    received_at,
                    received_at_epoch,
                    preview,
                    body,
                    has_attachments,
                    is_unread
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(id) DO UPDATE SET
                    provider_message_id = excluded.provider_message_id,
                    uid = excluded.uid,
                    from_address = excluded.from_address,
                    subject = excluded.subject,
                    received_at = excluded.received_at,
                    received_at_epoch = excluded.received_at_epoch,
                    preview = excluded.preview,
                    body = excluded.body,
                    has_attachments = excluded.has_attachments,
                    is_unread = excluded.is_unread",
                params![
                    message.id,
                    account_id,
                    folder,
                    message.id,
                    message_uid(&message.id),
                    message.from,
                    message.subject,
                    message.received_at,
                    crate::time_utils::received_at_sort_key(&message.received_at),
                    message.preview,
                    message.body,
                    message.has_attachments as i64,
                    message.is_unread as i64,
                ],
            )
            .map_err(|error| format!("Failed to save message {}: {error}", message.id))?;

        stored_count += 1;
    }

    Ok(stored_count)
}

pub(crate) fn last_message_uid(messages: &[MailMessage]) -> Option<String> {
    messages.last().map(|message| message_uid(&message.id))
}

fn message_uid(message_id: &str) -> String {
    message_id
        .rsplit(':')
        .next()
        .unwrap_or(message_id)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::account_repo::upsert_qq_account;
    use crate::db::connection::initialize_connection;
    use rusqlite::Connection;

    #[test]
    fn lists_synced_inbox_messages_for_account_and_folder() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        let account_id =
            upsert_qq_account(&connection, "student@qq.com").expect("upsert QQ account");
        upsert_messages(
            &connection,
            account_id,
            "INBOX",
            &[MailMessage {
                id: "qq:student@qq.com:1".into(),
                from: "Teacher <teacher@example.com>".into(),
                subject: "Real inbox sample".into(),
                received_at: "2026-05-29T08:00:00+08:00".into(),
                preview: "Fetched from IMAP in read-only mode.".into(),
                body: "Fetched from IMAP in read-only mode.".into(),
                has_attachments: false,
                is_unread: true,
            }],
        )
        .expect("upsert messages");

        let messages = list_messages(&connection, account_id, "INBOX").expect("list messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "qq:student@qq.com:1");
    }

    #[test]
    fn does_not_return_messages_for_other_folder() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        let account_id =
            upsert_qq_account(&connection, "student@qq.com").expect("upsert QQ account");

        let messages = list_messages(&connection, account_id, "Archive").expect("list messages");

        assert!(messages.is_empty());
    }

    #[test]
    fn upserts_synced_messages_for_one_account_and_folder() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        let account_id =
            upsert_qq_account(&connection, "student@qq.com").expect("upsert QQ account");

        let synced = vec![MailMessage {
            id: "qq:student@qq.com:10".into(),
            from: "Teacher <teacher@example.com>".into(),
            subject: "Real inbox sample".into(),
            received_at: "2026-05-29T08:00:00+08:00".into(),
            preview: "Fetched from IMAP in read-only mode.".into(),
            body: "Fetched from IMAP in read-only mode.".into(),
            has_attachments: false,
            is_unread: true,
        }];

        let stored =
            upsert_messages(&connection, account_id, "INBOX", &synced).expect("upsert messages");
        let messages = list_messages(&connection, account_id, "INBOX").expect("list messages");

        assert_eq!(stored, 1);
        assert!(messages
            .iter()
            .any(|message| message.id == "qq:student@qq.com:10"));
    }

    #[test]
    fn lists_newest_messages_first() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        let account_id =
            upsert_qq_account(&connection, "student@qq.com").expect("upsert QQ account");

        upsert_messages(
            &connection,
            account_id,
            "INBOX",
            &[
                MailMessage {
                    id: "qq:student@qq.com:older".into(),
                    from: "Older <older@example.com>".into(),
                    subject: "Older".into(),
                    received_at: "2026-05-29T08:00:00+08:00".into(),
                    preview: "Older message.".into(),
                    body: "Older message.".into(),
                    has_attachments: false,
                    is_unread: false,
                },
                MailMessage {
                    id: "qq:student@qq.com:newer".into(),
                    from: "Newer <newer@example.com>".into(),
                    subject: "Newer".into(),
                    received_at: "2026-05-29T10:00:00+08:00".into(),
                    preview: "Newer message.".into(),
                    body: "Newer message.".into(),
                    has_attachments: false,
                    is_unread: true,
                },
            ],
        )
        .expect("upsert messages");

        let messages = list_messages(&connection, account_id, "INBOX").expect("list messages");

        assert_eq!(messages[0].id, "qq:student@qq.com:newer");
        assert_eq!(messages[1].id, "qq:student@qq.com:older");
    }

    #[test]
    fn sorts_cached_rfc2822_dates_with_rfc3339_dates() {
        assert!(
            crate::time_utils::received_at_sort_key("Fri, 29 May 2026 09:30:00 +0800")
                > crate::time_utils::received_at_sort_key("2026-05-29T08:00:00+08:00")
        );
    }

    #[test]
    fn decodes_cached_mixed_q_encoded_subject_on_read() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        let account_id =
            upsert_qq_account(&connection, "student@qq.com").expect("upsert QQ account");

        upsert_messages(
            &connection,
            account_id,
            "INBOX",
            &[MailMessage {
                id: "qq:student@qq.com:20".into(),
                from: "=?utf-8?Q?=E9=82=AE=E4=BB=B6=E6=9C=8D=E5=8A=A1?= <service@qq.com>".into(),
                subject: "www.donkX.net邮箱验=?utf-8?Q?=E8=AF=81=E7=A0=81?=".into(),
                received_at: "2026-05-29T08:00:00+08:00".into(),
                preview: "Cached encoded subject sample.".into(),
                body: "Cached encoded subject sample.".into(),
                has_attachments: false,
                is_unread: true,
            }],
        )
        .expect("upsert messages");

        let messages = list_messages(&connection, account_id, "INBOX").expect("list messages");

        assert_eq!(messages[0].subject, "www.donkX.net邮箱验证码");
        assert_eq!(messages[0].from, "邮件服务 <service@qq.com>");
        assert!(!messages[0].subject.contains("=?"));
    }
}
