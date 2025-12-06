import { Stats } from "../types";

interface Props {
  stats: Stats | null;
  merkleRoot: string;
}

export function StatusBar({ stats, merkleRoot }: Props) {
  return (
    <div className="grid three">
      <div className="card">
        <h3>Pending Oversight</h3>
        <div className="metric">{stats?.pending ?? 0}</div>
        <div className="muted">Escalated for human review</div>
      </div>
      <div className="card">
        <h3>Review Rate</h3>
        <div className="metric">{((stats?.review_rate ?? 0) * 100).toFixed(1)}%</div>
        <div className="muted">Approved/Rejected over total</div>
      </div>
      <div className="card">
        <h3>Merkle Root</h3>
        <div className="stack">
          <span className="muted">Current audit root</span>
          <code style={{ wordBreak: "break-all", color: "#80a6ff" }}>{merkleRoot || "N/A"}</code>
        </div>
      </div>
    </div>
  );
}
