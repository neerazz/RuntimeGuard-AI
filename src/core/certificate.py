import hashlib
import logging
from datetime import datetime
from uuid import uuid4

from .attestation import AttestationService
from .models import Certificate
from .storage import Storage

LOGGER = logging.getLogger(__name__)


class CertificateService:
    def __init__(self, storage: Storage, attestation: AttestationService) -> None:
        self.storage = storage
        self.attestation = attestation

    def issue_certificate(self) -> Certificate:
        counts = self.storage.decision_counts()
        proof = self.attestation.generate_proof()
        period_end = datetime.utcnow()
        period_start = period_end.replace(hour=0, minute=0, second=0, microsecond=0)

        human_review_rate = (
            counts["ESCALATE"] / counts["total"] if counts.get("total", 0) else 0.0
        )
        summary = f"{proof.id}:{proof.merkle_root}:{counts}"
        certificate_hash = hashlib.sha256(summary.encode()).hexdigest()
        cert = Certificate(
            id=uuid4().hex,
            proof_id=proof.id,
            period_start=period_start,
            period_end=period_end,
            total_requests=counts["total"],
            allowed_count=counts["ALLOW"],
            blocked_count=counts["BLOCK"],
            escalated_count=counts["ESCALATE"],
            human_review_rate=round(human_review_rate, 3),
            certificate_hash=certificate_hash,
        )
        self.storage.save_certificate(cert)
        LOGGER.info(
            "certificate_issued id=%s proof_id=%s root=%s",
            cert.id,
            proof.id,
            proof.merkle_root,
        )
        return cert
