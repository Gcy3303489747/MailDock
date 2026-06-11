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
        <p className="state-title">Loading local inbox</p>
        <p className="state-copy">Reading cached messages from this device.</p>
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
        <p className="state-copy">MailDock will show synced inbox messages here.</p>
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
              <strong title={message.from}>{message.from}</strong>
              <time dateTime={message.receivedAt}>{formatDate(message.receivedAt)}</time>
            </span>
            <span className="message-subject" title={message.subject}>
              {message.isUnread && <span className="unread-dot" aria-label="Unread" />}
              <span>{message.subject}</span>
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
