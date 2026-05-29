import { useEffect, useMemo, useState } from "react";
import { MailDetail } from "./features/mail/components/MailDetail";
import { MailList } from "./features/mail/components/MailList";
import { Sidebar } from "./features/mail/components/Sidebar";
import { Toolbar } from "./features/mail/components/Toolbar";
import { loadAccounts, loadInboxMessages } from "./features/mail/mailApi";
import type { MailAccount, MailFolder, MailMessage } from "./features/mail/types";

export default function App() {
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(null);
  const [selectedFolder] = useState<MailFolder>("INBOX");
  const [messages, setMessages] = useState<MailMessage[]>([]);
  const [selectedMessageId, setSelectedMessageId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const selectedMessage = useMemo(
    () => messages.find((message) => message.id === selectedMessageId) ?? null,
    [messages, selectedMessageId],
  );

  const selectedAccount = useMemo(
    () => accounts.find((account) => account.id === selectedAccountId) ?? null,
    [accounts, selectedAccountId],
  );

  async function refreshMessages(accountIdOverride?: number) {
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

      setSelectedAccountId(nextAccountId);

      if (nextAccountId === null) {
        setMessages([]);
        setSelectedMessageId(null);
        return;
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
    } catch (unknownError) {
      const message =
        unknownError instanceof Error ? unknownError.message : "Unable to load messages.";
      setError(message);
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void refreshMessages();
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
      />
      <section className="mail-workspace" aria-label="MailDock inbox">
        <Toolbar
          account={selectedAccount}
          folder={selectedFolder}
          isLoading={isLoading}
          messageCount={messages.length}
          onRefresh={() => void refreshMessages()}
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
