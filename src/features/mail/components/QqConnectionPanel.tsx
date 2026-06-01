import { FormEvent, useState } from "react";
import { syncQqInbox } from "../mailApi";
import type { QqInboxSyncReport } from "../types";

interface QqConnectionPanelProps {
  onClose: () => void;
  onSyncComplete: (accountId: number) => void;
}

export function QqConnectionPanel({ onClose, onSyncComplete }: QqConnectionPanelProps) {
  const [email, setEmail] = useState("");
  const [authorizationCode, setAuthorizationCode] = useState("");
  const [isBusy, setIsBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncReport, setSyncReport] = useState<QqInboxSyncReport | null>(null);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsBusy(true);
    setError(null);
    setSyncReport(null);

    try {
      const nextSyncReport = await syncQqInbox({ email, authorizationCode, limit: 50 });
      setSyncReport(nextSyncReport);
      setAuthorizationCode("");
      onSyncComplete(nextSyncReport.accountId);
    } catch (unknownError) {
      const message =
        unknownError instanceof Error
          ? unknownError.message
          : typeof unknownError === "string"
            ? unknownError
            : "QQ mailbox import failed.";
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
        <span>Import saves the code in your system credential store for future sync.</span>
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
        <button className="primary-button" disabled={isBusy} type="submit">
          {isBusy ? "Importing" : "Import inbox"}
        </button>
      </div>

      {syncReport && (
        <p className="connection-result">
          Synced {syncReport.stored} messages from {syncReport.address}.
          {syncReport.credentialSaved ? " Future sync is enabled." : ""}
        </p>
      )}
      {error && <p className="connection-error">{error}</p>}
    </form>
  );
}
