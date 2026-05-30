import { useState } from "react";
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
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isImportOpen, setIsImportOpen] = useState(false);

  return (
    <aside className="sidebar" aria-label="Mail accounts">
      <div className="sidebar-header">
        <div className="sidebar-menu">
          <button
            aria-label="Open mailbox menu"
            aria-expanded={isMenuOpen}
            aria-haspopup="menu"
            className={`menu-trigger ${isMenuOpen ? "menu-trigger-active" : ""}`}
            onClick={() => setIsMenuOpen((current) => !current)}
            type="button"
            title="Mailbox menu"
          >
            <span />
            <span />
            <span />
          </button>
          {isMenuOpen && (
            <div className="sidebar-menu-popover" role="menu">
              <button
                onClick={() => {
                  setIsImportOpen(true);
                  setIsMenuOpen(false);
                }}
                role="menuitem"
                type="button"
              >
                <span className="menu-option-mark">+</span>
                <span>
                  <strong>Import mailbox</strong>
                  <small>Connect QQ Mail inbox</small>
                </span>
              </button>
            </div>
          )}
        </div>
        <div>
          <p className="brand-label">MailDock</p>
          <h1>Inbox</h1>
        </div>
      </div>

      <nav className="account-list" aria-label="Configured mailboxes">
        {accounts.length === 0 ? (
          <p className="empty-account-copy">Use the menu to import a mailbox.</p>
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

      {isImportOpen && (
        <QqConnectionPanel
          onClose={() => setIsImportOpen(false)}
          onSyncComplete={(accountId) => {
            onSyncComplete(accountId);
            setIsImportOpen(false);
          }}
        />
      )}

      <div className="learning-card">
        <strong>Week 03</strong>
        <span>QQ IMAP sync now imports messages into the local SQLite cache.</span>
      </div>
    </aside>
  );
}

function providerInitial(provider: MailAccount["provider"]): string {
  return provider.slice(0, 1).toUpperCase();
}
