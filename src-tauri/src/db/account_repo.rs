use crate::models::{AuthType, MailAccount, ProviderKind};
use rusqlite::{params, Connection};

pub(crate) fn get_account(connection: &Connection, account_id: i64) -> Result<MailAccount, String> {
    let row = connection
        .query_row(
            "SELECT id, provider, address, display_name, auth_type, imap_host, imap_port, is_enabled
             FROM accounts
             WHERE id = ?1",
            params![account_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            },
        )
        .map_err(|error| format!("Failed to read account {account_id}: {error}"))?;

    mail_account_from_row(row)
}

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
        accounts.push(mail_account_from_row(
            row.map_err(|error| format!("Failed to map account row: {error}"))?,
        )?);
    }

    Ok(accounts)
}

fn mail_account_from_row(
    row: (i64, String, String, String, String, String, i64, i64),
) -> Result<MailAccount, String> {
    let (id, provider, address, display_name, auth_type, imap_host, imap_port, is_enabled) = row;

    Ok(MailAccount {
        id,
        provider: ProviderKind::from_db(provider)?,
        address,
        display_name,
        auth_type: AuthType::from_db(auth_type)?,
        imap_host,
        imap_port,
        is_enabled: is_enabled != 0,
    })
}

pub(crate) fn upsert_qq_account(connection: &Connection, address: &str) -> Result<i64, String> {
    let normalized_address = address.trim().to_lowercase();

    connection
        .execute(
            "INSERT INTO accounts(
                provider,
                address,
                display_name,
                auth_type,
                imap_host,
                imap_port,
                is_enabled
             )
             VALUES ('qq', ?1, 'QQ Mail', 'authorization_code', 'imap.qq.com', 993, 1)
             ON CONFLICT(address) DO UPDATE SET
                provider = excluded.provider,
                display_name = excluded.display_name,
                auth_type = excluded.auth_type,
                imap_host = excluded.imap_host,
                imap_port = excluded.imap_port,
                is_enabled = excluded.is_enabled",
            params![normalized_address],
        )
        .map_err(|error| format!("Failed to save QQ account metadata: {error}"))?;

    connection
        .query_row(
            "SELECT id FROM accounts WHERE address = ?1",
            params![normalized_address],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to read QQ account id: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::initialize_connection;
    use rusqlite::Connection;

    #[test]
    fn lists_no_accounts_before_import() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");

        let accounts = list_accounts(&connection).expect("list accounts");

        assert!(accounts.is_empty());
    }

    #[test]
    fn upserts_real_qq_account_metadata_without_credentials() {
        let connection = Connection::open_in_memory().expect("open test database");
        initialize_connection(&connection).expect("initialize schema");

        let account_id =
            upsert_qq_account(&connection, "Student@qq.com").expect("upsert QQ account");
        let account_id_again =
            upsert_qq_account(&connection, "student@qq.com").expect("upsert same QQ account");

        assert_eq!(account_id, account_id_again);

        let accounts = list_accounts(&connection).expect("list accounts");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].address, "student@qq.com");
        assert_eq!(accounts[0].provider, ProviderKind::Qq);
        assert_eq!(accounts[0].auth_type, AuthType::AuthorizationCode);
    }
}
