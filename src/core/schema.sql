-- RuntimeGuard-AI SQLite schema

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS inference_requests (
    id TEXT PRIMARY KEY,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    request_hash TEXT NOT NULL,
    request_payload TEXT NOT NULL,
    source_ip TEXT,
    user_id TEXT,
    model_id TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS policy_decisions (
    id TEXT PRIMARY KEY,
    request_id TEXT REFERENCES inference_requests(id),
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    decision TEXT NOT NULL CHECK (decision IN ('ALLOW', 'BLOCK', 'ESCALATE')),
    rules_triggered TEXT NOT NULL,
    confidence_score REAL,
    explanation TEXT,
    merkle_index INTEGER,
    merkle_hash TEXT
);

CREATE TABLE IF NOT EXISTS merkle_nodes (
    index_val INTEGER PRIMARY KEY,
    level INTEGER NOT NULL,
    hash TEXT NOT NULL,
    left_child INTEGER,
    right_child INTEGER,
    data_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS oversight_actions (
    id TEXT PRIMARY KEY,
    decision_id TEXT REFERENCES policy_decisions(id),
    reviewer_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('APPROVE', 'REJECT', 'MODIFY', 'DEFER')),
    justification TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    review_duration_seconds INTEGER
);

CREATE TABLE IF NOT EXISTS attestation_proofs (
    id TEXT PRIMARY KEY,
    batch_start_index INTEGER NOT NULL,
    batch_end_index INTEGER NOT NULL,
    merkle_root TEXT NOT NULL,
    proof_data TEXT NOT NULL,
    public_inputs TEXT NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    proving_time_ms INTEGER
);

CREATE TABLE IF NOT EXISTS certificates (
    id TEXT PRIMARY KEY,
    proof_id TEXT REFERENCES attestation_proofs(id),
    period_start DATETIME NOT NULL,
    period_end DATETIME NOT NULL,
    total_requests INTEGER NOT NULL,
    allowed_count INTEGER NOT NULL,
    blocked_count INTEGER NOT NULL,
    escalated_count INTEGER NOT NULL,
    human_review_rate REAL NOT NULL,
    certificate_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_decisions_timestamp ON policy_decisions(timestamp);
CREATE INDEX IF NOT EXISTS idx_decisions_request ON policy_decisions(request_id);
CREATE INDEX IF NOT EXISTS idx_oversight_decision ON oversight_actions(decision_id);
CREATE INDEX IF NOT EXISTS idx_merkle_level ON merkle_nodes(level);
