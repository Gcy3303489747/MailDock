# MailDock

MailDock is a learning-driven Windows desktop mail aggregator built with Tauri, React, TypeScript, Rust, and SQLite.

The first version is intentionally small: QQ Mail, read-only inbox, local cache, saved authorization code, and one-click sync. The goal is to build something personally useful while learning desktop app development one milestone at a time.

## Why I Built This

I am building MailDock as a long-term personal project to learn:

- React component design
- TypeScript types
- Tauri desktop app structure
- Rust backend commands
- SQLite local storage
- IMAP email synchronization
- practical security habits for credentials

## MVP

- QQ Mail inbox only
- Read-only IMAP sync
- Local SQLite cache
- Saved QQ authorization code in the system credential store
- One-click sync
- Windows desktop app

Not included in the MVP:

- sending email
- deleting or moving email
- multi-account inbox
- notifications
- advanced search

## Tech Stack

- Tauri v2
- React
- TypeScript
- Rust
- SQLite
- IMAP over TLS

## Current Progress

- [x] Project skeleton
- [x] Mock mailbox UI
- [x] Tauri command boundary
- [x] SQLite cache foundation
- [x] QQ Mail read-only IMAP sync to SQLite
- [x] Saved QQ authorization code through the system credential store
- [ ] Windows installer

## Architecture

```text
React UI -> Tauri commands -> Rust backend -> SQLite
                                  |
                                  -> QQ Mail IMAP
```

The MVP keeps React focused on interface state. Rust owns the local database, IMAP access, and credential storage.

The current SQLite milestone creates:

- `accounts`
- `messages`
- `sync_state`

The app starts with no configured mailbox. Use the top-left menu to import a QQ mailbox, then the UI reads from the same SQLite cache used by IMAP sync. The command boundary is account-aware:

```text
list_accounts()
list_messages(account_id, folder)
sync_qq_inbox(email, authorization_code, limit)
sync_saved_qq_inbox(account_id, limit)
```

The first real sync command connects to QQ Mail with `EXAMINE INBOX` and `BODY.PEEK`, saves the latest messages into SQLite, then the React UI reloads from the local cache. After a successful import, the QQ authorization code is saved through the operating system credential store so later syncs do not require typing it again.

The `accounts` table stores provider metadata such as `qq`, `fudan`, or `gmail`, plus the future IMAP host and auth type. Secrets still do not belong in SQLite.

## Local Development

Install the required Windows development tools first. See [Windows Development Setup](docs/setup/windows.md) for the tool list and optional helper script.

Then install dependencies and start the desktop app:

```powershell
npm install
npm run tauri dev
```

For browser-only UI development:

```powershell
npm run dev
```

## Security Notes

MailDock is read-only in the MVP. The IMAP implementation uses `EXAMINE INBOX` and `BODY.PEEK` so syncing does not mark remote messages as read.

Email authorization codes are not stored as plaintext in SQLite. QQ authorization codes are saved through the operating system credential store via the Rust `security` module.

Gmail OAuth tokens should use the same credential boundary later.

## Learning Log

- [Week 01: React UI and Tauri basics](docs/learning-log/week-01.md)
- [Week 02: SQLite local cache](docs/learning-log/week-02.md)
- [Week 03: QQ IMAP sync](docs/learning-log/week-03.md)

## Roadmap

1. QQ Mail mock UI
2. Tauri command integration
3. SQLite local cache
4. QQ Mail read-only IMAP sync
5. Fudan cloud mail provider
6. Gmail OAuth support
7. Multi-account unified inbox
8. Local full-text search
