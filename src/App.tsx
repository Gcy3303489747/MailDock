import { useEffect, useMemo, useRef, useState } from "react";
import { MailDetail } from "./features/mail/components/MailDetail";
import { MailList } from "./features/mail/components/MailList";
import { Sidebar } from "./features/mail/components/Sidebar";
import { Toolbar } from "./features/mail/components/Toolbar";
import { loadAccounts, loadInboxMessages, syncSavedQqInbox } from "./features/mail/mailApi";
import type { MailAccount, MailFolder, MailMessage } from "./features/mail/types";

interface SyncOptions {
  quietCredentialError?: boolean;
}

const AUTO_SYNC_INTERVAL_MS = 5 * 60 * 1000;

export default function App() {
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(null);
  const [selectedFolder] = useState<MailFolder>("INBOX");
  const [messages, setMessages] = useState<MailMessage[]>([]);
  const [selectedMessageId, setSelectedMessageId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSyncing, setIsSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncError, setSyncError] = useState<string | null>(null);
  const [lastSyncedAt, setLastSyncedAt] = useState<Date | null>(null);
  const isSyncingRef = useRef(false);
  const selectedAccountIdRef = useRef<number | null>(null);

  const selectedMessage = useMemo(
    () => messages.find((message) => message.id === selectedMessageId) ?? null,
    [messages, selectedMessageId],
  );

  const selectedAccount = useMemo(
    () => accounts.find((account) => account.id === selectedAccountId) ?? null,
    [accounts, selectedAccountId],
  );

  async function loadCachedMailbox(accountIdOverride?: number): Promise<number | null> {
    setIsLoading(true);
    setError(null);

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

      selectedAccountIdRef.current = nextAccountId;
      setSelectedAccountId(nextAccountId);

      if (nextAccountId === null) {
        setMessages([]);
        setSelectedMessageId(null);
        return null;
      }

      const nextMessages = await loadInboxMessages(nextAccountId, selectedFolder);
      applyMessages(nextAccountId, nextMessages);

      return nextAccountId;
    } catch (unknownError) {
      setError(messageFromUnknown(unknownError, "Unable to load messages."));
      return null;
    } finally {
      setIsLoading(false);
    }
  }

  async function syncMailbox(accountId: number | null, options: SyncOptions = {}) {
    if (accountId === null || isSyncingRef.current) {
      return;
    }

    isSyncingRef.current = true;
    setIsSyncing(true);
    setSyncError(null);

    try {
      await syncSavedQqInbox({ accountId, limit: 50 });
      const nextMessages = await loadInboxMessages(accountId, selectedFolder);
      applyMessages(accountId, nextMessages);
      setLastSyncedAt(new Date());
    } catch (syncError) {
      const nextSyncError = messageFromUnknown(syncError, "Unable to sync inbox.");
      console.info("Saved credential sync skipped; showing cached inbox instead.", syncError);

      if (!options.quietCredentialError) {
        setSyncError(nextSyncError);
      }
    } finally {
      isSyncingRef.current = false;
      setIsSyncing(false);
    }
  }

  function applyMessages(accountId: number, nextMessages: MailMessage[]) {
    if (selectedAccountIdRef.current !== accountId) {
      return;
    }

    setMessages(nextMessages);
    setSelectedMessageId((currentId) => {
      if (nextMessages.length === 0) {
        return null;
      }

      const stillExists = nextMessages.some((message) => message.id === currentId);
      return stillExists ? currentId : nextMessages[0].id;
    });
  }

  useEffect(() => {
    void (async () => {
      const accountId = await loadCachedMailbox();
      void syncMailbox(accountId, { quietCredentialError: true });
    })();
  }, []);

  useEffect(() => {
    const intervalId = window.setInterval(() => {
      void syncMailbox(selectedAccountIdRef.current);
    }, AUTO_SYNC_INTERVAL_MS);

    return () => window.clearInterval(intervalId);
  }, []);

  return (
    <main className="app-shell">
      <Sidebar
        accounts={accounts}
        selectedAccountId={selectedAccountId}
        onSelectAccount={(accountId) => {
          selectedAccountIdRef.current = accountId;
          setSelectedAccountId(accountId);
          void (async () => {
            const nextAccountId = await loadCachedMailbox(accountId);
            void syncMailbox(nextAccountId);
          })();
        }}
        onSyncComplete={(accountId) => {
          selectedAccountIdRef.current = accountId;
          setSelectedAccountId(accountId);
          setLastSyncedAt(new Date());
          void loadCachedMailbox(accountId);
        }}
      />
      <section className="mail-workspace" aria-label="MailDock inbox">
        <Toolbar
          account={selectedAccount}
          folder={selectedFolder}
          lastSyncedAt={lastSyncedAt}
          isSyncing={isSyncing}
          messageCount={messages.length}
          onRefresh={() => void syncMailbox(selectedAccountIdRef.current)}
          syncError={syncError}
        />
        <div className="mail-columns">
          <MailList
            error={error}
            isLoading={isLoading}
            messages={messages}
            selectedMessageId={selectedMessageId}
            onSelectMessage={setSelectedMessageId}
            onRetry={() => void loadCachedMailbox()}
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
