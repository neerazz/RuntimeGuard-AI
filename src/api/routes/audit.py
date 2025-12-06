import logging
from fastapi import APIRouter, Depends

from src.api.deps import get_storage

router = APIRouter(prefix="/api/v1/audit", tags=["audit"])
LOGGER = logging.getLogger(__name__)


@router.get("/logs")
def list_logs(limit: int = 50, storage=Depends(get_storage)):
    logs = storage.list_decisions(limit=limit)
    LOGGER.info("audit_logs_requested count=%s", len(logs))
    return {"logs": logs, "count": len(logs), "merkle_root": storage.get_merkle_root()}


@router.get("/merkle-root")
def merkle_root(storage=Depends(get_storage)):
    root = storage.get_merkle_root()
    return {"merkle_root": root}
