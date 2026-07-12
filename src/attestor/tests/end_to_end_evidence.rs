use chrono::{TimeZone, Utc};
use ed25519_dalek::SigningKey;
use runtimeguard_attestor::epoch::{epoch_proof, seal_epoch, verify_record_inclusion};
use runtimeguard_inline::durable_log::SyncPolicy;
use runtimeguard_inline::engine::{EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::policy::CompiledPolicy;
use runtimeguard_inline::types::InferenceRequest;

fn request(index: usize) -> InferenceRequest {
    InferenceRequest {
        id: format!("request-{index}"),
        timestamp: Utc
            .with_ymd_and_hms(2026, 7, 11, 16, 0, index as u32)
            .single()
            .expect("valid fixture timestamp"),
        model_id: "model-v1".to_owned(),
        prompt: "ordinary support request".to_owned(),
        input_data: "fixture".to_owned(),
        context: "support".to_owned(),
        user_id: None,
    }
}

#[test]
fn durable_receipt_can_be_resolved_to_a_trusted_epoch_inclusion_proof() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let receipt_key = SigningKey::from_bytes(&[0x41; 32]);
    let epoch_key = SigningKey::from_bytes(&[0x52; 32]);
    let engine = InlinePolicyEngine::open(EngineConfig {
        log_directory: directory.path().to_path_buf(),
        num_shards: 2,
        sync_policy: SyncPolicy::Data,
        policy: CompiledPolicy::example_v1().expect("compile policy"),
        receipt_signing_key: receipt_key.clone(),
    })
    .expect("open engine");

    let outcomes = (0..5)
        .map(|index| engine.evaluate(&request(index)).expect("evaluate request"))
        .collect::<Vec<_>>();
    assert!(outcomes
        .iter()
        .all(|outcome| outcome.evidence.verify_with(&receipt_key.verifying_key())));

    let records = engine.recover_records().expect("recover durable records");
    let epoch = seal_epoch("epoch-1", &records, [0_u8; 32], &epoch_key).expect("seal epoch");
    let challenged = &outcomes[3].evidence;
    let record_index = records
        .iter()
        .position(|record| record.commitment() == challenged.record_commitment)
        .expect("receipt commitment is present in recovered records");
    let proof = epoch_proof(&records, record_index).expect("generate proof");

    assert!(verify_record_inclusion(
        &records[record_index],
        &proof,
        &epoch,
        &epoch_key.verifying_key(),
    ));
}
