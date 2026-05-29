# MailDock

MailDock is a learning-driven Windows desktop mail aggregator built with Tauri, React, TypeScript, Rust, and SQLite.

The first version is intentionally small: QQ Mail, read-only inbox, local cache, and manual refresh. The goal is to build something personally useful while learning desktop app development one milestone at a time.

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
- Manual refresh
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
- [x] QQ IMAP connection test
- [ ] QQ Mail IMAP sync
- [ ] Windows installer

## Architecture

```text
React UI -> Tauri commands -> Rust backend -> SQLite
                                  |
                                  -> QQ Mail IMAP
```

The MVP keeps React focused on interface state. Rust owns the local database, IMAP access, and future credential storage.

The current SQLite milestone creates:

- `accounts`
- `messages`
- `sync_state`

The app seeds one learning QQ account locally so the UI already reads from the same path that will later receive QQ Mail IMAP data. The command boundary is now account-aware:

```text
list_accounts()
list_messages(account_id, folder)
```

The `accounts` table stores provider metadata such as `qq`, `fudan`, or `gmail`, plus the future IMAP host and auth type. Secrets still do not belong in SQLite.

## Local Development

Install the required tools first:

- Git
- Node.js LTS, including npm
- Rust stable MSVC toolchain
- Microsoft Visual Studio C++ Build Tools
- WebView2 Runtime

Then run:

```powershell
npm install
npm run tauri dev
```

For browser-only UI development:

```powershell
npm run dev
```

## Security Notes

MailDock is read-only in the MVP. The future IMAP implementation should use `EXAMINE INBOX` and `BODY.PEEK` so syncing does not mark remote messages as read.

Email authorization codes should not be stored as plaintext in SQLite. A later milestone should store them in the system credential vault.

The backend has a placeholder credential service boundary so future QQ authorization codes and Gmail OAuth tokens can be stored outside the mail cache.

The current QQ IMAP test uses the authorization code for one connection attempt only. It does not save the code to SQLite.

## Learning Log

- [Week 01: React UI and Tauri basics](docs/learning-log/week-01.md)
- [Week 02: SQLite local cache](docs/learning-log/week-02.md)

## Roadmap

1. QQ Mail mock UI
2. Tauri command integration
3. SQLite local cache
4. QQ Mail read-only IMAP sync
5. Fudan cloud mail provider
6. Gmail OAuth support
7. Multi-account unified inbox
8. Local full-text search
