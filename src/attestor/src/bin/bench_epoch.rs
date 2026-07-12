use anyhow::{bail, Context, Result};
use chrono::{TimeZone, Utc};
use clap::Parser;
use ed25519_dalek::SigningKey;
use runtimeguard_attestor::epoch::seal_epoch;
use runtimeguard_attestor::merkle::MerkleTree;
use runtimeguard_inline::types::{
    ComplianceRecord, InferenceRequest, PolicyDescriptor, PolicyResult,
};
use serde::Serialize;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Parser)]
#[command(about = "Benchmark real RuntimeGuard signed-epoch operations")]
struct Args {
    #[arg(long)]
    records: usize,
    #[arg(long, default_value_t = 30)]
    repetitions: usize,
    #[arg(long, default_value_t = 5)]
    warmup: usize,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value = "manual")]
    run_id: String,
}

#[derive(Debug, Serialize)]
struct Sample {
    run_id: String,
    record_count: usize,
    repetition: usize,
    sampled_leaf_index: usize,
    seal_epoch_ns: u128,
    build_tree_ns: u128,
    generate_proof_ns: u128,
    verify_proof_ns: u128,
    verify_signature_ns: u128,
    root_matches: bool,
    proof_valid: bool,
    signature_valid: bool,
}

fn fixture_records(count: usize) -> Vec<ComplianceRecord> {
    let policy = PolicyDescriptor::from_bytes(
        "benchmark-policy",
        "v2",
        b"deterministic benchmark policy fixture",
    );
    (0..count)
        .map(|index| {
            let timestamp = Utc
                .timestamp_opt(1_700_000_000 + index as i64, 0)
                .single()
                .expect("valid benchmark timestamp");
            let request = InferenceRequest {
                id: format!("request-{index}"),
                timestamp,
                model_id: "benchmark-model".to_owned(),
                prompt: "deterministic benchmark prompt".to_owned(),
                input_data: "deterministic benchmark input".to_owned(),
                context: "deterministic benchmark context".to_owned(),
                user_id: None,
            };
            ComplianceRecord::from_evaluation(
                &request,
                PolicyResult::allow(),
                policy.clone(),
                (index % 4) as u16,
                index as u64,
                timestamp,
            )
        })
        .collect()
}

fn measure_once(
    run_id: &str,
    records: &[ComplianceRecord],
    signing_key: &SigningKey,
    repetition: usize,
) -> Result<Sample> {
    let seal_started = Instant::now();
    let signed = seal_epoch(
        format!("{run_id}-epoch-{repetition}"),
        records,
        [0_u8; 32],
        signing_key,
    )?;
    let seal_epoch_ns = seal_started.elapsed().as_nanos();

    let commitments = records
        .iter()
        .map(ComplianceRecord::commitment)
        .collect::<Vec<_>>();
    let commitment_slices = commitments
        .iter()
        .map(<[u8; 32]>::as_slice)
        .collect::<Vec<_>>();
    let tree_started = Instant::now();
    let tree = MerkleTree::from_data(&commitment_slices);
    let build_tree_ns = tree_started.elapsed().as_nanos();

    let sampled_leaf_index = repetition.wrapping_mul(7_919) % records.len();
    let proof_started = Instant::now();
    let proof = tree.generate_proof(sampled_leaf_index);
    let generate_proof_ns = proof_started.elapsed().as_nanos();

    let proof_verify_started = Instant::now();
    let proof_valid = MerkleTree::verify_proof(
        &commitments[sampled_leaf_index],
        sampled_leaf_index,
        &proof,
        &signed.statement.merkle_root,
    );
    let verify_proof_ns = proof_verify_started.elapsed().as_nanos();

    let signature_verify_started = Instant::now();
    let signature_valid = signed.verify_with(&signing_key.verifying_key());
    let verify_signature_ns = signature_verify_started.elapsed().as_nanos();

    Ok(Sample {
        run_id: run_id.to_owned(),
        record_count: records.len(),
        repetition,
        sampled_leaf_index,
        seal_epoch_ns,
        build_tree_ns,
        generate_proof_ns,
        verify_proof_ns,
        verify_signature_ns,
        root_matches: tree.root() == Some(signed.statement.merkle_root),
        proof_valid,
        signature_valid,
    })
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.records == 0 {
        bail!("records must be greater than zero");
    }
    if args.repetitions == 0 {
        bail!("repetitions must be greater than zero");
    }
    if args.output.exists() {
        bail!(
            "refusing to overwrite existing output {}",
            args.output.display()
        );
    }
    if let Some(parent) = args.output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let records = fixture_records(args.records);
    let signing_key = SigningKey::from_bytes(&[0x52; 32]);
    for repetition in 0..args.warmup {
        let _ = measure_once("warmup", &records, &signing_key, repetition)?;
    }

    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args.output)
        .with_context(|| format!("create output {}", args.output.display()))?;
    let mut writer = csv::Writer::from_writer(file);
    for repetition in 0..args.repetitions {
        let sample = measure_once(&args.run_id, &records, &signing_key, repetition)?;
        if !(sample.root_matches && sample.proof_valid && sample.signature_valid) {
            bail!("attestation correctness check failed at repetition {repetition}");
        }
        writer.serialize(sample)?;
    }
    writer.flush()?;
    println!(
        "wrote {} attestation samples to {}",
        args.repetitions,
        args.output.display()
    );
    Ok(())
}
