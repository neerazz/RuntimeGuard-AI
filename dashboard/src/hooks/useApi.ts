import { AuditLog, CertificateItem, OversightItem, Stats } from "../types";

const API_BASE = import.meta.env.VITE_API_URL || "http://localhost:8000";

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) throw new Error(`Request failed: ${res.status}`);
  return res.json();
}

async function post<T>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) throw new Error(`Request failed: ${res.status}`);
  return res.json();
}

export const useApi = () => {
  return {
    getAuditLogs: () => get<{ logs: AuditLog[]; merkle_root: string; count: number }>("/api/v1/audit/logs?limit=20"),
    getOversightQueue: () => get<{ items: OversightItem[]; count: number }>("/api/v1/oversight/queue"),
    getOversightStats: () => get<Stats>("/api/v1/oversight/stats"),
    getCertificates: () => get<{ certificates: CertificateItem[] }>("/api/v1/certificates"),
    issueCertificate: () => post<CertificateItem>("/api/v1/certificates/issue"),
    actOnDecision: (id: string, action: { action: string; reviewer_id: string; justification: string }) =>
      post(`/api/v1/oversight/${id}/action`, action),
  };
};
