import type { MailAccount, MailFolder } from "../types";

interface ToolbarProps {
  account: MailAccount | null;
  folder: MailFolder;
  isLoading: boolean;
  messageCount: number;
  onRefresh: () => void;
}

export function Toolbar({ account, folder, isLoading, messageCount, onRefresh }: ToolbarProps) {
  return (
    <header className="toolbar">
      <div>
        <p className="eyebrow">
          {account ? `${providerLabel(account.provider)} / ${folder}` : `No account / ${folder}`}
        </p>
        <h2>Read-only inbox</h2>
        {account && <p className="toolbar-account">{account.address}</p>}
      </div>
      <div className="toolbar-actions">
        <span className="message-count">{messageCount} messages</span>
        <button className="primary-button" disabled={isLoading} onClick={onRefresh} type="button">
          {isLoading ? "Refreshing" : "Refresh"}
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
