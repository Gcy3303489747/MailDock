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
      <div className="toolbar-main">
        <p className="eyebrow">
          {account ? `${providerLabel(account.provider)} / ${folder}` : `No mailbox / ${folder}`}
        </p>
        <h2>Inbox</h2>
        {account && <p className="toolbar-account">{account.address}</p>}
        {account && (
          <p className={`toolbar-sync-status ${syncError ? "toolbar-sync-status-error" : ""}`}>
            {syncError
              ? `Sync failed: ${syncError}`
              : syncStatusText(isSyncing, lastSyncedAt)}
          </p>
        )}
      </div>
      <div className="toolbar-actions">
        <span className="message-count">{messageCountLabel(messageCount)}</span>
        <button
          className="secondary-button toolbar-refresh"
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

function syncStatusText(isSyncing: boolean, lastSyncedAt: Date | null): string {
  if (isSyncing) {
    return "Syncing in background";
  }

  if (lastSyncedAt) {
    return `Last synced ${formatTime(lastSyncedAt)}`;
  }

  return "Waiting for first sync";
}

function messageCountLabel(count: number): string {
  return `${count} ${count === 1 ? "message" : "messages"}`;
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
