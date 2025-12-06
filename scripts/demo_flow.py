"""Run an in-process end-to-end demo without HTTP."""
from __future__ import annotations

from src.core.attestation import AttestationService
from src.core.certificate import CertificateService
from src.core.models import generate_id
from src.core.policy_engine import PolicyEngine
from src.core.storage import Storage
from src.mock.ai_model import MockModel
from src.mock.workload import generate_requests


def main():
    storage = Storage()
    policy = PolicyEngine()
    model = MockModel()

    print("Generating demo requests...")
    for req in generate_requests(5):
        model_resp = model.predict(req)
        eval_res = policy.evaluate(req, model_resp)
        storage.log_request_and_decision(generate_id(), req, eval_res)

    attest = AttestationService()
    cert_service = CertificateService(storage, attest)
    cert = cert_service.issue_certificate()

    print("Demo complete.")
    print("Certificate:", cert.model_dump())
    print("Current Merkle root:", storage.get_merkle_root())


if __name__ == "__main__":
    main()
