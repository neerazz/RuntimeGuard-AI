use chrono::{TimeZone, Utc};
use ed25519_dalek::SigningKey;
use runtimeguard_attestor::epoch::{epoch_proof, seal_epoch, verify_record_inclusion};
use runtimeguard_inline::types::{
    ComplianceRecord, InferenceRequest, PolicyDescriptor, PolicyResult,
};

fn records(count: u64) -> Vec<ComplianceRecord> {
    let policy = PolicyDescriptor::from_bytes("policy", "v1", b"policy bytes");
    (0..count)
        .map(|sequence| {
            let timestamp = Utc
                .with_ymd_and_hms(2026, 7, 11, 15, 0, sequence as u32)
                .single()
                .expect("valid fixture timestamp");
            let request = InferenceRequest {
                id: format!("req-{sequence}"),
                timestamp,
                model_id: "model-v1".to_owned(),
                prompt: "fixture".to_owned(),
                input_data: "fixture".to_owned(),
                context: "fixture".to_owned(),
                user_id: None,
            };
            ComplianceRecord::from_evaluation(
                &request,
                PolicyResult::allow(),
                policy.clone(),
                0,
                sequence,
                timestamp,
            )
        })
        .collect()
}

#[test]
fn signed_epoch_binds_contiguous_records_and_verifies() {
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let records = records(5);
    let signed = seal_epoch("epoch-1", &records, [0_u8; 32], &signing_key).expect("seal epoch");

    assert!(signed.verify_with(&signing_key.verifying_key()));
    assert_eq!(signed.statement.first_sequence, 0);
    assert_eq!(signed.statement.last_sequence, 4);
    assert_eq!(signed.statement.record_count, 5);

    let proof = epoch_proof(&records, 4).expect("inclusion proof");
    assert!(verify_record_inclusion(
        &records[4],
        &proof,
        &signed,
        &signing_key.verifying_key(),
    ));
}

#[test]
fn signed_epoch_rejects_statement_tampering() {
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let mut signed =
        seal_epoch("epoch-1", &records(3), [0_u8; 32], &signing_key).expect("seal epoch");
    signed.statement.record_count += 1;

    assert!(!signed.verify_with(&signing_key.verifying_key()));
}

#[test]
fn signed_epoch_is_not_trusted_under_an_attacker_supplied_key() {
    let trusted_key = SigningKey::from_bytes(&[7_u8; 32]);
    let attacker_key = SigningKey::from_bytes(&[9_u8; 32]);
    let signed =
        seal_epoch("epoch-attack", &records(3), [0_u8; 32], &attacker_key).expect("seal epoch");

    assert!(!signed.verify_with(&trusted_key.verifying_key()));
}

#[test]
fn sealing_rejects_sequence_gaps() {
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let mut records = records(3);
    records[2].sequence = 7;

    assert!(seal_epoch("epoch-1", &records, [0_u8; 32], &signing_key).is_err());
}
