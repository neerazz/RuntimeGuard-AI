import { useEffect, useMemo, useState } from "react";
import { AuditLog, CertificateItem, OversightItem, Stats } from "./types";
import { useApi } from "./hooks/useApi";
import { StatusBar } from "./components/StatusBar";
import { AuditLogTable } from "./components/AuditLogTable";
import { EscalationQueue } from "./components/EscalationQueue";
import { CertificatePanel } from "./components/CertificatePanel";

function App() {
  const api = useApi();
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [queue, setQueue] = useState<OversightItem[]>([]);
  const [certificates, setCertificates] = useState<CertificateItem[]>([]);
  const [stats, setStats] = useState<Stats | null>(null);
  const [merkleRoot, setMerkleRoot] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const refreshAll = async () => {
    try {
      const [auditRes, queueRes, statRes, certRes] = await Promise.all([
        api.getAuditLogs(),
        api.getOversightQueue(),
        api.getOversightStats(),
        api.getCertificates(),
      ]);
      setLogs(auditRes.logs || []);
      setMerkleRoot(auditRes.merkle_root);
      setQueue(queueRes.items || []);
      setStats(statRes);
      setCertificates(certRes.certificates || []);
      setError(null);
    } catch (err) {
      setError((err as Error).message);
    }
  };

  useEffect(() => {
    refreshAll();
    const interval = setInterval(refreshAll, 8000);
    return () => clearInterval(interval);
  }, []);

  const handleAction = async (id: string, action: "APPROVE" | "REJECT") => {
    const justification = prompt("Add a brief justification:");
    if (!justification) return;
    await api.actOnDecision(id, {
      action,
      reviewer_id: "ops-user",
      justification,
    });
    refreshAll();
  };

  const handleIssue = async () => {
    await api.issueCertificate();
    refreshAll();
  };

  const headline = useMemo(() => {
    const total = stats?.total_requests ?? 0;
    const esc = stats?.pending ?? 0;
    return `${total} requests · ${esc} pending review`;
  }, [stats]);

  return (
    <div className="app-shell">
      <div className="header">
        <div>
          <h1 className="title">RuntimeGuard-AI Oversight</h1>
          <p className="subtitle">{headline}</p>
        </div>
        <div className="pill">Live Compliance Chain</div>
      </div>

      {error && (
        <div className="card" style={{ border: "1px solid #ff6b6b" }}>
          <strong>Error:</strong> {error}
        </div>
      )}

      <StatusBar stats={stats} merkleRoot={merkleRoot} />

      <div className="grid three" style={{ marginTop: 16 }}>
        <EscalationQueue items={queue} onAction={handleAction} />
        <CertificatePanel certificates={certificates} onIssue={handleIssue} />
      </div>

      <div style={{ marginTop: 16 }}>
        <AuditLogTable logs={logs} />
      </div>

      <div className="footer">RuntimeGuard-AI • Attested Article 14 Oversight</div>
    </div>
  );
}

export default App;
