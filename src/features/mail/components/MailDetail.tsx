import type { MailMessage } from "../types";

interface MailDetailProps {
  message: MailMessage | null;
}

type BodyBlock =
  | { type: "paragraph"; text: string }
  | { type: "quote"; text: string }
  | { type: "divider"; key: string };

export function MailDetail({ message }: MailDetailProps) {
  if (!message) {
    return (
      <article className="mail-detail empty-detail" aria-label="No message selected">
        <p className="state-title">Select a message</p>
        <p className="state-copy">Synced messages will appear here after you import a mailbox.</p>
      </article>
    );
  }

  const bodyBlocks = buildBodyBlocks(message.body);

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
        {bodyBlocks.map((block, index) => {
          if (block.type === "divider") {
            return <hr className="detail-divider" key={block.key} />;
          }

          if (block.type === "quote") {
            return (
              <blockquote className="detail-quote" key={`${block.type}-${index}`}>
                {block.text}
              </blockquote>
            );
          }

          return (
            <p className="detail-paragraph" key={`${block.type}-${index}`}>
              {block.text}
            </p>
          );
        })}
      </div>
    </article>
  );
}

function buildBodyBlocks(body: string): BodyBlock[] {
  const normalized = body.replace(/\r\n/g, "\n").replace(/\r/g, "\n").trim();
  if (!normalized) {
    return [{ type: "paragraph", text: "This message has no readable text body yet." }];
  }

  const blocks: BodyBlock[] = [];
  let pending: string[] = [];
  let pendingType: "paragraph" | "quote" = "paragraph";

  const flush = () => {
    if (pending.length === 0) {
      return;
    }

    blocks.push({
      type: pendingType,
      text: pending.join("\n").trim(),
    });
    pending = [];
  };

  for (const line of normalized.split("\n")) {
    const trimmed = line.trim();

    if (!trimmed) {
      flush();
      continue;
    }

    if (/^[-_=]{3,}$/.test(trimmed)) {
      flush();
      blocks.push({ type: "divider", key: `divider-${blocks.length}` });
      continue;
    }

    const isQuote = trimmed.startsWith(">") || trimmed.startsWith("|");
    const nextType = isQuote ? "quote" : "paragraph";
    const text = isQuote ? trimmed.replace(/^[>|]\s?/, "") : line.trimEnd();

    if (pending.length > 0 && pendingType !== nextType) {
      flush();
    }

    pendingType = nextType;
    pending.push(text);
  }

  flush();
  return blocks;
}

function formatFullDate(value: string): string {
  return new Intl.DateTimeFormat("en", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}
