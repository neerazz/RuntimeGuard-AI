from __future__ import annotations

import logging
import time
from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Any, Callable, Dict, List, Optional

from .models import Decision, InferenceRequest, ModelResponse, PolicyDecision

LOGGER = logging.getLogger(__name__)


@dataclass
class EvaluationResult:
    decision: Decision
    rules_triggered: List[str]
    confidence_score: float
    explanation: str
    evaluation_time_ms: float
    request_hash: str


class RateLimiter:
    """Simple sliding-window rate limiter keyed by user or IP."""

    def __init__(self, window_seconds: int = 60, max_requests: int = 30) -> None:
        self.window = timedelta(seconds=window_seconds)
        self.max_requests = max_requests
        self.events: Dict[str, List[datetime]] = {}

    def hit(self, key: str) -> bool:
        now = datetime.utcnow()
        events = self.events.setdefault(key, [])
        window_start = now - self.window
        # drop expired
        self.events[key] = [t for t in events if t >= window_start]
        self.events[key].append(now)
        return len(self.events[key]) > self.max_requests


class PolicyRule:
    def __init__(
        self,
        rule_id: str,
        name: str,
        description: str,
        severity: str,
        evaluator: Callable[[InferenceRequest, Optional[ModelResponse]], tuple[bool, str]],
        block_on_violation: bool = True,
    ) -> None:
        self.id = rule_id
        self.name = name
        self.description = description
        self.severity = severity
        self.evaluator = evaluator
        self.block_on_violation = block_on_violation


class PolicyEngine:
    """
    Evaluates inference requests against default EU AI Act-inspired rules.
    """

    def __init__(self) -> None:
        self.rules: List[PolicyRule] = []
        self.rate_limiter = RateLimiter()
        self._load_default_rules()

    def _load_default_rules(self) -> None:
        self.rules = [
            PolicyRule(
                "EU-AI-ACT-ART14-PC",
                "Protected Category Detection",
                "Escalate if request contains protected category signals",
                "CRITICAL",
                self._check_protected_categories,
                block_on_violation=False,
            ),
            PolicyRule(
                "EU-AI-ACT-ART14-HS",
                "High-Stakes Decision",
                "Escalate for safety-critical or life-impacting contexts",
                "HIGH",
                self._check_high_stakes,
                block_on_violation=False,
            ),
            PolicyRule(
                "EU-AI-ACT-ART14-CONF",
                "Confidence Threshold",
                "Block low-confidence automated outputs",
                "HIGH",
                self._check_confidence_threshold,
                block_on_violation=True,
            ),
            PolicyRule(
                "EU-AI-ACT-ART14-RATE",
                "Rate Limiting",
                "Block abusive request rates per actor",
                "MEDIUM",
                self._check_rate_limit,
                block_on_violation=True,
            ),
            PolicyRule(
                "EU-AI-ACT-ART5-PROHIB",
                "Prohibited Content",
                "Block obvious prohibited or manipulative content",
                "CRITICAL",
                self._check_prohibited_content,
                block_on_violation=True,
            ),
        ]

    def evaluate(
        self, request: InferenceRequest, model_response: Optional[ModelResponse] = None
    ) -> EvaluationResult:
        start = time.perf_counter()
        triggered: List[str] = []
        block = False
        escalate = False
        rationales: List[str] = []

        for rule in self.rules:
            result, rationale = rule.evaluator(request, model_response)
            if result:
                triggered.append(rule.id)
                rationales.append(f"{rule.id}: {rationale}")
                if rule.block_on_violation:
                    block = True
                else:
                    escalate = True

        decision = Decision.ALLOW
        if block:
            decision = Decision.BLOCK
        elif escalate:
            decision = Decision.ESCALATE

        confidence_score = self._compute_confidence(decision, triggered, model_response)
        explanation = "; ".join(rationales) if rationales else "All policies satisfied"
        duration_ms = (time.perf_counter() - start) * 1000

        LOGGER.info(
            "policy_evaluation_complete decision=%s triggered=%s confidence=%.3f eval_ms=%.2f",
            decision.value,
            triggered,
            confidence_score,
            round(duration_ms, 2),
        )

        return EvaluationResult(
            decision=decision,
            rules_triggered=triggered,
            confidence_score=confidence_score,
            explanation=explanation,
            evaluation_time_ms=duration_ms,
            request_hash=request.hash_repr(),
        )

    # --- Rule evaluators ---
    def _check_protected_categories(
        self, request: InferenceRequest, _: Optional[ModelResponse]
    ) -> tuple[bool, str]:
        protected_keys = {"race", "ethnicity", "religion", "biometric", "gender", "political_view"}
        found = [k for k in request.input_data if k.lower() in protected_keys]
        return (len(found) > 0, f"protected attributes present: {found}") if found else (False, "")

    def _check_high_stakes(
        self, request: InferenceRequest, _: Optional[ModelResponse]
    ) -> tuple[bool, str]:
        high_stakes_contexts = {"credit_decision", "medical_triage", "hiring", "admissions"}
        if request.context in high_stakes_contexts:
            return True, f"high_stakes_context={request.context}"
        return False, ""

    def _check_confidence_threshold(
        self, request: InferenceRequest, model_response: Optional[ModelResponse]
    ) -> tuple[bool, str]:
        threshold = 0.6
        confidence = model_response.confidence if model_response else request.model_confidence
        if confidence is None:
            return True, "missing_confidence"
        return (confidence < threshold, f"confidence={confidence} below {threshold}") if confidence < threshold else (
            False,
            "",
        )

    def _check_rate_limit(
        self, request: InferenceRequest, _: Optional[ModelResponse]
    ) -> tuple[bool, str]:
        key = request.user_id or request.source_ip or "anonymous"
        limited = self.rate_limiter.hit(key)
        return (limited, f"rate_limit_exceeded for {key}") if limited else (False, "")

    def _check_prohibited_content(
        self, request: InferenceRequest, _: Optional[ModelResponse]
    ) -> tuple[bool, str]:
        prohibited_keywords = ["social scoring", "real-time biometric surveillance", "emotion recognition"]
        content = str(request.input_data).lower()
        for kw in prohibited_keywords:
            if kw in content:
                return True, f"prohibited_content={kw}"
        return False, ""

    @staticmethod
    def _compute_confidence(
        decision: Decision, triggered: List[str], model_response: Optional[ModelResponse]
    ) -> float:
        base = 0.9 if decision == Decision.ALLOW else 0.7
        penalty = 0.05 * len(triggered)
        confidence = max(0.1, base - penalty)
        if model_response:
            confidence = min(confidence, model_response.confidence)
        return round(confidence, 3)


def decision_from_evaluation(
    request_id: str, evaluation: EvaluationResult, merkle_index: Optional[int], merkle_hash: Optional[str]
) -> PolicyDecision:
    return PolicyDecision(
        id=request_id,
        request_id=request_id,
        decision=evaluation.decision,
        rules_triggered=evaluation.rules_triggered,
        confidence_score=evaluation.confidence_score,
        explanation=evaluation.explanation,
        merkle_index=merkle_index,
        merkle_hash=merkle_hash,
    )
