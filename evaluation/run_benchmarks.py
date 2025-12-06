"""Quick synthetic benchmark runner."""
from __future__ import annotations

import statistics
import time

from src.core.models import generate_id
from src.core.policy_engine import PolicyEngine
from src.core.storage import Storage
from src.mock.ai_model import MockModel
from src.mock.workload import generate_requests


def run_benchmark(count: int = 50) -> dict:
    storage = Storage()
    policy = PolicyEngine()
    model = MockModel()

    latencies = []
    for req in generate_requests(count):
        start = time.perf_counter()
        model_resp = model.predict(req)
        eval_result = policy.evaluate(req, model_resp)
        storage.log_request_and_decision(generate_id(), req, eval_result)
        latencies.append((time.perf_counter() - start) * 1000 + model_resp.latency_ms)

    return {
        "count": count,
        "p50_ms": round(statistics.median(latencies), 2),
        "p95_ms": round(statistics.quantiles(latencies, n=20)[-1], 2),
        "max_ms": round(max(latencies), 2),
        "merkle_root": storage.get_merkle_root(),
    }


if __name__ == "__main__":
    result = run_benchmark()
    print("Benchmark complete:", result)
