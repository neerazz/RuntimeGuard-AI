import logging
import time
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware

from src.api.routes import audit, certificates, inference, oversight
from src.core.db import init_db
from src.core.logging_config import configure_logging

configure_logging()
LOGGER = logging.getLogger("runtimeguard")

app = FastAPI(title="RuntimeGuard-AI", version="0.1.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.middleware("http")
async def timing_middleware(request: Request, call_next):
    start = time.perf_counter()
    response = await call_next(request)
    duration_ms = (time.perf_counter() - start) * 1000
    LOGGER.info(
        "request_complete path=%s method=%s status=%s duration_ms=%.2f",
        request.url.path,
        request.method,
        getattr(response, "status_code", None),
        round(duration_ms, 2),
    )
    return response


@app.on_event("startup")
async def _startup() -> None:
    init_db()
    LOGGER.info("runtimeguard_startup")


@app.get("/healthz")
async def health() -> dict:
    return {"status": "ok"}


app.include_router(inference.router)
app.include_router(audit.router)
app.include_router(oversight.router)
app.include_router(certificates.router)
