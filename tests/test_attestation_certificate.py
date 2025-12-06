from pathlib import Path

from src.core.attestation import AttestationService
from src.core.certificate import CertificateService
from src.core.models import InferenceRequest, generate_id
from src.core.policy_engine import PolicyEngine
from src.core.storage import Storage
from src.mock.ai_model import MockModel


def test_certificate_flow(tmp_path: Path):
    db_path = tmp_path / "audit.db"
    storage = Storage(str(db_path))
    policy = PolicyEngine()
    model = MockModel()

    # Log a few decisions
    for _ in range(3):
        req = InferenceRequest(model_id="m1", input_data={"feature": "x"}, model_confidence=0.9)
        eval_res = policy.evaluate(req, model.predict(req))
        storage.log_request_and_decision(generate_id(), req, eval_res)

    attest = AttestationService(str(db_path))
    cert_service = CertificateService(storage, attest)
    cert = cert_service.issue_certificate()

    assert cert.total_requests == 3
    assert storage.list_certificates()
    assert attest.verify_proof(cert.proof_id)
