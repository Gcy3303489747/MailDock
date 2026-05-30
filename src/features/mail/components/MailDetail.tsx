import type { MailMessage } from "../types";

interface MailDetailProps {
  message: MailMessage | null;
}

export function MailDetail({ message }: MailDetailProps) {
  if (!message) {
    return (
      <article className="mail-detail empty-detail" aria-label="No message selected">
        <p className="state-title">Select a message</p>
        <p className="state-copy">Synced messages will appear here after you import a mailbox.</p>
      </article>
    );
  }

  return (
    <article className="mail-detail" aria-label="Message detail">
      <header className="detail-header">
        <p className="eyebrow">Message preview</p>
        <h2>{message.subject}</h2>
        <div className="detail-meta">
          <span>{message.from}</span>
          <time dateTime={message.receivedAt}>{formatFullDate(message.receivedAt)}</time>
        </div>
      </header>
      <div className="detail-body">
        <p>{message.body}</p>
      </div>
    </article>
  );
}

function formatFullDate(value: string): string {
  return new Intl.DateTimeFormat("en", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}
