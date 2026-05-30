use rusqlite::{params, Connection, OptionalExtension};

pub(crate) const SEED_ACCOUNT_ADDRESS: &str = "learning@qq.com";

pub(crate) fn cleanup_legacy_seed_data(connection: &Connection) -> Result<(), String> {
    remove_learning_seed(connection)
}

fn remove_learning_seed(connection: &Connection) -> Result<(), String> {
    let account_id = connection
        .query_row(
            "SELECT id FROM accounts WHERE address = ?1",
            params![SEED_ACCOUNT_ADDRESS],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| format!("Failed to inspect learning seed account: {error}"))?;

    let Some(account_id) = account_id else {
        return Ok(());
    };

    connection
        .execute(
            "DELETE FROM messages WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| format!("Failed to remove learning seed messages: {error}"))?;
    connection
        .execute(
            "DELETE FROM sync_state WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| format!("Failed to remove learning seed sync state: {error}"))?;
    connection
        .execute("DELETE FROM accounts WHERE id = ?1", params![account_id])
        .map_err(|error| format!("Failed to remove learning seed account: {error}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::initialize_connection;

    #[test]
    fn removes_legacy_learning_seed_account() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        connection
            .execute(
                "INSERT INTO accounts(provider, address, display_name, auth_type, imap_host, imap_port, is_enabled)
                 VALUES ('qq', ?1, 'QQ Mail', 'authorization_code', 'imap.qq.com', 993, 1)",
                params![SEED_ACCOUNT_ADDRESS],
            )
            .expect("insert learning account");

        cleanup_legacy_seed_data(&connection).expect("cleanup seed");

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
            .expect("count accounts");
        assert_eq!(count, 0);
    }
}
