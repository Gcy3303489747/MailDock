import { FormEvent, useState } from "react";
import { syncQqInbox, testQqImapConnection } from "../mailApi";
import type { ImapConnectionReport, QqInboxSyncReport } from "../types";

interface QqConnectionPanelProps {
  onClose: () => void;
  onSyncComplete: (accountId: number) => void;
}

export function QqConnectionPanel({ onClose, onSyncComplete }: QqConnectionPanelProps) {
  const [email, setEmail] = useState("");
  const [authorizationCode, setAuthorizationCode] = useState("");
  const [isBusy, setIsBusy] = useState(false);
  const [action, setAction] = useState<"test" | "sync">("test");
  const [error, setError] = useState<string | null>(null);
  const [report, setReport] = useState<ImapConnectionReport | null>(null);
  const [syncReport, setSyncReport] = useState<QqInboxSyncReport | null>(null);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await runQqAction("test");
  }

  async function runQqAction(nextAction: "test" | "sync") {
    setAction(nextAction);
    setIsBusy(true);
    setError(null);
    setReport(null);
    setSyncReport(null);

    try {
      if (nextAction === "sync") {
        const nextSyncReport = await syncQqInbox({ email, authorizationCode, limit: 50 });
        setSyncReport(nextSyncReport);
        onSyncComplete(nextSyncReport.accountId);
      } else {
        const nextReport = await testQqImapConnection({ email, authorizationCode });
        setReport(nextReport);
      }

      setAuthorizationCode("");
    } catch (unknownError) {
      const message =
        unknownError instanceof Error
          ? unknownError.message
          : typeof unknownError === "string"
            ? unknownError
            : "QQ IMAP connection test failed.";
      setError(message);
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <form className="connection-panel" onSubmit={handleSubmit}>
      <div>
        <div className="connection-panel-header">
          <strong>QQ IMAP</strong>
          <button aria-label="Close import panel" onClick={onClose} type="button">
            Close
          </button>
        </div>
        <span>Authorization code is used for this action only.</span>
      </div>

      <label>
        Email
        <input
          autoComplete="email"
          onChange={(event) => setEmail(event.target.value)}
          placeholder="name@qq.com"
          type="email"
          value={email}
        />
      </label>

      <label>
        Authorization code
        <input
          autoComplete="off"
          onChange={(event) => setAuthorizationCode(event.target.value)}
          placeholder="QQ Mail code"
          type="password"
          value={authorizationCode}
        />
      </label>

      <div className="connection-actions">
        <button
          className="secondary-button"
          disabled={isBusy}
          onClick={() => void runQqAction("test")}
          type="button"
        >
          {isBusy && action === "test" ? "Testing" : "Test"}
        </button>
        <button
          className="primary-button"
          disabled={isBusy}
          onClick={() => void runQqAction("sync")}
          type="button"
        >
          {isBusy && action === "sync" ? "Syncing" : "Sync inbox"}
        </button>
      </div>

      {report && (
        <p className="connection-result">
          Connected to {report.host}:{report.port}. INBOX has {report.exists} messages.
        </p>
      )}
      {syncReport && (
        <p className="connection-result">
          Synced {syncReport.stored} messages from {syncReport.address}.
        </p>
      )}
      {error && <p className="connection-error">{error}</p>}
    </form>
  );
}
