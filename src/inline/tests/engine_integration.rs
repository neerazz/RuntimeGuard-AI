use chrono::{TimeZone, Utc};
use ed25519_dalek::SigningKey;
use runtimeguard_inline::durable_log::{DurableLog, SyncPolicy};
use runtimeguard_inline::engine::{EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::policy::CompiledPolicy;
use runtimeguard_inline::types::{ComplianceRecord, Decision, InferenceRequest, PolicyResult};
use std::collections::HashSet;
use std::sync::Arc;

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

#[test]
fn durable_evaluation_returns_receipt_and_survives_reopen() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let config = EngineConfig {
        log_directory: directory.path().to_path_buf(),
        num_shards: 4,
        sync_policy: SyncPolicy::Data,
        policy: CompiledPolicy::example_v1().expect("compile policy"),
        receipt_signing_key: SigningKey::from_bytes(&[0x41; 32]),
    };
    let engine = InlinePolicyEngine::open(config.clone()).expect("open engine");

    let outcome = engine
        .evaluate(&request("req-1", "contains SSN 123-45-6789"))
        .expect("evaluate request");

    assert_eq!(outcome.policy_result.decision, Decision::Blocked);
    assert!(outcome.evidence.durable);
    drop(engine);

    let reopened = InlinePolicyEngine::open(config).expect("reopen engine");
    let records = reopened.recover_records().expect("recover records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].request_id, "req-1");
    assert_eq!(records[0].policy_result.decision, Decision::Blocked);
}

#[test]
fn zero_shards_is_rejected_instead_of_panicking() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let config = EngineConfig {
        log_directory: directory.path().to_path_buf(),
        num_shards: 0,
        sync_policy: SyncPolicy::Data,
        policy: CompiledPolicy::example_v1().expect("compile policy"),
        receipt_signing_key: SigningKey::from_bytes(&[0x41; 32]),
    };

    assert!(InlinePolicyEngine::open(config).is_err());
}

#[test]
fn reopening_with_a_different_shard_count_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let mut config = EngineConfig {
        log_directory: directory.path().to_path_buf(),
        num_shards: 4,
        sync_policy: SyncPolicy::Data,
        policy: CompiledPolicy::example_v1().expect("compile policy"),
        receipt_signing_key: SigningKey::from_bytes(&[0x41; 32]),
    };
    drop(InlinePolicyEngine::open(config.clone()).expect("open engine"));
    config.num_shards = 2;

    assert!(InlinePolicyEngine::open(config).is_err());
}

#[test]
fn reopening_rejects_a_validly_framed_sequence_gap() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = CompiledPolicy::example_v1().expect("compile policy");
    let config = EngineConfig {
        log_directory: directory.path().to_path_buf(),
        num_shards: 1,
        sync_policy: SyncPolicy::Data,
        policy: policy.clone(),
        receipt_signing_key: SigningKey::from_bytes(&[0x41; 32]),
    };
    let engine = InlinePolicyEngine::open(config.clone()).expect("open engine");
    engine
        .evaluate(&request("request-0", "safe prompt"))
        .expect("evaluate request");
    drop(engine);

    let gap_request = request("request-7", "safe prompt");
    let mut log = DurableLog::open(directory.path().join("shard-0.rgl"), SyncPolicy::Data)
        .expect("open shard log");
    log.append(&ComplianceRecord::from_evaluation(
        &gap_request,
        PolicyResult::allow(),
        policy.descriptor().clone(),
        0,
        7,
        gap_request.timestamp,
    ))
    .expect("append valid frame with sequence gap");
    drop(log);

    assert!(InlinePolicyEngine::open(config).is_err());
}

#[test]
fn concurrent_durable_evaluations_recover_every_record_once() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let engine = Arc::new(
        InlinePolicyEngine::open(EngineConfig {
            log_directory: directory.path().to_path_buf(),
            num_shards: 4,
            sync_policy: SyncPolicy::Data,
            policy: CompiledPolicy::example_v1().expect("compile policy"),
            receipt_signing_key: SigningKey::from_bytes(&[0x41; 32]),
        })
        .expect("open engine"),
    );

    std::thread::scope(|scope| {
        for worker in 0..4 {
            let engine = Arc::clone(&engine);
            scope.spawn(move || {
                for offset in 0..25 {
                    engine
                        .evaluate(&request(
                            &format!("worker-{worker}-request-{offset}"),
                            "safe prompt",
                        ))
                        .expect("evaluate request");
                }
            });
        }
    });

    let records = engine.recover_records().expect("recover records");
    let ids: HashSet<&str> = records
        .iter()
        .map(|record| record.request_id.as_str())
        .collect();
    let sequences: Vec<u64> = records.iter().map(|record| record.sequence).collect();
    assert_eq!(records.len(), 100);
    assert_eq!(ids.len(), 100);
    assert_eq!(sequences, (0..100).collect::<Vec<_>>());
}
