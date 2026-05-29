use crate::models::{AuthType, MailAccount, ProviderKind};
use rusqlite::Connection;

pub(crate) fn list_accounts(connection: &Connection) -> Result<Vec<MailAccount>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, provider, address, display_name, auth_type, imap_host, imap_port, is_enabled
             FROM accounts
             WHERE is_enabled = 1
             ORDER BY id ASC",
        )
        .map_err(|error| format!("Failed to prepare account query: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            let provider: String = row.get(1)?;
            let auth_type: String = row.get(4)?;

            Ok((
                row.get::<_, i64>(0)?,
                provider,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                auth_type,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })
        .map_err(|error| format!("Failed to read accounts: {error}"))?;

    let mut accounts = Vec::new();
    for row in rows {
        let (id, provider, address, display_name, auth_type, imap_host, imap_port, is_enabled) =
            row.map_err(|error| format!("Failed to map account row: {error}"))?;

        accounts.push(MailAccount {
            id,
            provider: ProviderKind::from_db(provider)?,
            address,
            display_name,
            auth_type: AuthType::from_db(auth_type)?,
            imap_host,
            imap_port,
            is_enabled: is_enabled != 0,
        });
    }

    Ok(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::initialize_connection;
    use crate::db::seed::seed_database;
    use rusqlite::Connection;

    #[test]
    fn lists_seeded_qq_account() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");
        seed_database(&connection).expect("seed database");

        let accounts = list_accounts(&connection).expect("list accounts");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].provider, ProviderKind::Qq);
        assert_eq!(accounts[0].auth_type, AuthType::AuthorizationCode);
        assert_eq!(accounts[0].address, "learning@qq.com");
        assert_eq!(accounts[0].imap_host, "imap.qq.com");
        assert_eq!(accounts[0].imap_port, 993);
    }
}
