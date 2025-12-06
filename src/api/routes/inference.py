import logging
from fastapi import APIRouter, Depends, Request

from src.api.deps import get_mock_model, get_policy_engine, get_storage
from src.core.models import InferenceRequest, generate_id

router = APIRouter(prefix="/api/v1", tags=["inference"])
LOGGER = logging.getLogger(__name__)


@router.post("/inference")
async def proxy_inference(
    req: InferenceRequest,
    request: Request,
    storage=Depends(get_storage),
    policy_engine=Depends(get_policy_engine),
    model=Depends(get_mock_model),
):
    # Capture client IP if not provided
    resolved_req = req.copy()
    if not resolved_req.source_ip and request.client:
        resolved_req.source_ip = request.client.host

    request_id = generate_id()
    LOGGER.info("inference_received id=%s context=%s", request_id, resolved_req.context)

    model_response = model.predict(resolved_req)
    evaluation = policy_engine.evaluate(resolved_req, model_response)
    decision, merkle_root = storage.log_request_and_decision(request_id, resolved_req, evaluation)

    return {
        "request_id": request_id,
        "decision": decision.decision.value,
        "rules_triggered": decision.rules_triggered,
        "confidence_score": decision.confidence_score,
        "explanation": decision.explanation,
        "merkle_index": decision.merkle_index,
        "merkle_root": merkle_root,
        "model_output": model_response.output,
        "model_confidence": model_response.confidence,
        "latency_ms": model_response.latency_ms + evaluation.evaluation_time_ms,
    }
