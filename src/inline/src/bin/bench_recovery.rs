use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Parser;
use ed25519_dalek::SigningKey;
use runtimeguard_inline::durable_log::SyncPolicy;
use runtimeguard_inline::engine::{EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::policy::CompiledPolicy;
use runtimeguard_inline::types::InferenceRequest;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Parser)]
#[command(about = "Benchmark RuntimeGuard evidence-log reopening and recovery")]
struct Args {
    #[arg(long)]
    run_id: String,
    #[arg(long)]
    records: usize,
    #[arg(long, default_value_t = 30)]
    repetitions: usize,
    #[arg(long, default_value_t = 5)]
    warmup: usize,
    #[arg(long, default_value_t = 4)]
    shards: usize,
    #[arg(long)]
    log_directory: PathBuf,
    #[arg(long)]
    output: PathBuf,
}

#[derive(Debug, Serialize)]
struct Sample {
    run_id: String,
    record_count: usize,
    shards: usize,
    repetition: usize,
    open_ns: u128,
    recover_ns: u128,
    recovered_count: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    validate(&args)?;
    prepare_log(&args)?;

    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    let mut writer = csv::Writer::from_path(&args.output)
        .with_context(|| format!("open output {}", args.output.display()))?;
    let total = args.warmup + args.repetitions;
    for iteration in 0..total {
        let open_start = Instant::now();
        let engine = InlinePolicyEngine::open(config(&args))?;
        let open_ns = open_start.elapsed().as_nanos();

        let recover_start = Instant::now();
        let recovered_count = engine.recover_records()?.len();
        let recover_ns = recover_start.elapsed().as_nanos();
        if recovered_count != args.records {
            bail!(
                "recovery conservation failed: expected {}, recovered {recovered_count}",
                args.records
            );
        }
        if iteration >= args.warmup {
            writer.serialize(Sample {
                run_id: args.run_id.clone(),
                record_count: args.records,
                shards: args.shards,
                repetition: iteration - args.warmup,
                open_ns,
                recover_ns,
                recovered_count,
            })?;
        }
    }
    writer.flush()?;
    println!(
        "wrote {} recovery samples for {} records to {}",
        args.repetitions,
        args.records,
        args.output.display()
    );
    Ok(())
}

fn validate(args: &Args) -> Result<()> {
    if args.records == 0 || args.repetitions == 0 || args.shards == 0 {
        bail!("records, repetitions, and shards must be greater than zero");
    }
    if args.log_directory.exists() {
        bail!(
            "refusing to reuse log directory {}",
            args.log_directory.display()
        );
    }
    if args.output.exists() {
        bail!("refusing to overwrite output {}", args.output.display());
    }
    Ok(())
}

fn config(args: &Args) -> EngineConfig {
    EngineConfig {
        log_directory: args.log_directory.clone(),
        num_shards: args.shards,
        sync_policy: SyncPolicy::None,
        policy: CompiledPolicy::example_v1().expect("static benchmark policy compiles"),
        receipt_signing_key: SigningKey::from_bytes(&[0x52; 32]),
    }
}

fn prepare_log(args: &Args) -> Result<()> {
    let engine = InlinePolicyEngine::open(config(args))?;
    for index in 0..args.records {
        engine.evaluate(&fixture_request(index))?;
    }
    let recovered = engine.recover_records()?.len();
    if recovered != args.records {
        bail!(
            "fixture conservation failed: wrote {}, recovered {recovered}",
            args.records
        );
    }
    drop(engine);
    Ok(())
}

fn fixture_request(index: usize) -> InferenceRequest {
    InferenceRequest {
        id: format!("recovery-request-{index}"),
        timestamp: Utc::now(),
        model_id: "benchmark-model".to_owned(),
        prompt: "ordinary support request".to_owned(),
        input_data: "benchmark-input".to_owned(),
        context: "benchmark".to_owned(),
        user_id: Some(format!("user-{}", index % 100)),
    }
}
