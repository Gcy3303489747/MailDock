import { FormEvent, useState } from "react";
import { testQqImapConnection } from "../mailApi";
import type { ImapConnectionReport } from "../types";

export function QqConnectionPanel() {
  const [email, setEmail] = useState("");
  const [authorizationCode, setAuthorizationCode] = useState("");
  const [isTesting, setIsTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [report, setReport] = useState<ImapConnectionReport | null>(null);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsTesting(true);
    setError(null);
    setReport(null);

    try {
      const nextReport = await testQqImapConnection({ email, authorizationCode });
      setReport(nextReport);
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
      setIsTesting(false);
    }
  }

  return (
    <form className="connection-panel" onSubmit={handleSubmit}>
      <div>
        <strong>QQ IMAP test</strong>
        <span>Authorization code is used once and not saved.</span>
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

      <button className="secondary-button" disabled={isTesting} type="submit">
        {isTesting ? "Testing" : "Test connection"}
      </button>

      {report && (
        <p className="connection-result">
          Connected to {report.host}:{report.port}. INBOX has {report.exists} messages.
        </p>
      )}
      {error && <p className="connection-error">{error}</p>}
    </form>
  );
}
