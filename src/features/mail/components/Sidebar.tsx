import type { MailAccount } from "../types";
import { QqConnectionPanel } from "./QqConnectionPanel";

interface SidebarProps {
  accounts: MailAccount[];
  selectedAccountId: number | null;
  onSelectAccount: (accountId: number) => void;
  onSyncComplete: (accountId: number) => void;
}

export function Sidebar({
  accounts,
  selectedAccountId,
  onSelectAccount,
  onSyncComplete,
}: SidebarProps) {
  return (
    <aside className="sidebar" aria-label="Mail accounts">
      <div>
        <p className="brand-label">MailDock</p>
        <h1>Inbox</h1>
      </div>

      <nav className="account-list" aria-label="Configured mailboxes">
        {accounts.length === 0 ? (
          <p className="empty-account-copy">No accounts loaded</p>
        ) : (
          accounts.map((account) => (
            <button
              className={`account-item ${
                account.id === selectedAccountId ? "account-item-active" : ""
              }`}
              key={account.id}
              onClick={() => onSelectAccount(account.id)}
              type="button"
            >
              <span className="account-avatar">{providerInitial(account.provider)}</span>
              <span>
                <strong>{account.displayName}</strong>
                <small>{account.address}</small>
              </span>
            </button>
          ))
        )}
      </nav>

      <div className="learning-card">
        <strong>Week 01</strong>
        <span>Mock UI, TypeScript types, and Tauri command basics.</span>
      </div>

      <QqConnectionPanel onSyncComplete={onSyncComplete} />
    </aside>
  );
}

function providerInitial(provider: MailAccount["provider"]): string {
  return provider.slice(0, 1).toUpperCase();
}
