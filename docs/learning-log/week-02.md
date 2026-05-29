# Week 02: SQLite Local Cache

## Goal

Move from mock-only messages to a local database cache.

## Planned Tasks

- [x] Choose a Rust SQLite library
- [x] Add migrations for `accounts`, `messages`, and `sync_state`
- [x] Initialize the database when MailDock reads messages
- [x] Insert sample messages into SQLite
- [x] Load messages from SQLite through a Tauri command
- [x] Add account-aware commands for future multi-account support

## Concepts To Learn

- Tables and columns
- Primary keys
- SQL queries
- Database migrations
- Why desktop apps often cache remote data locally

## Useful Files

- `src-tauri/migrations/001_initial.sql`: first database schema
- `src-tauri/src/db/connection.rs`: database initialization and compatibility migrations
- `src-tauri/src/db/account_repo.rs`: account queries
- `src-tauri/src/db/message_repo.rs`: message queries
- `src-tauri/src/db/seed.rs`: learning seed data
- `src-tauri/src/commands/mail.rs`: Tauri commands exposed to React
- `src/features/mail/mailApi.ts`: frontend command call

## What Changed

The UI still looks like a mock inbox, but the data path is now real:

```text
React -> Tauri command -> Rust -> SQLite -> Rust -> React
```

The current command flow is:

```text
React -> list_accounts -> choose account -> list_messages(accountId, "INBOX")
```
