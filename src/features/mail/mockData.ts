import type { MailAccount, MailMessage } from "./types";

export const mockAccounts: MailAccount[] = [
  {
    id: 1,
    provider: "qq",
    address: "learning@qq.com",
    displayName: "QQ Mail",
    authType: "authorization_code",
    imapHost: "imap.qq.com",
    imapPort: 993,
    isEnabled: true,
  },
];

export const mockMessages: MailMessage[] = [
  {
    id: "qq-001",
    from: "QQ Mail Team <service@qq.com>",
    subject: "Welcome to MailDock",
    receivedAt: "2026-05-26T08:30:00.000Z",
    preview: "This mock message helps the first UI milestone work before IMAP is ready.",
    body:
      "MailDock starts with mock messages so the interface can be built and understood before real email protocols are introduced.",
    hasAttachments: false,
    isUnread: true,
  },
  {
    id: "qq-002",
    from: "Learning Log <notes@maildock.local>",
    subject: "Week 01: React components",
    receivedAt: "2026-05-25T13:05:00.000Z",
    preview: "Today the goal is to split the mailbox into small, readable components.",
    body:
      "React components let the app grow in small pieces: Sidebar, Toolbar, MailList, and MailDetail. Each component has one clear job.",
    hasAttachments: false,
    isUnread: true,
  },
  {
    id: "qq-003",
    from: "Tauri Backend <backend@maildock.local>",
    subject: "Mock data now crosses the command boundary",
    receivedAt: "2026-05-24T21:18:00.000Z",
    preview: "The frontend will call a Tauri command first, then fall back to local mock data.",
    body:
      "The command boundary is the bridge between React and Rust. Later, the same frontend code can receive real SQLite or IMAP data.",
    hasAttachments: false,
    isUnread: false,
  },
  {
    id: "qq-004",
    from: "SQLite Notes <db@maildock.local>",
    subject: "Next milestone: local cache",
    receivedAt: "2026-05-23T16:42:00.000Z",
    preview: "The database milestone will add accounts, messages, and sync state tables.",
    body:
      "SQLite is a good fit for a personal desktop app. It keeps mail metadata available locally without requiring a server.",
    hasAttachments: true,
    isUnread: false,
  },
  {
    id: "qq-005",
    from: "Security Reminder <security@maildock.local>",
    subject: "Do not store authorization codes in plaintext",
    receivedAt: "2026-05-22T10:10:00.000Z",
    preview: "QQ Mail authorization codes should be stored in the system credential vault later.",
    body:
      "The MVP should avoid plaintext secrets in SQLite. The first implementation can use mock data, then add a credential store when real login is implemented.",
    hasAttachments: false,
    isUnread: false,
  },
];
