import random
import string
from typing import Generator, List
from uuid import uuid4

from src.core.models import InferenceRequest


def random_string(length: int = 6) -> str:
    return "".join(random.choice(string.ascii_lowercase) for _ in range(length))


def generate_requests(count: int = 10) -> List[InferenceRequest]:
    return [next(generate_request_stream()) for _ in range(count)]


def generate_request_stream() -> Generator[InferenceRequest, None, None]:
    contexts = ["credit_decision", "hiring", "generic", "medical_triage"]
    for _ in range(10**9):
        yield InferenceRequest(
            model_id="mock-compliance-v1",
            user_id=random_string(),
            context=random.choice(contexts),
            source_ip=f"10.0.0.{random.randint(1, 200)}",
            input_data={
                "applicant_id": uuid4().hex,
                "feature": random.random(),
                "notes": random.choice(
                    ["", "social scoring is prohibited", "regular request", "urgent medical"]
                ),
            },
            model_confidence=round(random.uniform(0.4, 0.95), 3),
        )
