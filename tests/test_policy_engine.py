from src.core.models import InferenceRequest
from src.core.policy_engine import Decision, PolicyEngine


def test_confidence_rule_blocks_low_confidence():
    engine = PolicyEngine()
    req = InferenceRequest(model_id="m1", input_data={}, model_confidence=0.2)
    result = engine.evaluate(req, None)
    assert result.decision == Decision.BLOCK
    assert "EU-AI-ACT-ART14-CONF" in result.rules_triggered


def test_high_stakes_escalates():
    engine = PolicyEngine()
    req = InferenceRequest(
        model_id="m1",
        input_data={"patient": "x"},
        context="medical_triage",
        model_confidence=0.9,
    )
    result = engine.evaluate(req, None)
    assert result.decision == Decision.ESCALATE
    assert "EU-AI-ACT-ART14-HS" in result.rules_triggered
