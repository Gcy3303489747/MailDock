import { invoke } from "@tauri-apps/api/core";
import { mockAccounts, mockMessages } from "./mockData";
import type {
  ImapConnectionReport,
  MailAccount,
  MailFolder,
  MailMessage,
  QqInboxSyncInput,
  QqInboxSyncReport,
  QqImapConnectionInput,
  SavedQqInboxSyncInput,
} from "./types";

export async function loadAccounts(): Promise<MailAccount[]> {
  try {
    return await invoke<MailAccount[]>("list_accounts");
  } catch (error) {
    console.info("Using browser mock accounts until the Tauri SQLite backend is available.", error);
    return mockAccounts;
  }
}

export async function loadInboxMessages(
  accountId: number,
  folder: MailFolder = "INBOX",
): Promise<MailMessage[]> {
  try {
    return await invoke<MailMessage[]>("list_messages", { accountId, folder });
  } catch (error) {
    console.info("Using browser mock messages until the Tauri SQLite backend is available.", error);
    return mockMessages;
  }
}

export async function testQqImapConnection(
  input: QqImapConnectionInput,
): Promise<ImapConnectionReport> {
  return await invoke<ImapConnectionReport>("test_qq_imap_connection", { input });
}

export async function syncQqInbox(input: QqInboxSyncInput): Promise<QqInboxSyncReport> {
  return await invoke<QqInboxSyncReport>("sync_qq_inbox", { input });
}

export async function syncSavedQqInbox(
  input: SavedQqInboxSyncInput,
): Promise<QqInboxSyncReport> {
  try {
    return await invoke<QqInboxSyncReport>("sync_saved_qq_inbox", { input });
  } catch (error) {
    if (hasTauriBackend()) {
      throw error;
    }

    console.info("Skipping saved credential sync until the Tauri backend is available.", error);
    return {
      accountId: input.accountId,
      address: mockAccounts.find((account) => account.id === input.accountId)?.address ?? "",
      folder: "INBOX",
      fetched: mockMessages.length,
      stored: mockMessages.length,
      totalInboxMessages: mockMessages.length,
    };
  }
}

function hasTauriBackend(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
