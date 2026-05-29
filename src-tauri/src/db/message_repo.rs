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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::account_repo::list_accounts;
    use crate::db::connection::initialize_connection;
    use crate::db::seed::seed_database;
    use rusqlite::Connection;

    #[test]
    fn lists_seeded_inbox_messages_for_account_and_folder() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        seed_database(&connection).expect("seed database");
        let account = list_accounts(&connection)
            .expect("list accounts")
            .pop()
            .expect("seeded account");

        let messages = list_messages(&connection, account.id, "INBOX").expect("list messages");

        assert_eq!(messages.len(), 5);
        assert_eq!(messages[0].id, "qq-001");
    }

    #[test]
    fn does_not_return_messages_for_other_folder() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        seed_database(&connection).expect("seed database");
        let account = list_accounts(&connection)
            .expect("list accounts")
            .pop()
            .expect("seeded account");

        let messages = list_messages(&connection, account.id, "Archive").expect("list messages");

        assert!(messages.is_empty());
    }
}
