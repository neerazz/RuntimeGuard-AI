import { CertificateItem } from "../types";

interface Props {
  certificates: CertificateItem[];
  onIssue: () => void;
}

export function CertificatePanel({ certificates, onIssue }: Props) {
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h3>Compliance Certificates</h3>
          <p className="muted">Attested summaries bound to Merkle roots</p>
        </div>
        <button className="btn" onClick={onIssue}>
          Issue New
        </button>
      </div>
      <div className="scroll">
        {certificates.length === 0 && <p className="muted">No certificates yet</p>}
        {certificates.map((c) => (
          <div key={c.id} className="list-item">
            <div>
              <div style={{ fontWeight: 700 }}>#{c.id.slice(0, 8)}</div>
              <div className="muted">
                {new Date(c.period_start).toLocaleString()} →{" "}
                {new Date(c.period_end).toLocaleString()}
              </div>
              <div className="muted" style={{ fontSize: 12 }}>
                proof {c.proof_id.slice(0, 8)}… | hash {c.certificate_hash.slice(0, 12)}…
              </div>
            </div>
            <div className="stack" style={{ alignItems: "flex-end" }}>
              <div className="metric" style={{ fontSize: 18, color: "#80a6ff" }}>
                {c.total_requests} reqs
              </div>
              <div className="muted">
                allow {c.allowed_count} · block {c.blocked_count} · escalate {c.escalated_count}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
