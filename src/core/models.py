from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional
from uuid import uuid4

from pydantic import BaseModel, Field


def generate_id() -> str:
    return uuid4().hex


class Decision(str, Enum):
    ALLOW = "ALLOW"
    BLOCK = "BLOCK"
    ESCALATE = "ESCALATE"


class InferenceRequest(BaseModel):
    model_id: str
    input_data: Dict[str, Any]
    context: str = "generic"
    user_id: Optional[str] = None
    source_ip: Optional[str] = None
    model_confidence: Optional[float] = Field(default=None, ge=0.0, le=1.0)

    def hash_repr(self) -> str:
        import hashlib
        import json

        payload = {
            "model_id": self.model_id,
            "input_data": self.input_data,
            "context": self.context,
            "user_id": self.user_id,
            "source_ip": self.source_ip,
            "model_confidence": self.model_confidence,
        }
        serialized = json.dumps(payload, sort_keys=True, separators=(",", ":"))
        return hashlib.sha256(serialized.encode()).hexdigest()


class ModelResponse(BaseModel):
    output: Dict[str, Any]
    confidence: float
    latency_ms: float


@dataclass
class PolicyRuleResult:
    id: str
    name: str
    triggered: bool
    severity: str
    rationale: str


@dataclass
class PolicyDecision:
    id: str
    request_id: str
    decision: Decision
    rules_triggered: List[str]
    confidence_score: float
    explanation: str
    merkle_index: Optional[int] = None
    merkle_hash: Optional[str] = None
    timestamp: datetime = datetime.utcnow()


class OversightAction(BaseModel):
    action: str
    reviewer_id: str
    justification: str
    review_duration_seconds: Optional[int] = None


class Certificate(BaseModel):
    id: str
    proof_id: str
    period_start: datetime
    period_end: datetime
    total_requests: int
    allowed_count: int
    blocked_count: int
    escalated_count: int
    human_review_rate: float
    certificate_hash: str
