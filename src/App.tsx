import { useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { MailDetail } from "./features/mail/components/MailDetail";
import { MailList } from "./features/mail/components/MailList";
import { Sidebar } from "./features/mail/components/Sidebar";
import { Toolbar } from "./features/mail/components/Toolbar";
import { loadAccounts, loadInboxMessages, loadSyncState, syncAccountNow } from "./features/mail/mailApi";
import type {
  MailAccount,
  MailFolder,
  MailMessage,
  MessagesChangedEvent,
  SyncFailedEvent,
  SyncFinishedEvent,
  SyncStartedEvent,
} from "./features/mail/types";

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
      await applySyncState(nextAccountId);

      return nextAccountId;
    } catch (unknownError) {
      setError(messageFromUnknown(unknownError, "Unable to load messages."));
      return null;
    } finally {
      setIsLoading(false);
    }
  }

  async function refreshMailbox(accountId: number | null) {
    if (accountId === null) {
      return;
    }

    setIsSyncing(true);
    setSyncError(null);

    try {
      await syncAccountNow({ accountId });
    } catch (syncError) {
      const nextSyncError = messageFromUnknown(syncError, "Unable to sync inbox.");
      setSyncError(nextSyncError);
      console.info("Manual sync failed; showing cached inbox instead.", syncError);
    } finally {
      setIsSyncing(false);
    }
  }

  async function applySyncState(accountId: number) {
    const syncState = await loadSyncState(accountId, selectedFolder);
    if (selectedAccountIdRef.current !== accountId) {
      return;
    }

    setIsSyncing(syncState.isSyncing);
    setSyncError(syncState.lastError);
    setLastSyncedAt(syncState.lastSuccessAt ? new Date(syncState.lastSuccessAt) : null);
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
    void loadCachedMailbox();
  }, []);

  useEffect(() => {
    const unlistenCallbacks: Array<() => void> = [];
    let isMounted = true;

    void listen<SyncStartedEvent>("maildock:sync-started", (event) => {
      if (selectedAccountIdRef.current !== event.payload.accountId) {
        return;
      }

      setIsSyncing(true);
      setSyncError(null);
    }).then((unlisten) => {
      if (isMounted) {
        unlistenCallbacks.push(unlisten);
      } else {
        unlisten();
      }
    });

    void listen<SyncFinishedEvent>("maildock:sync-finished", (event) => {
      if (selectedAccountIdRef.current !== event.payload.accountId) {
        return;
      }

      setIsSyncing(false);
      setSyncError(null);
      setLastSyncedAt(new Date(event.payload.lastSuccessAt));
    }).then((unlisten) => {
      if (isMounted) {
        unlistenCallbacks.push(unlisten);
      } else {
        unlisten();
      }
    });

    void listen<SyncFailedEvent>("maildock:sync-failed", (event) => {
      if (selectedAccountIdRef.current !== event.payload.accountId) {
        return;
      }

      setIsSyncing(false);
      setSyncError(event.payload.message);
    }).then((unlisten) => {
      if (isMounted) {
        unlistenCallbacks.push(unlisten);
      } else {
        unlisten();
      }
    });

    void listen<MessagesChangedEvent>("maildock:messages-changed", (event) => {
      if (selectedAccountIdRef.current !== event.payload.accountId) {
        return;
      }

      void loadInboxMessages(event.payload.accountId, event.payload.folder).then((nextMessages) =>
        applyMessages(event.payload.accountId, nextMessages),
      );
    }).then((unlisten) => {
      if (isMounted) {
        unlistenCallbacks.push(unlisten);
      } else {
        unlisten();
      }
    });

    return () => {
      isMounted = false;
      unlistenCallbacks.forEach((unlisten) => unlisten());
    };
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
            void refreshMailbox(nextAccountId);
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
          onRefresh={() => void refreshMailbox(selectedAccountIdRef.current)}
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
