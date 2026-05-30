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
             ORDER BY received_at DESC
             LIMIT 100",
        )
        .map_err(|error| format!("Failed to prepare message query: {error}"))?;

    let rows = statement
        .query_map(params![account_id, folder], |row| {
            Ok(MailMessage {
                id: row.get(0)?,
                from: row.get(1)?,
                subject: row.get(2)?,
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
    let mut last_uid: Option<String> = None;

    for message in messages {
        connection
            .execute(
                "INSERT INTO messages(
                    id,
                    account_id,
                    folder,
                    from_address,
                    subject,
                    received_at,
                    preview,
                    body,
                    has_attachments,
                    is_unread
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                    from_address = excluded.from_address,
                    subject = excluded.subject,
                    received_at = excluded.received_at,
                    preview = excluded.preview,
                    body = excluded.body,
                    has_attachments = excluded.has_attachments,
                    is_unread = excluded.is_unread",
                params![
                    message.id,
                    account_id,
                    folder,
                    message.from,
                    message.subject,
                    message.received_at,
                    message.preview,
                    message.body,
                    message.has_attachments as i64,
                    message.is_unread as i64,
                ],
            )
            .map_err(|error| format!("Failed to save message {}: {error}", message.id))?;

        stored_count += 1;
        last_uid = message.id.rsplit(':').next().map(str::to_owned);
    }

    connection
        .execute(
            "INSERT INTO sync_state(account_id, folder, last_synced_at, last_uid)
             VALUES (?1, ?2, CURRENT_TIMESTAMP, ?3)
             ON CONFLICT(account_id, folder) DO UPDATE SET
                last_synced_at = excluded.last_synced_at,
                last_uid = excluded.last_uid",
            params![account_id, folder, last_uid],
        )
        .map_err(|error| format!("Failed to update sync state: {error}"))?;

    Ok(stored_count)
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
}
