use chrono::{TimeZone, Utc};
use runtimeguard_inline::types::{
    ComplianceRecord, InferenceRequest, PolicyDescriptor, PolicyResult,
};

fn request() -> InferenceRequest {
    InferenceRequest {
        id: "req-42".to_owned(),
        timestamp: Utc
            .with_ymd_and_hms(2026, 7, 11, 12, 30, 0)
            .single()
            .expect("valid fixture timestamp"),
        model_id: "model-v1".to_owned(),
        prompt: "Summarize this account without exposing 123-45-6789".to_owned(),
        input_data: "tenant=example".to_owned(),
        context: "support".to_owned(),
        user_id: Some("user-7".to_owned()),
    }
}

fn policy() -> PolicyDescriptor {
    PolicyDescriptor {
        id: "pii-guard".to_owned(),
        version: "2026-07-11".to_owned(),
        digest: [0xA5; 32],
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn request_commitment_is_deterministic_and_field_sensitive() {
    let original = request();
    let identical = request();
    let mut changed = request();
    changed.prompt.push('!');

    assert_eq!(original.commitment(), identical.commitment());
    assert_ne!(original.commitment(), changed.commitment());
    assert_eq!(
        hex(&original.commitment()),
        "f8995ff63e264a97d159588ad445c489dbb85e4056f115d6840def9f3dbf515a"
    );
}

#[test]
fn record_commitment_binds_policy_version_and_excludes_raw_prompt() {
    let evaluated_at = Utc
        .with_ymd_and_hms(2026, 7, 11, 12, 30, 1)
        .single()
        .expect("valid fixture timestamp");
    let original = ComplianceRecord::from_evaluation(
        &request(),
        PolicyResult::block("PII_DETECTED: SSN".to_owned()),
        policy(),
        2,
        17,
        evaluated_at,
    );
    let mut changed_policy = policy();
    changed_policy.version = "2026-07-12".to_owned();
    let changed = ComplianceRecord::from_evaluation(
        &request(),
        PolicyResult::block("PII_DETECTED: SSN".to_owned()),
        changed_policy,
        2,
        17,
        evaluated_at,
    );

    assert_ne!(original.commitment(), changed.commitment());
    assert_eq!(original.canonical_bytes().len(), 251);
    assert_eq!(
        hex(&original.commitment()),
        "32f1bec0d61e8c3a853e4f12c8bbdaf13353b96d6df475e87dc1b0d94cac92d3"
    );

    let serialized = serde_json::to_string(&original).expect("record serializes");
    assert!(!serialized.contains("123-45-6789"));
    assert!(serialized.contains("request_commitment"));
    assert!(serialized.contains("policy"));
}
