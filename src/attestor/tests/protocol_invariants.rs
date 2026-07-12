use chrono::{TimeZone, Utc};
use ed25519_dalek::SigningKey;
use runtimeguard_attestor::epoch::{
    epoch_proof, seal_epoch, statement_hash, verify_epoch_successor, verify_record_inclusion,
};
use runtimeguard_inline::types::{
    ComplianceRecord, InferenceRequest, PolicyDescriptor, PolicyResult,
};

fn records(start: u64, count: u64) -> Vec<ComplianceRecord> {
    let policy = PolicyDescriptor::from_bytes("policy", "v1", b"policy bytes");
    (start..start + count)
        .map(|sequence| {
            let timestamp = Utc
                .timestamp_opt(1_700_000_000 + sequence as i64, 0)
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
                (sequence % 4) as u16,
                sequence,
                timestamp,
            )
        })
        .collect()
}

#[test]
fn inclusion_rejects_wrong_index_tree_size_and_record() {
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let records = records(0, 5);
    let signed = seal_epoch("epoch-1", &records, [0_u8; 32], &signing_key).expect("seal epoch");
    let proof = epoch_proof(&records, 2).expect("inclusion proof");

    assert!(verify_record_inclusion(
        &records[2],
        &proof,
        &signed,
        &signing_key.verifying_key(),
    ));

    let mut wrong_index = proof.clone();
    wrong_index.index = 1;
    assert!(!verify_record_inclusion(
        &records[2],
        &wrong_index,
        &signed,
        &signing_key.verifying_key(),
    ));

    let mut wrong_size = proof.clone();
    wrong_size.leaf_count = 6;
    assert!(!verify_record_inclusion(
        &records[2],
        &wrong_size,
        &signed,
        &signing_key.verifying_key(),
    ));

    assert!(!verify_record_inclusion(
        &records[3],
        &proof,
        &signed,
        &signing_key.verifying_key(),
    ));
}

#[test]
fn chained_epoch_binds_previous_signed_statement() {
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let first =
        seal_epoch("epoch-1", &records(0, 3), [0_u8; 32], &signing_key).expect("seal first epoch");
    let second = seal_epoch(
        "epoch-2",
        &records(3, 3),
        statement_hash(&first.statement),
        &signing_key,
    )
    .expect("seal second epoch");

    assert_eq!(
        second.statement.previous_statement_hash,
        statement_hash(&first.statement)
    );
    assert!(second.verify_with(&signing_key.verifying_key()));
    assert!(verify_epoch_successor(
        &first,
        &second,
        &signing_key.verifying_key(),
        &signing_key.verifying_key(),
    ));

    let mut forked = second;
    forked.statement.first_sequence = 4;
    assert!(!verify_epoch_successor(
        &first,
        &forked,
        &signing_key.verifying_key(),
        &signing_key.verifying_key(),
    ));
}
