# Week 03: QQ IMAP Sync

This milestone connects MailDock to real QQ Mail data while keeping the MVP read-only.

## What Changed

- Added `sync_qq_inbox` as a Tauri command.
- Used QQ Mail IMAP over TLS at `imap.qq.com:993`.
- Opened `INBOX` with `EXAMINE` so the app does not modify the remote mailbox.
- Fetched recent messages with `BODY.PEEK` so reading from the app does not mark mail as read.
- Saved fetched message metadata and text preview into SQLite.
- Reloaded the React UI from SQLite after sync.
- Saved the QQ authorization code through the system credential store after a successful import.
- Added saved-credential sync for app startup and the `Sync now` button.

## Concepts Learned

- IMAP is the protocol used to read mailboxes from a mail server.
- `EXAMINE` opens a mailbox in read-only mode.
- `BODY.PEEK` reads message content without changing the remote read state.
- A Tauri command is the bridge between React and Rust.
- SQLite is acting as a local cache, not as credential storage.
- The operating system credential store is a better place for mailbox secrets than SQLite.

## Security Note

The QQ authorization code is not saved to SQLite. MailDock stores it behind the Rust `security` module using the operating system credential store, keyed by provider and account id.
