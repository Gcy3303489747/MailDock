import { invoke } from "@tauri-apps/api/core";
import { mockAccounts, mockMessages } from "./mockData";
import type {
  ImapConnectionReport,
  MailAccount,
  MailFolder,
  MailMessage,
  QqImapConnectionInput,
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
