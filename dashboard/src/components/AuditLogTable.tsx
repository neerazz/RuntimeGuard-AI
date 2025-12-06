import { AuditLog } from "../types";

interface Props {
  logs: AuditLog[];
}

const decisionClass = (decision: string) => {
  if (decision === "ALLOW") return "chip allow";
  if (decision === "BLOCK") return "chip block";
  return "chip escalate";
};

export function AuditLogTable({ logs }: Props) {
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h3>Audit Log</h3>
          <p className="muted">Latest policy decisions with Merkle indices</p>
        </div>
      </div>
      <div className="scroll">
        <table className="table">
          <thead>
            <tr>
              <th>Decision</th>
              <th>Model</th>
              <th>User</th>
              <th>Rules</th>
              <th>Merkle</th>
              <th>Time</th>
            </tr>
          </thead>
          <tbody>
            {logs.map((log) => (
              <tr key={log.id}>
                <td>
                  <span className={decisionClass(log.decision)}>{log.decision}</span>
                </td>
                <td>{log.model_id}</td>
                <td>{log.user_id || "n/a"}</td>
                <td>
                  <div className="stack">
                    {log.rules_triggered.length === 0 ? (
                      <span className="muted">none</span>
                    ) : (
                      log.rules_triggered.map((r) => (
                        <span key={r} className="muted">
                          {r}
                        </span>
                      ))
                    )}
                  </div>
                </td>
                <td>
                  <div className="stack">
                    <strong>#{log.merkle_index}</strong>
                    <span className="muted">{log.merkle_hash.slice(0, 12)}...</span>
                  </div>
                </td>
                <td className="muted">{new Date(log.timestamp).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
