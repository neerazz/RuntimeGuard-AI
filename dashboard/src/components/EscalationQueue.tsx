import { OversightItem } from "../types";

interface Props {
  items: OversightItem[];
  onAction: (id: string, action: "APPROVE" | "REJECT") => void;
}

export function EscalationQueue({ items, onAction }: Props) {
  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h3>Escalation Queue</h3>
          <p className="muted">Human oversight required</p>
        </div>
      </div>
      <div className="scroll">
        {items.length === 0 && <p className="muted">No pending escalations</p>}
        {items.map((item) => (
          <div key={item.decision_id} className="list-item">
            <div>
              <div style={{ fontWeight: 700 }}>{item.model_id}</div>
              <div className="muted">{item.explanation}</div>
              <div className="muted" style={{ fontSize: 12 }}>
                {new Date(item.timestamp).toLocaleString()}
              </div>
            </div>
            <div style={{ display: "flex", gap: 8 }}>
              <button
                className="btn"
                style={{ background: "linear-gradient(135deg,#66e5b0,#4bd6a2)", color: "#0b1121" }}
                onClick={() => onAction(item.decision_id, "APPROVE")}
              >
                Approve
              </button>
              <button
                className="btn"
                style={{ background: "linear-gradient(135deg,#ff8a8a,#ff6b6b)", color: "#0b1121" }}
                onClick={() => onAction(item.decision_id, "REJECT")}
              >
                Reject
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
