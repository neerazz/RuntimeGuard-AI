use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, ValueEnum};
use ed25519_dalek::SigningKey;
use runtimeguard_inline::durable_log::SyncPolicy;
use runtimeguard_inline::engine::{EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::policy::CompiledPolicy;
use runtimeguard_inline::types::{Decision, InferenceRequest, PolicyResult};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::time::Instant;

#[derive(Debug, Parser)]
#[command(about = "Run one reproducible RuntimeGuard inline-evidence benchmark condition")]
struct Args {
    #[arg(long)]
    run_id: String,
    #[arg(long, default_value_t = 10_000)]
    requests: usize,
    #[arg(long, default_value_t = 1_000)]
    warmup: usize,
    #[arg(long, default_value_t = 1)]
    threads: usize,
    #[arg(long, default_value_t = 1)]
    shards: usize,
    #[arg(long, value_enum)]
    sync: SyncArg,
    #[arg(long, value_enum, default_value_t = ModeArg::Evidence)]
    mode: ModeArg,
    #[arg(long, default_value_t = 256)]
    prompt_bytes: usize,
    #[arg(long)]
    log_directory: PathBuf,
    #[arg(long)]
    output: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ModeArg {
    PolicyOnly,
    Evidence,
}

impl ModeArg {
    fn label(self) -> &'static str {
        match self {
            Self::PolicyOnly => "policy_only",
            Self::Evidence => "evidence",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SyncArg {
    None,
    Data,
    Full,
}

impl SyncArg {
    fn policy(self) -> SyncPolicy {
        match self {
            Self::None => SyncPolicy::None,
            Self::Data => SyncPolicy::Data,
            Self::Full => SyncPolicy::Full,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Data => "data",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Serialize)]
struct Sample {
    run_id: String,
    mode: &'static str,
    sync_policy: &'static str,
    shards: usize,
    threads: usize,
    prompt_bytes: usize,
    request_index: usize,
    prompt_class: &'static str,
    decision: &'static str,
    latency_ns: u128,
    condition_elapsed_ns: u128,
    durable: bool,
}

enum Runner {
    PolicyOnly(CompiledPolicy),
    Evidence(Box<InlinePolicyEngine>),
}

impl Runner {
    fn evaluate(&self, request: &InferenceRequest) -> Result<(PolicyResult, bool)> {
        match self {
            Self::PolicyOnly(policy) => Ok((policy.evaluate(&request.prompt), false)),
            Self::Evidence(engine) => {
                let outcome = engine.evaluate(request)?;
                Ok((outcome.policy_result, outcome.evidence.durable))
            }
        }
    }

    fn recovered_record_count(&self) -> Result<Option<usize>> {
        match self {
            Self::PolicyOnly(_) => Ok(None),
            Self::Evidence(engine) => Ok(Some(engine.recover_records()?.len())),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    validate_args(&args)?;
    if args.output.exists() {
        bail!("refusing to overwrite output {}", args.output.display());
    }
    if matches!(args.mode, ModeArg::Evidence) {
        ensure_new_directory(&args.log_directory)?;
    }

    let warmup_directory = args.log_directory.join("warmup");
    let measured_directory = args.log_directory.join("measured");
    if args.warmup > 0 {
        let warmup_runner = Arc::new(open_runner(&args, warmup_directory)?);
        run_workload(&args, warmup_runner, args.warmup, false)?;
    }

    let runner = Arc::new(open_runner(&args, measured_directory)?);
    let samples = run_workload(&args, Arc::clone(&runner), args.requests, true)?;
    let recovered = runner.recovered_record_count()?;
    if let Some(recovered) = recovered {
        if recovered != args.requests {
            bail!(
                "evidence conservation failed: submitted {} records but recovered {recovered}",
                args.requests
            );
        }
    }

    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    let mut writer = csv::Writer::from_path(&args.output)
        .with_context(|| format!("open benchmark output {}", args.output.display()))?;
    for sample in samples {
        writer.serialize(sample)?;
    }
    writer.flush()?;
    println!(
        "wrote {} measured samples to {} (recovered {:?} records)",
        args.requests,
        args.output.display(),
        recovered
    );
    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if args.requests == 0 {
        bail!("requests must be greater than zero");
    }
    if args.threads == 0 {
        bail!("threads must be greater than zero");
    }
    if args.shards == 0 {
        bail!("shards must be greater than zero");
    }
    if args.prompt_bytes < 32 {
        bail!("prompt_bytes must be at least 32");
    }
    Ok(())
}

fn ensure_new_directory(path: &Path) -> Result<()> {
    if path.exists() {
        bail!(
            "refusing to reuse benchmark log directory {}; choose a new path",
            path.display()
        );
    }
    fs::create_dir_all(path)
        .with_context(|| format!("create benchmark log directory {}", path.display()))
}

fn open_runner(args: &Args, log_directory: PathBuf) -> Result<Runner> {
    let policy = CompiledPolicy::example_v1()?;
    match args.mode {
        ModeArg::PolicyOnly => Ok(Runner::PolicyOnly(policy)),
        ModeArg::Evidence => Ok(Runner::Evidence(Box::new(InlinePolicyEngine::open(
            EngineConfig {
                log_directory,
                num_shards: args.shards,
                sync_policy: args.sync.policy(),
                policy,
                receipt_signing_key: SigningKey::from_bytes(&[0x52; 32]),
            },
        )?))),
    }
}

fn run_workload(
    args: &Args,
    runner: Arc<Runner>,
    request_count: usize,
    record_samples: bool,
) -> Result<Vec<Sample>> {
    let barrier = Arc::new(Barrier::new(args.threads + 1));
    let (mut all_samples, condition_elapsed_ns) = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(args.threads);
        for worker in 0..args.threads {
            let runner = Arc::clone(&runner);
            let barrier = Arc::clone(&barrier);
            handles.push(scope.spawn(move || -> Result<Vec<Sample>> {
                let mut samples = Vec::new();
                barrier.wait();
                for index in (worker..request_count).step_by(args.threads) {
                    let (request, prompt_class) = fixture_request(index, args.prompt_bytes);
                    let start = Instant::now();
                    let (policy_result, durable) = runner.evaluate(&request)?;
                    let elapsed = start.elapsed();
                    if record_samples {
                        samples.push(Sample {
                            run_id: args.run_id.clone(),
                            mode: args.mode.label(),
                            sync_policy: match args.mode {
                                ModeArg::PolicyOnly => "not_applicable",
                                ModeArg::Evidence => args.sync.label(),
                            },
                            shards: args.shards,
                            threads: args.threads,
                            prompt_bytes: args.prompt_bytes,
                            request_index: index,
                            prompt_class,
                            decision: decision_label(&policy_result.decision),
                            latency_ns: elapsed.as_nanos(),
                            condition_elapsed_ns: 0,
                            durable,
                        });
                    }
                }
                Ok(samples)
            }));
        }
        barrier.wait();
        let condition_start = Instant::now();
        let mut samples = Vec::new();
        for handle in handles {
            samples.extend(handle.join().expect("benchmark worker panicked")?);
        }
        Ok::<(Vec<Sample>, u128), anyhow::Error>((samples, condition_start.elapsed().as_nanos()))
    })?;
    for sample in &mut all_samples {
        sample.condition_elapsed_ns = condition_elapsed_ns;
    }
    all_samples.sort_by_key(|sample| sample.request_index);
    Ok(all_samples)
}

fn fixture_request(index: usize, prompt_bytes: usize) -> (InferenceRequest, &'static str) {
    let (prefix, class) = match index % 10 {
        0 => ("SSN 123-45-6789 ", "blocked_ssn"),
        1 => ("attempt to bypass controls ", "escalated_keyword"),
        _ => ("ordinary support request ", "allowed"),
    };
    let padding = "x".repeat(prompt_bytes.saturating_sub(prefix.len()));
    (
        InferenceRequest {
            id: format!("request-{index}"),
            timestamp: Utc::now(),
            model_id: "benchmark-model".to_owned(),
            prompt: format!("{prefix}{padding}"),
            input_data: "benchmark-input".to_owned(),
            context: "benchmark".to_owned(),
            user_id: Some(format!("user-{}", index % 100)),
        },
        class,
    )
}

fn decision_label(decision: &Decision) -> &'static str {
    match decision {
        Decision::Allowed => "allowed",
        Decision::Blocked => "blocked",
        Decision::Escalated => "escalated",
    }
}
