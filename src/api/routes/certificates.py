import logging
from fastapi import APIRouter, Depends, HTTPException

from src.api.deps import (
    get_attestation_service,
    get_certificate_service,
    get_storage,
)

router = APIRouter(prefix="/api/v1", tags=["certificates"])
LOGGER = logging.getLogger(__name__)


@router.post("/certificates/issue")
def issue_certificate(cert_service=Depends(get_certificate_service)):
    cert = cert_service.issue_certificate()
    return cert.model_dump()


@router.get("/certificates")
def list_certificates(storage=Depends(get_storage)):
    return {"certificates": storage.list_certificates()}


@router.get("/attestations")
def list_attestations(storage=Depends(get_storage)):
    return {"attestations": storage.list_attestations()}


@router.post("/attestations/{proof_id}/verify")
def verify_attestation(proof_id: str, attest=Depends(get_attestation_service)):
    ok = attest.verify_proof(proof_id)
    if not ok:
        raise HTTPException(status_code=400, detail="Proof invalid")
    return {"proof_id": proof_id, "verified": ok}
