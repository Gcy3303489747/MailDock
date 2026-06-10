import type { MailAccount, MailFolder } from "../types";

interface ToolbarProps {
  account: MailAccount | null;
  folder: MailFolder;
  isSyncing: boolean;
  messageCount: number;
  onRefresh: () => void;
  syncError: string | null;
}

export function Toolbar({
  account,
  folder,
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
          {isSyncing ? "Syncing" : "Sync now"}
        </button>
      </div>
    </header>
  );
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
