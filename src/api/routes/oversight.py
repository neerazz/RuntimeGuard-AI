import logging
from fastapi import APIRouter, Depends, HTTPException

from src.api.deps import get_storage
from src.core.models import OversightAction

router = APIRouter(prefix="/api/v1/oversight", tags=["oversight"])
LOGGER = logging.getLogger(__name__)

ALLOWED_ACTIONS = {"APPROVE", "REJECT", "MODIFY", "DEFER"}


@router.get("/queue")
def escalation_queue(storage=Depends(get_storage)):
    queue = storage.list_escalations(limit=100)
    return {"items": queue, "count": len(queue)}


@router.post("/{decision_id}/action")
def take_action(decision_id: str, action: OversightAction, storage=Depends(get_storage)):
    if action.action not in ALLOWED_ACTIONS:
        raise HTTPException(status_code=400, detail="Invalid action")
    storage.record_oversight_action(
        decision_id=decision_id,
        reviewer_id=action.reviewer_id,
        action=action.action,
        justification=action.justification,
    )
    return {"status": "ok", "decision_id": decision_id}


@router.get("/stats")
def oversight_stats(storage=Depends(get_storage)):
    stats = storage.oversight_stats()
    counts = storage.decision_counts()
    review_rate = stats.get("APPROVE", 0) + stats.get("REJECT", 0)
    total = counts.get("total", 0) or 1
    return {
        "pending": stats.get("PENDING", 0),
        "approved": stats.get("APPROVE", 0),
        "rejected": stats.get("REJECT", 0),
        "deferred": stats.get("DEFER", 0),
        "total_requests": counts.get("total", 0),
        "review_rate": round(review_rate / total, 3),
    }
