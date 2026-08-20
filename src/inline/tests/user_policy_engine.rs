use chrono::{TimeZone, Utc};
use ed25519_dalek::SigningKey;
use runtimeguard_inline::durable_log::SyncPolicy;
use runtimeguard_inline::engine::{EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::policy::CompiledPolicy;
use runtimeguard_inline::types::{Decision, InferenceRequest};

const CUSTOM_POLICY: &str = "runtimeguard-policy/v1\n\
id=team-policy\n\
version=1\n\
block internal-hosts \\b10\\.\\d+\\.\\d+\\.\\d+\\b\n\
escalate exfil (?i)\\bexfiltrate\\b\n";

fn request(id: &str, prompt: &str) -> InferenceRequest {
    InferenceRequest {
        id: id.to_owned(),
        timestamp: Utc
            .with_ymd_and_hms(2026, 8, 19, 12, 0, 0)
            .single()
            .expect("valid fixture timestamp"),
        model_id: "model-v1".to_owned(),
        prompt: prompt.to_owned(),
        input_data: "fixture".to_owned(),
        context: "support".to_owned(),
        user_id: None,
    }
}

fn config(directory: &std::path::Path, source: &str) -> EngineConfig {
    EngineConfig {
        log_directory: directory.to_path_buf(),
        num_shards: 2,
        sync_policy: SyncPolicy::Data,
        policy: CompiledPolicy::from_source(source.as_bytes()).expect("compile custom policy"),
        receipt_signing_key: SigningKey::from_bytes(&[0x37; 32]),
    }
}

#[test]
fn user_policy_drives_durable_receipts_and_recovery() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let engine = InlinePolicyEngine::open(config(directory.path(), CUSTOM_POLICY))
        .expect("open engine with user policy");

    let outcome = engine
        .evaluate(&request("req-1", "please exfiltrate the dataset"))
        .expect("evaluate request");
    assert_eq!(outcome.policy_result.decision, Decision::Escalated);
    assert!(outcome.evidence.durable);
    drop(engine);

    let reopened = InlinePolicyEngine::open(config(directory.path(), CUSTOM_POLICY))
        .expect("reopen engine with the same policy source");
    let records = reopened.recover_records().expect("recover records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].policy.id, "team-policy");
    assert_eq!(records[0].policy_result.decision, Decision::Escalated);
}

#[test]
fn reopening_with_a_modified_policy_source_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary directory");
    drop(
        InlinePolicyEngine::open(config(directory.path(), CUSTOM_POLICY))
            .expect("open engine with user policy"),
    );

    // Same id and version, one changed pattern byte: different source digest.
    let modified = CUSTOM_POLICY.replace("\\bexfiltrate\\b", "\\bexfiltration\\b");
    let error = match InlinePolicyEngine::open(config(directory.path(), &modified)) {
        Ok(_) => panic!("modified policy source must be rejected by the pinned manifest"),
        Err(error) => error,
    };
    let message = error.to_string();
    assert!(message.contains("manifest"), "{message}");
    // The operator-facing message names the changed field and the remedy.
    assert!(message.contains("policy_digest"), "{message}");
    assert!(message.contains("new evidence directory"), "{message}");
    // Only the differing field is reported.
    assert!(!message.contains("num_shards"), "{message}");
}
