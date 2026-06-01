import { useEffect, useMemo, useState } from "react";
import { MailDetail } from "./features/mail/components/MailDetail";
import { MailList } from "./features/mail/components/MailList";
import { Sidebar } from "./features/mail/components/Sidebar";
import { Toolbar } from "./features/mail/components/Toolbar";
import { loadAccounts, loadInboxMessages, syncSavedQqInbox } from "./features/mail/mailApi";
import type { MailAccount, MailFolder, MailMessage } from "./features/mail/types";

interface RefreshOptions {
  syncSaved?: boolean;
  quietCredentialError?: boolean;
}

export default function App() {
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(null);
  const [selectedFolder] = useState<MailFolder>("INBOX");
  const [messages, setMessages] = useState<MailMessage[]>([]);
  const [selectedMessageId, setSelectedMessageId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [syncError, setSyncError] = useState<string | null>(null);

  const selectedMessage = useMemo(
    () => messages.find((message) => message.id === selectedMessageId) ?? null,
    [messages, selectedMessageId],
  );

  const selectedAccount = useMemo(
    () => accounts.find((account) => account.id === selectedAccountId) ?? null,
    [accounts, selectedAccountId],
  );

  async function refreshMessages(accountIdOverride?: number, options: RefreshOptions = {}) {
    setIsLoading(true);
    setError(null);
    setSyncError(null);

    try {
      const nextAccounts = await loadAccounts();
      setAccounts(nextAccounts);

      const selectedAccountStillExists = nextAccounts.some(
        (account) => account.id === selectedAccountId,
      );
      const nextAccountId =
        accountIdOverride ??
        (selectedAccountStillExists ? selectedAccountId : null) ??
        nextAccounts.find((account) => account.isEnabled)?.id ??
        nextAccounts[0]?.id ??
        null;

      setSelectedAccountId(nextAccountId);

      if (nextAccountId === null) {
        setMessages([]);
        setSelectedMessageId(null);
        return;
      }

      let nextSyncError: string | null = null;

      if (options.syncSaved) {
        try {
          await syncSavedQqInbox({ accountId: nextAccountId, limit: 50 });
        } catch (syncError) {
          nextSyncError = messageFromUnknown(syncError, "Unable to sync inbox.");
          console.info("Saved credential sync skipped; showing cached inbox instead.", syncError);
        }
      }

      const nextMessages = await loadInboxMessages(nextAccountId, selectedFolder);
      setMessages(nextMessages);
      setSelectedMessageId((currentId) => {
        if (nextMessages.length === 0) {
          return null;
        }

        const stillExists = nextMessages.some((message) => message.id === currentId);
        return stillExists ? currentId : nextMessages[0].id;
      });

      if (nextSyncError && !options.quietCredentialError) {
        setSyncError(nextSyncError);
      }
    } catch (unknownError) {
      setError(messageFromUnknown(unknownError, "Unable to load messages."));
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void refreshMessages(undefined, { syncSaved: true, quietCredentialError: true });
  }, []);

  return (
    <main className="app-shell">
      <Sidebar
        accounts={accounts}
        selectedAccountId={selectedAccountId}
        onSelectAccount={(accountId) => {
          setSelectedAccountId(accountId);
          void refreshMessages(accountId);
        }}
        onSyncComplete={(accountId) => {
          setSelectedAccountId(accountId);
          void refreshMessages(accountId);
        }}
      />
      <section className="mail-workspace" aria-label="MailDock inbox">
        <Toolbar
          account={selectedAccount}
          folder={selectedFolder}
          isLoading={isLoading}
          messageCount={messages.length}
          onRefresh={() => void refreshMessages(undefined, { syncSaved: true })}
          syncError={syncError}
        />
        <div className="mail-columns">
          <MailList
            error={error}
            isLoading={isLoading}
            messages={messages}
            selectedMessageId={selectedMessageId}
            onSelectMessage={setSelectedMessageId}
            onRetry={() => void refreshMessages()}
          />
          <MailDetail message={selectedMessage} />
        </div>
      </section>
    </main>
  );
}

function messageFromUnknown(unknownError: unknown, fallback: string): string {
  if (unknownError instanceof Error && unknownError.message) {
    return unknownError.message;
  }

  if (typeof unknownError === "string" && unknownError.trim()) {
    return unknownError;
  }

  return fallback;
}
