use chrono::{TimeZone, Utc};
use ed25519_dalek::SigningKey;
use runtimeguard_inline::durable_log::SyncPolicy;
use runtimeguard_inline::engine::{EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::policy::CompiledPolicy;
use runtimeguard_inline::types::InferenceRequest;
use std::sync::Arc;
use std::thread;

fn request(id: &str, prompt: &str) -> InferenceRequest {
    InferenceRequest {
        id: id.to_owned(),
        timestamp: Utc
            .with_ymd_and_hms(2026, 7, 11, 14, 0, 0)
            .single()
            .expect("valid fixture timestamp"),
        model_id: "model-v1".to_owned(),
        prompt: prompt.to_owned(),
        input_data: "fixture".to_owned(),
        context: "support".to_owned(),
        user_id: Some("user-1".to_owned()),
    }
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[0x41; 32])
}

fn config(directory: &std::path::Path) -> EngineConfig {
    EngineConfig {
        log_directory: directory.to_path_buf(),
        num_shards: 4,
        sync_policy: SyncPolicy::Data,
        policy: CompiledPolicy::example_v1().expect("compile policy"),
        receipt_signing_key: signing_key(),
    }
}

#[test]
fn signed_receipt_verifies_under_an_external_trust_anchor() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let engine = InlinePolicyEngine::open(config(directory.path())).expect("open engine");

    let outcome = engine
        .evaluate(&request("signed-request", "safe prompt"))
        .expect("evaluate request");

    assert!(outcome.evidence.durable);
    assert!(outcome.evidence.verify_with(&signing_key().verifying_key()));
}

#[test]
fn idempotent_replay_returns_original_receipt_without_duplicate_record() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let engine = InlinePolicyEngine::open(config(directory.path())).expect("open engine");
    let original_request = request("idempotent-request", "safe prompt");

    let first = engine
        .evaluate(&original_request)
        .expect("first evaluation");
    let replay = engine
        .evaluate(&original_request)
        .expect("idempotent replay");

    assert_eq!(first, replay);
    assert_eq!(engine.recover_records().expect("recover records").len(), 1);
}

#[test]
fn reused_request_id_with_different_commitment_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let engine = InlinePolicyEngine::open(config(directory.path())).expect("open engine");
    engine
        .evaluate(&request("conflicting-request", "safe prompt"))
        .expect("first evaluation");

    let error = engine
        .evaluate(&request("conflicting-request", "changed prompt"))
        .expect_err("conflicting replay must fail");

    assert!(error.to_string().contains("request ID conflict"));
    assert_eq!(engine.recover_records().expect("recover records").len(), 1);
}

#[test]
fn sequence_determines_protocol_stable_shard_assignment() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let engine = InlinePolicyEngine::open(config(directory.path())).expect("open engine");
    for index in 0..20 {
        engine
            .evaluate(&request(&format!("request-{index}"), "safe prompt"))
            .expect("evaluate request");
    }

    let records = engine.recover_records().expect("recover records");
    assert!(records
        .iter()
        .all(|record| record.shard_id == (record.sequence % 4) as u16));
}

#[test]
fn reopening_with_a_different_receipt_key_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let original = config(directory.path());
    drop(InlinePolicyEngine::open(original.clone()).expect("open engine"));
    let mut changed = original;
    changed.receipt_signing_key = SigningKey::from_bytes(&[0x42; 32]);

    assert!(InlinePolicyEngine::open(changed).is_err());
}

#[test]
fn concurrent_exact_replays_commit_only_once() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let engine = Arc::new(InlinePolicyEngine::open(config(directory.path())).expect("open engine"));
    let shared_request = request("concurrent-replay", "safe prompt");
    let handles = (0..8)
        .map(|_| {
            let engine = Arc::clone(&engine);
            let request = shared_request.clone();
            thread::spawn(move || engine.evaluate(&request).expect("evaluate replay"))
        })
        .collect::<Vec<_>>();
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().expect("worker did not panic"))
        .collect::<Vec<_>>();

    assert!(outcomes.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(engine.recover_records().expect("recover records").len(), 1);
}

#[test]
fn second_writer_for_the_same_log_directory_is_rejected_until_release() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = InlinePolicyEngine::open(config(directory.path())).expect("open first writer");

    let error = match InlinePolicyEngine::open(config(directory.path())) {
        Ok(_) => panic!("a concurrent writer must not open the same evidence directory"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("writer lease"));

    drop(first);
    InlinePolicyEngine::open(config(directory.path())).expect("reopen after writer release");
}
