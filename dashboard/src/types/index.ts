export type Decision = "ALLOW" | "BLOCK" | "ESCALATE";

export interface AuditLog {
  id: string;
  timestamp: string;
  decision: Decision;
  rules_triggered: string[];
  confidence_score: number;
  explanation: string;
  merkle_index: number;
  merkle_hash: string;
  model_id: string;
  user_id?: string;
  source_ip?: string;
}

export interface OversightItem {
  decision_id: string;
  timestamp: string;
  explanation: string;
  model_id: string;
  user_id?: string;
}

export interface Stats {
  pending: number;
  approved: number;
  rejected: number;
  deferred: number;
  total_requests: number;
  review_rate: number;
}

export interface CertificateItem {
  id: string;
  proof_id: string;
  period_start: string;
  period_end: string;
  total_requests: number;
  allowed_count: number;
  blocked_count: number;
  escalated_count: number;
  human_review_rate: number;
  certificate_hash: string;
}
