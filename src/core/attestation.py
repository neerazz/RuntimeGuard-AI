import hashlib
import json
import logging
from dataclasses import dataclass
from datetime import datetime
from typing import Dict, Optional

from .db import DEFAULT_DB_PATH, get_connection, init_db
from .merkle_log import MerkleAuditLog

LOGGER = logging.getLogger(__name__)


@dataclass
class Proof:
    id: str
    batch_start_index: int
    batch_end_index: int
    merkle_root: str
    proof_data: Dict[str, str]
    public_inputs: Dict[str, str]
    verified: bool
    proving_time_ms: int


class AttestationService:
    """Generates deterministic proof artifacts without external ZK tools."""

    def __init__(self, db_path: str = str(DEFAULT_DB_PATH)) -> None:
        self.db_path = db_path
        init_db(db_path)
        self.merkle = MerkleAuditLog(db_path)

    def generate_proof(self) -> Proof:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute("SELECT COUNT(*) as c FROM merkle_nodes WHERE level=0")
            total = int(cur.fetchone()["c"])
        finally:
            conn.close()

        if total == 0:
            raise ValueError("No Merkle leaves to attest")

        start_ts = datetime.utcnow()
        root = self.merkle.get_root()
        public_inputs = {"merkle_root": root, "total_leaves": str(total)}
        digest_source = json.dumps(public_inputs, sort_keys=True)
        digest = hashlib.sha256(digest_source.encode()).hexdigest()
        proof_id = digest[:32]
        proof_data = {"digest": digest, "algorithm": "sha256"}
        proving_time_ms = int((datetime.utcnow() - start_ts).total_seconds() * 1000)

        conn = get_connection(self.db_path)
        try:
            conn.execute(
                """
                INSERT INTO attestation_proofs
                (id, batch_start_index, batch_end_index, merkle_root, proof_data, public_inputs, verified, proving_time_ms)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    proof_id,
                    0,
                    total - 1,
                    root,
                    json.dumps(proof_data),
                    json.dumps(public_inputs),
                    True,
                    proving_time_ms,
                ),
            )
            conn.commit()
            LOGGER.info(
                "attestation_generated id=%s root=%s leaves=%s", proof_id, root, total
            )
        finally:
            conn.close()

        return Proof(
            id=proof_id,
            batch_start_index=0,
            batch_end_index=total - 1,
            merkle_root=root,
            proof_data=proof_data,
            public_inputs=public_inputs,
            verified=True,
            proving_time_ms=proving_time_ms,
        )

    def verify_proof(self, proof_id: str) -> bool:
        conn = get_connection(self.db_path)
        try:
            cur = conn.execute(
                "SELECT merkle_root, public_inputs FROM attestation_proofs WHERE id=?", (proof_id,)
            )
            row = cur.fetchone()
            if not row:
                return False
            stored_root = row["merkle_root"]
            stored_public_inputs = json.loads(row["public_inputs"])
            current_root = self.merkle.get_root()
            ok = stored_root == current_root == stored_public_inputs.get("merkle_root")
            conn.execute(
                "UPDATE attestation_proofs SET verified=? WHERE id=?", (ok, proof_id)
            )
            conn.commit()
            LOGGER.info("attestation_verified id=%s ok=%s", proof_id, ok)
            return ok
        finally:
            conn.close()
