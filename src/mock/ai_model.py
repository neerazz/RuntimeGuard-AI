import hashlib
import logging
import time
from typing import Dict

from src.core.models import InferenceRequest, ModelResponse

LOGGER = logging.getLogger(__name__)


class MockModel:
    """Deterministic mock model for demo and tests."""

    def __init__(self, base_latency_ms: float = 25.0) -> None:
        self.base_latency_ms = base_latency_ms

    def predict(self, request: InferenceRequest) -> ModelResponse:
        start = time.perf_counter()
        score = self._score(request.input_data)
        latency_ms = (time.perf_counter() - start) * 1000 + self.base_latency_ms
        output = {"risk_score": score, "context": request.context}
        confidence = max(0.4, min(0.99, score))
        LOGGER.info(
            "mock_model_prediction context=%s model_id=%s confidence=%.3f latency_ms=%.2f",
            request.context,
            request.model_id,
            confidence,
            latency_ms,
        )
        return ModelResponse(output=output, confidence=confidence, latency_ms=latency_ms)

    def _score(self, payload: Dict) -> float:
        serialized = str(sorted(payload.items())).encode()
        digest = hashlib.sha256(serialized).hexdigest()
        # Map digest to [0, 1)
        raw_val = int(digest[:8], 16) / 0xFFFFFFFF
        return round(raw_val, 3)
