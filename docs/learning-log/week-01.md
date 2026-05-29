# Week 01: React UI and Tauri Basics

## Goal

Build the first visible version of MailDock without connecting to a real mailbox yet.

## What This Week Adds

- A Tauri + React + TypeScript project skeleton
- A three-column mailbox interface
- Typed mock mail messages
- A first Tauri command boundary for loading messages
- Loading, empty, and error UI states

## Concepts To Learn

1. `src/` contains the React frontend.
2. `src-tauri/` contains the Rust desktop backend.
3. React components are small UI building blocks.
4. TypeScript interfaces describe the shape of data.
5. Tauri commands are the bridge between React and Rust.

## Useful Files

- `src/App.tsx`: app-level state and message loading
- `src/features/mail/types.ts`: TypeScript mail types
- `src/features/mail/components/`: mailbox UI components
- `src-tauri/src/commands/mail.rs`: Rust commands exposed to React

## Reflection Prompts

- What is the difference between a component and a page?
- Why does the app use mock data before real IMAP?
- What data does a mail list need compared with a mail detail view?
- What should stay in React, and what should move to Rust later?
- Why should authorization codes not be stored in plaintext SQLite?

## Next Week

Add SQLite so mock messages can be stored and loaded from a local database.
