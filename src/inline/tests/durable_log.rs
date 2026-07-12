use chrono::{TimeZone, Utc};
use runtimeguard_inline::durable_log::{DurableLog, SyncPolicy};
use runtimeguard_inline::types::{
    ComplianceRecord, InferenceRequest, PolicyDescriptor, PolicyResult,
};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

fn record(request_id: &str, sequence: u64) -> ComplianceRecord {
    let timestamp = Utc
        .with_ymd_and_hms(2026, 7, 11, 13, 0, sequence as u32)
        .single()
        .expect("valid fixture timestamp");
    let request = InferenceRequest {
        id: request_id.to_owned(),
        timestamp,
        model_id: "model-v1".to_owned(),
        prompt: "fixture prompt".to_owned(),
        input_data: "fixture input".to_owned(),
        context: "fixture context".to_owned(),
        user_id: None,
    };
    ComplianceRecord::from_evaluation(
        &request,
        PolicyResult::allow(),
        PolicyDescriptor::from_bytes("fixture-policy", "v1", b"fixture policy"),
        0,
        sequence,
        timestamp,
    )
}

#[test]
fn durable_log_round_trips_records_and_receipts() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let path = dir.path().join("records.rgl");
    let mut log = DurableLog::open(&path, SyncPolicy::Data).expect("open log");

    let first = record("req-1", 0);
    let second = record("req-2", 1);
    let first_receipt = log.append(&first).expect("append first record");
    let second_receipt = log.append(&second).expect("append second record");

    assert_eq!(first_receipt.sequence, 0);
    assert_eq!(first_receipt.record_commitment, first.commitment());
    assert!(first_receipt.durable);
    assert!(second_receipt.offset > first_receipt.offset);
    drop(log);

    let reopened = DurableLog::open(&path, SyncPolicy::Data).expect("reopen log");
    assert_eq!(
        reopened.records().expect("recover records"),
        vec![first, second]
    );
}

#[test]
fn durable_log_truncates_only_an_incomplete_tail_frame() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let path = dir.path().join("records.rgl");
    let first = record("req-1", 0);
    let mut log = DurableLog::open(&path, SyncPolicy::Data).expect("open log");
    log.append(&first).expect("append record");
    drop(log);

    let clean_len = std::fs::metadata(&path).expect("metadata").len();
    OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("open tail")
        .write_all(b"RGL1\0\0")
        .expect("write incomplete frame");

    let reopened = DurableLog::open(&path, SyncPolicy::Data).expect("recover partial tail");
    assert_eq!(reopened.records().expect("records"), vec![first]);
    assert_eq!(std::fs::metadata(&path).expect("metadata").len(), clean_len);
}

#[test]
fn durable_log_rejects_tampering_in_a_complete_frame() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let path = dir.path().join("records.rgl");
    let mut log = DurableLog::open(&path, SyncPolicy::Data).expect("open log");
    log.append(&record("req-1", 0)).expect("append record");
    drop(log);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .expect("open for tampering");
    file.seek(SeekFrom::Start(8)).expect("seek to payload");
    let mut byte = [0_u8; 1];
    file.read_exact(&mut byte).expect("read payload byte");
    byte[0] ^= 0x01;
    file.seek(SeekFrom::Start(8)).expect("rewind to payload");
    file.write_all(&byte).expect("tamper payload");
    file.sync_data().expect("persist tamper");
    drop(file);

    assert!(DurableLog::open(&path, SyncPolicy::Data).is_err());
}
