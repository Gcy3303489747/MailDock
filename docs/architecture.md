# MailDock Architecture

MailDock is split into a React frontend and a Rust backend.

## Frontend

The frontend lives in `src/`.

Responsibilities:

- render the mailbox interface
- keep selected message and loading state
- call Tauri commands
- show empty and error states

The frontend should not directly store credentials or connect to IMAP servers.

## Backend

The backend lives in `src-tauri/src/`.

Responsibilities:

- expose Tauri commands to the frontend
- read and write SQLite data
- connect to mail providers through IMAP
- store sensitive credentials through the operating system where possible

## Cached Read Flow

```text
App starts or user selects an account
  -> React calls list_accounts command
  -> React chooses the selected account
  -> React calls list_messages(account_id, folder)
  -> Rust opens local SQLite database
  -> Rust returns cached messages for that account and folder
```

The QQ Mail sync flow is now:

```text
User imports QQ mailbox
  -> React calls sync_qq_inbox(email, authorization_code, limit)
  -> Rust connects to QQ Mail with read-only IMAP
  -> Rust saves the authorization code through the system credential store
  -> Rust stores message data in SQLite
  -> React reloads messages through list_messages command
```

Later syncs use the saved credential:

```text
App starts or user clicks Sync now
  -> React calls sync_saved_qq_inbox(account_id, limit)
  -> Rust loads the QQ authorization code through the security module
  -> Rust syncs INBOX with read-only IMAP
  -> Rust updates SQLite cached messages
  -> React reloads cached messages
```

## Read-Only Mail Rule

The MVP should avoid remote mailbox mutation. The IMAP layer should not implement delete, move, mark-read, or flag update commands until the read-only version is stable.

## SQLite Schema

The first schema uses three tables:

- `accounts`: one row for the learning QQ mailbox
- `messages`: cached INBOX messages
- `sync_state`: future IMAP sync cursor storage

The `accounts` table already includes provider and connection metadata:

- `provider`: `qq`, `fudan`, or `gmail`
- `auth_type`: `authorization_code` or `oauth`
- `imap_host`
- `imap_port`
- `is_enabled`

Authorization codes and OAuth tokens should be referenced through the credential service boundary, not stored in these tables.

## Current Command API

```text
list_accounts() -> MailAccount[]
list_messages(account_id, folder) -> MailMessage[]
test_qq_imap_connection(input) -> ImapConnectionReport
sync_qq_inbox(input) -> QqInboxSyncReport
sync_saved_qq_inbox(input) -> QqInboxSyncReport
```

The browser-only development fallback now starts empty. The Tauri desktop path reads through Rust and SQLite.

## QQ IMAP Connection Test

The provider commands cover both connection testing and read-only syncing:

```text
React form
  -> test_qq_imap_connection(email, authorization_code)
  -> Rust connects to imap.qq.com:993 over TLS
  -> Rust logs in with the QQ Mail authorization code
  -> Rust opens INBOX with EXAMINE, which is read-only
  -> Rust returns mailbox counts
```

```text
React form
  -> sync_qq_inbox(email, authorization_code, limit)
  -> Rust connects to imap.qq.com:993 over TLS
  -> Rust logs in with the QQ Mail authorization code
  -> Rust opens INBOX with EXAMINE, which is read-only
  -> Rust fetches recent messages with BODY.PEEK
  -> Rust saves the authorization code in the system credential store
  -> Rust upserts account metadata and cached messages into SQLite
  -> React reloads accounts and messages from SQLite
```

The authorization code is not stored in SQLite. The `security` module stores and reads it through the operating system credential store, keyed by provider and account id.
