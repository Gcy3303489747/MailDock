export type ProviderKind = "qq" | "fudan" | "gmail";

export type AuthType = "authorization_code" | "oauth";

export type MailFolder = "INBOX";

export interface MailAccount {
  id: number;
  provider: ProviderKind;
  address: string;
  displayName: string;
  authType: AuthType;
  imapHost: string;
  imapPort: number;
  isEnabled: boolean;
}

export interface MailMessage {
  id: string;
  from: string;
  subject: string;
  receivedAt: string;
  preview: string;
  body: string;
  hasAttachments: boolean;
  isUnread: boolean;
}

export interface MailboxSummary {
  provider: ProviderKind;
  address: string;
  folder: MailFolder;
  unreadCount: number;
}

export interface QqInboxSyncInput {
  email: string;
  authorizationCode: string;
  limit?: number;
}

export interface SavedQqInboxSyncInput {
  accountId: number;
}

export interface QqInboxSyncReport {
  accountId: number;
  address: string;
  folder: MailFolder;
  fetched: number;
  stored: number;
  totalInboxMessages: number;
  credentialSaved: boolean;
}

export interface SyncState {
  accountId: number;
  folder: MailFolder;
  lastAttemptAt: string | null;
  lastSuccessAt: string | null;
  lastError: string | null;
  isSyncing: boolean;
}

export interface SyncStartedEvent {
  accountId: number;
  folder: MailFolder;
}

export interface SyncFinishedEvent {
  accountId: number;
  folder: MailFolder;
  stored: number;
  lastSuccessAt: string;
}

export interface SyncFailedEvent {
  accountId: number;
  folder: MailFolder;
  message: string;
  lastAttemptAt: string;
}

export interface MessagesChangedEvent {
  accountId: number;
  folder: MailFolder;
}
