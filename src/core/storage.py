import json
import logging
import sqlite3
from datetime import datetime
from typing import Any, Dict, List, Optional, Tuple

from .db import DEFAULT_DB_PATH, get_connection, init_db
from .merkle_log import MerkleAuditLog
from .models import Certificate, Decision, InferenceRequest, PolicyDecision
from .policy_engine import EvaluationResult

LOGGER = logging.getLogger(__name__)


class Storage:
    """Persistence layer for requests, decisions, oversight actions, and proofs."""

    def __init__(self, db_path: str = str(DEFAULT_DB_PATH)) -> None:
        self.db_path = db_path
        init_db(db_path)
        self.merkle = MerkleAuditLog(db_path)

    # --- Inference + decision logging ---
    def record_request(
        self, request_id: str, request: InferenceRequest, request_hash: str
    ) -> None:
        conn = get_connection(self.db_path)
        try:
            conn.execute(
                """
                INSERT INTO inference_requests (id, request_hash, request_payload, source_ip, user_id, model_id)
                VALUES (?, ?, ?, ?, ?, ?)
                """,
                (
                    request_id,
                    request_hash,
                    json.dumps(request.model_dump()),
                    request.source_ip,
                    request.user_id,
                    request.model_id,
                ),
            )
            conn.commit()
        finally:
            conn.close()

    def record_decision(
        self, decision: PolicyDecision, evaluation: EvaluationResult
    ) -> PolicyDecision:
        conn = get_connection(self.db_path)
        try:
            conn.execute(
                """
                INSERT INTO policy_decisions
                (id, request_id, decision, rules_triggered, confidence_score, explanation, merkle_index, merkle_hash)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    decision.id,
                    decision.request_id,
                    decision.decision.value,
                    json.dumps(decision.rules_triggered),
                    decision.confidence_score,
                    decision.explanation,
                    decision.merkle_index,
                    decision.merkle_hash,
                ),
            )
            conn.commit()
            LOGGER.info(
                "decision_recorded id=%s decision=%s merkle_index=%s",
                decision.id,
                decision.decision.value,
                decision.merkle_index,
            )
            return decision
        finally:
            conn.close()

    def log_request_and_decision(
        self, request_id: str, request: InferenceRequest, evaluation: EvaluationResult
    ) -> Tuple[PolicyDecision, str]:
        self.record_request(request_id, request, evaluation.request_hash)
        data_hash = self._decision_data_hash(request_id, evaluation)
        merkle_index, merkle_root = self.merkle.append(data_hash)
        decision = PolicyDecision(
            id=request_id,
            request_id=request_id,
            decision=evaluation.decision,
            rules_triggered=evaluation.rules_triggered,
            confidence_score=evaluation.confidence_score,
            explanation=evaluation.explanation,
            merkle_index=merkle_index,
            merkle_hash=merkle_root,
        )
        self.record_decision(decision, evaluation)
        return decision, merkle_root

    # --- Oversight ---
    def list_escalations(self, limit: int = 50) -> List[Dict[str, Any]]:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute(
                """
                SELECT pd.id as decision_id, pd.timestamp, pd.explanation, ir.model_id, ir.user_id
                FROM policy_decisions pd
                JOIN inference_requests ir ON ir.id = pd.request_id
                WHERE pd.decision = 'ESCALATE'
                ORDER BY pd.timestamp DESC
                LIMIT ?
                """,
                (limit,),
            )
            return [dict(row) for row in cur.fetchall()]
        finally:
            conn.close()

    def list_decisions(self, limit: int = 100) -> List[Dict[str, Any]]:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute(
                """
                SELECT pd.id, pd.timestamp, pd.decision, pd.rules_triggered, pd.confidence_score,
                       pd.explanation, pd.merkle_index, pd.merkle_hash,
                       ir.model_id, ir.user_id, ir.source_ip
                FROM policy_decisions pd
                JOIN inference_requests ir ON ir.id = pd.request_id
                ORDER BY pd.timestamp DESC
                LIMIT ?
                """,
                (limit,),
            )
            rows = []
            for row in cur.fetchall():
                record = dict(row)
                record["rules_triggered"] = json.loads(record["rules_triggered"])
                rows.append(record)
            return rows
        finally:
            conn.close()

    def record_oversight_action(
        self, decision_id: str, reviewer_id: str, action: str, justification: str
    ) -> None:
        conn = get_connection(self.db_path)
        try:
            conn.execute(
                """
                INSERT INTO oversight_actions (id, decision_id, reviewer_id, action, justification)
                VALUES (?, ?, ?, ?, ?)
                """,
                (decision_id, decision_id, reviewer_id, action, justification),
            )
            conn.commit()
            LOGGER.info(
                "oversight_action_recorded decision_id=%s reviewer_id=%s action=%s",
                decision_id,
                reviewer_id,
                action,
            )
        finally:
            conn.close()

    def oversight_stats(self) -> Dict[str, Any]:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute(
                """
                SELECT action, COUNT(*) as c FROM oversight_actions GROUP BY action
                """
            )
            stats = {row["action"]: int(row["c"]) for row in cur.fetchall()}
            pending_cur = conn.execute(
                "SELECT COUNT(*) as c FROM policy_decisions WHERE decision='ESCALATE'"
            )
            stats["PENDING"] = int(pending_cur.fetchone()["c"])
            return stats
        finally:
            conn.close()

    # --- Certificates & proofs ---
    def decision_counts(self) -> Dict[str, int]:
        conn = get_connection(self.db_path)
        try:
            counts: Dict[str, int] = {"ALLOW": 0, "BLOCK": 0, "ESCALATE": 0}
            cur = conn.execute(
                """
                SELECT decision, COUNT(*) as c FROM policy_decisions GROUP BY decision
                """
            )
            for row in cur.fetchall():
                counts[row["decision"]] = int(row["c"])
            counts["total"] = sum(counts.values())
            return counts
        finally:
            conn.close()

    def save_certificate(self, cert: Certificate) -> None:
        conn = get_connection(self.db_path)
        try:
            conn.execute(
                """
                INSERT INTO certificates
                (id, proof_id, period_start, period_end, total_requests, allowed_count, blocked_count,
                 escalated_count, human_review_rate, certificate_hash)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    cert.id,
                    cert.proof_id,
                    cert.period_start.isoformat(),
                    cert.period_end.isoformat(),
                    cert.total_requests,
                    cert.allowed_count,
                    cert.blocked_count,
                    cert.escalated_count,
                    cert.human_review_rate,
                    cert.certificate_hash,
                ),
            )
            conn.commit()
            LOGGER.info("certificate_recorded id=%s", cert.id)
        finally:
            conn.close()

    def list_certificates(self, limit: int = 20) -> List[Dict[str, Any]]:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute(
                "SELECT * FROM certificates ORDER BY created_at DESC LIMIT ?", (limit,)
            )
            return [dict(row) for row in cur.fetchall()]
        finally:
            conn.close()

    def list_attestations(self, limit: int = 10) -> List[Dict[str, Any]]:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute(
                "SELECT * FROM attestation_proofs ORDER BY created_at DESC LIMIT ?", (limit,)
            )
            return [dict(row) for row in cur.fetchall()]
        finally:
            conn.close()

    def get_merkle_root(self) -> str:
        return self.merkle.get_root()

    def _decision_data_hash(self, request_id: str, evaluation: EvaluationResult) -> str:
        payload = f"{request_id}:{evaluation.decision.value}:{evaluation.request_hash}:{','.join(evaluation.rules_triggered)}"
        return payload
