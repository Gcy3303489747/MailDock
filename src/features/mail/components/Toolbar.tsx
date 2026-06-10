import type { MailAccount, MailFolder } from "../types";

interface ToolbarProps {
  account: MailAccount | null;
  folder: MailFolder;
  lastSyncedAt: Date | null;
  isSyncing: boolean;
  messageCount: number;
  onRefresh: () => void;
  syncError: string | null;
}

export function Toolbar({
  account,
  folder,
  lastSyncedAt,
  isSyncing,
  messageCount,
  onRefresh,
  syncError,
}: ToolbarProps) {
  return (
    <header className="toolbar">
      <div>
        <p className="eyebrow">
          {account ? `${providerLabel(account.provider)} / ${folder}` : `No mailbox / ${folder}`}
        </p>
        <h2>Inbox</h2>
        {account && <p className="toolbar-account">{account.address}</p>}
        {account && (
          <p className="toolbar-sync-status">
            {isSyncing
              ? "Auto syncing..."
              : lastSyncedAt
                ? `Auto sync on. Last synced ${formatTime(lastSyncedAt)}.`
                : "Auto sync on. Waiting for first background sync."}
          </p>
        )}
        {syncError && <p className="toolbar-sync-error">{syncError}</p>}
      </div>
      <div className="toolbar-actions">
        <span className="message-count">{messageCount} messages</span>
        <button
          className="primary-button"
          disabled={isSyncing || !account}
          onClick={onRefresh}
          type="button"
        >
          {isSyncing ? "Syncing" : "Refresh"}
        </button>
      </div>
    </header>
  );
}

function formatTime(value: Date): string {
  return new Intl.DateTimeFormat("en", {
    hour: "2-digit",
    minute: "2-digit",
  }).format(value);
}

function providerLabel(provider: MailAccount["provider"]): string {
  switch (provider) {
    case "qq":
      return "QQ Mail";
    case "fudan":
      return "Fudan Mail";
    case "gmail":
      return "Gmail";
  }
}
