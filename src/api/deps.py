from functools import lru_cache

from src.core.attestation import AttestationService
from src.core.certificate import CertificateService
from src.core.logging_config import configure_logging
from src.core.policy_engine import PolicyEngine
from src.core.storage import Storage
from src.mock.ai_model import MockModel


configure_logging()


@lru_cache
def get_storage() -> Storage:
    return Storage()


@lru_cache
def get_policy_engine() -> PolicyEngine:
    return PolicyEngine()


@lru_cache
def get_mock_model() -> MockModel:
    return MockModel()


@lru_cache
def get_attestation_service() -> AttestationService:
    return AttestationService()


@lru_cache
def get_certificate_service() -> CertificateService:
    return CertificateService(get_storage(), get_attestation_service())
