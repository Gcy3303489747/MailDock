import type { MailMessage } from "../types";

interface MailListProps {
  error: string | null;
  isLoading: boolean;
  messages: MailMessage[];
  selectedMessageId: string | null;
  onRetry: () => void;
  onSelectMessage: (messageId: string) => void;
}

export function MailList({
  error,
  isLoading,
  messages,
  selectedMessageId,
  onRetry,
  onSelectMessage,
}: MailListProps) {
  if (isLoading) {
    return (
      <section className="mail-list state-panel" aria-label="Loading messages">
        <p className="state-title">Loading inbox</p>
        <p className="state-copy">Checking local mail and recent syncs.</p>
      </section>
    );
  }

  if (error) {
    return (
      <section className="mail-list state-panel" aria-label="Message loading error">
        <p className="state-title">Could not load messages</p>
        <p className="state-copy">{error}</p>
        <button className="secondary-button" onClick={onRetry} type="button">
          Try again
        </button>
      </section>
    );
  }

  if (messages.length === 0) {
    return (
      <section className="mail-list state-panel" aria-label="Empty inbox">
        <p className="state-title">No messages yet</p>
        <p className="state-copy">Import a QQ mailbox from the menu to sync recent mail.</p>
      </section>
    );
  }

  return (
    <section className="mail-list" aria-label="Message list">
      {messages.map((message) => {
        const isSelected = message.id === selectedMessageId;

        return (
          <button
            className={`message-row ${isSelected ? "message-row-selected" : ""}`}
            key={message.id}
            onClick={() => onSelectMessage(message.id)}
            type="button"
          >
            <span className="message-row-topline">
              <strong>{message.from}</strong>
              <time dateTime={message.receivedAt}>{formatDate(message.receivedAt)}</time>
            </span>
            <span className="message-subject">
              {message.isUnread && <span className="unread-dot" aria-label="Unread" />}
              {message.subject}
              {message.hasAttachments && <span className="attachment-pill">Attachment</span>}
            </span>
            <span className="message-preview">{message.preview}</span>
          </button>
        );
      })}
    </section>
  );
}

function formatDate(value: string): string {
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}
