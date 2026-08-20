//! `runtimeguard` — the clone-to-first-policy command line for RuntimeGuard-AI.
//!
//! This binary is a thin operator surface over the existing `runtimeguard-inline`
//! protocol implementation. It adds no protocol semantics: every guarantee and
//! boundary is the one specified in `docs/protocol-v2.md`.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use ed25519_dalek::{SigningKey, VerifyingKey};
use runtimeguard_inline::durable_log::SyncPolicy;
use runtimeguard_inline::engine::{CommitReceipt, EngineConfig, InlinePolicyEngine};
use runtimeguard_inline::policy::CompiledPolicy;
use runtimeguard_inline::types::InferenceRequest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "runtimeguard",
    about = "Policy-bound, crash-recoverable AI decision evidence (research prototype)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate an Ed25519 receipt signing key and print its verifying key.
    Keygen {
        /// File to write the 32-byte signing key to, hex encoded.
        #[arg(long)]
        out: PathBuf,
        /// Overwrite an existing key file.
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Parse and compile a policy source file, reporting its pinned identity.
    Validate {
        /// Path to a `runtimeguard-policy/v1` source file.
        policy: PathBuf,
    },
    /// Evaluate one prompt under a policy and commit a signed evidence record.
    Evaluate {
        /// Path to a `runtimeguard-policy/v1` source file.
        #[arg(long)]
        policy: PathBuf,
        /// Evidence directory (created if absent; pinned by manifest afterwards).
        #[arg(long)]
        evidence_dir: PathBuf,
        /// Path to the hex-encoded Ed25519 signing key from `keygen`.
        #[arg(long)]
        signing_key: PathBuf,
        /// Caller-chosen unique request identifier.
        #[arg(long)]
        request_id: String,
        /// The prompt text to evaluate.
        #[arg(long)]
        prompt: String,
        /// Model identifier recorded in the compliance record.
        #[arg(long, default_value = "unspecified-model")]
        model_id: String,
        /// Durability boundary acknowledged before the receipt is returned.
        #[arg(long, value_enum, default_value_t = SyncArg::Data)]
        sync: SyncArg,
        /// Number of evidence log shards (fixed at first open by the manifest).
        #[arg(long, default_value_t = 2)]
        shards: usize,
    },
    /// Verify a signed commit receipt against an independently supplied key.
    VerifyReceipt {
        /// Path to a receipt JSON file produced by `evaluate`.
        #[arg(long)]
        receipt: PathBuf,
        /// Path to the hex-encoded verifying key (from `keygen` output).
        #[arg(long)]
        verifying_key: PathBuf,
        /// Require `durable = true` on the receipt.
        #[arg(long, default_value_t = false)]
        require_durable: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum SyncArg {
    /// Buffered append; the receipt reports durable = false.
    None,
    /// fdatasync-equivalent boundary before acknowledgement.
    Data,
    /// fsync-equivalent boundary before acknowledgement.
    Full,
}

impl From<SyncArg> for SyncPolicy {
    fn from(value: SyncArg) -> Self {
        match value {
            SyncArg::None => SyncPolicy::None,
            SyncArg::Data => SyncPolicy::Data,
            SyncArg::Full => SyncPolicy::Full,
        }
    }
}

#[derive(Serialize)]
struct EvaluateOutput {
    decision: String,
    rules_triggered: Vec<String>,
    confidence_micros: u32,
    durable: bool,
    sequence: u64,
    receipt: ReceiptEnvelope,
}

/// Hex-encoded wire form of a [`CommitReceipt`].
///
/// The engine holds commitments, keys, and signatures as byte arrays, which
/// serde renders as unreadable integer lists. Operators copy, diff, and paste
/// these values, so the CLI speaks hex on the way out and back in.
#[derive(Serialize, Deserialize)]
struct ReceiptEnvelope {
    schema_version: u16,
    sequence: u64,
    request_id: String,
    request_commitment: String,
    record_commitment: String,
    durable: bool,
    verifying_key: String,
    signature: String,
}

impl From<&CommitReceipt> for ReceiptEnvelope {
    fn from(receipt: &CommitReceipt) -> Self {
        Self {
            schema_version: receipt.schema_version,
            sequence: receipt.sequence,
            request_id: receipt.request_id.clone(),
            request_commitment: hex_encode(&receipt.request_commitment),
            record_commitment: hex_encode(&receipt.record_commitment),
            durable: receipt.durable,
            verifying_key: hex_encode(&receipt.verifying_key),
            signature: hex_encode(&receipt.signature),
        }
    }
}

impl ReceiptEnvelope {
    fn into_receipt(self) -> Result<CommitReceipt> {
        Ok(CommitReceipt {
            schema_version: self.schema_version,
            sequence: self.sequence,
            request_id: self.request_id,
            request_commitment: hex_array(&self.request_commitment, "request_commitment")?,
            record_commitment: hex_array(&self.record_commitment, "record_commitment")?,
            durable: self.durable,
            verifying_key: hex_array(&self.verifying_key, "verifying_key")?,
            signature: hex_decode(&self.signature).context("decode signature")?,
        })
    }
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Keygen { out, force } => keygen(&out, force),
        Command::Validate { policy } => validate(&policy),
        Command::Evaluate {
            policy,
            evidence_dir,
            signing_key,
            request_id,
            prompt,
            model_id,
            sync,
            shards,
        } => evaluate(
            &policy,
            &evidence_dir,
            &signing_key,
            &request_id,
            &prompt,
            &model_id,
            sync.into(),
            shards,
        ),
        Command::VerifyReceipt {
            receipt,
            verifying_key,
            require_durable,
        } => verify_receipt(&receipt, &verifying_key, require_durable),
    }
}

fn keygen(out: &Path, force: bool) -> Result<()> {
    if out.exists() && !force {
        bail!(
            "refusing to overwrite existing key file {} (use --force)",
            out.display()
        );
    }
    let mut secret = [0_u8; 32];
    getrandom::getrandom(&mut secret)
        .map_err(|error| anyhow::anyhow!("gather entropy for signing key: {error}"))?;
    let signing_key = SigningKey::from_bytes(&secret);
    fs::write(out, format!("{}\n", hex_encode(&secret)))
        .with_context(|| format!("write signing key {}", out.display()))?;
    restrict_permissions(out)?;
    println!(
        "signing key written to {}\nverifying key (share with auditors): {}",
        out.display(),
        hex_encode(&signing_key.verifying_key().to_bytes())
    );
    Ok(())
}

fn validate(policy_path: &Path) -> Result<()> {
    let policy = load_policy(policy_path)?;
    let descriptor = policy.descriptor();
    println!(
        "policy OK\nid: {}\nversion: {}\nrules: {}\nsource digest (sha256): {}",
        descriptor.id,
        descriptor.version,
        policy.rule_count(),
        hex_encode(&descriptor.digest)
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn evaluate(
    policy_path: &Path,
    evidence_dir: &Path,
    signing_key_path: &Path,
    request_id: &str,
    prompt: &str,
    model_id: &str,
    sync_policy: SyncPolicy,
    shards: usize,
) -> Result<()> {
    let policy = load_policy(policy_path)?;
    let signing_key = load_signing_key(signing_key_path)?;
    let engine = InlinePolicyEngine::open(EngineConfig {
        log_directory: evidence_dir.to_path_buf(),
        num_shards: shards,
        sync_policy,
        policy,
        receipt_signing_key: signing_key,
    })?;

    let request = InferenceRequest {
        id: request_id.to_owned(),
        timestamp: Utc::now(),
        model_id: model_id.to_owned(),
        prompt: prompt.to_owned(),
        input_data: String::new(),
        context: String::new(),
        user_id: None,
    };
    let outcome = engine.evaluate(&request)?;
    let output = EvaluateOutput {
        decision: format!("{:?}", outcome.policy_result.decision),
        rules_triggered: outcome.policy_result.rules_triggered.clone(),
        confidence_micros: outcome.policy_result.confidence_micros,
        durable: outcome.evidence.durable,
        sequence: outcome.evidence.sequence,
        receipt: (&outcome.evidence).into(),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn verify_receipt(receipt_path: &Path, key_path: &Path, require_durable: bool) -> Result<()> {
    let receipt_text = fs::read_to_string(receipt_path)
        .with_context(|| format!("read receipt {}", receipt_path.display()))?;
    let receipt: CommitReceipt = parse_receipt(&receipt_text)
        .with_context(|| format!("parse receipt {}", receipt_path.display()))?;
    let verifying_key = load_verifying_key(key_path)?;

    if !receipt.verify_with(&verifying_key) {
        bail!("INVALID: receipt signature does not verify under the supplied key");
    }
    if require_durable && !receipt.durable {
        bail!("INVALID: receipt verifies but durable = false (buffered acknowledgement)");
    }
    println!(
        "VALID: sequence {} request '{}' durable {}",
        receipt.sequence, receipt.request_id, receipt.durable
    );
    Ok(())
}

/// Accepts either a bare receipt object or the full `evaluate` output.
fn parse_receipt(text: &str) -> Result<CommitReceipt> {
    if let Ok(envelope) = serde_json::from_str::<ReceiptEnvelope>(text) {
        return envelope.into_receipt();
    }
    #[derive(serde::Deserialize)]
    struct Wrapper {
        receipt: ReceiptEnvelope,
    }
    serde_json::from_str::<Wrapper>(text)?
        .receipt
        .into_receipt()
}

fn load_policy(path: &Path) -> Result<CompiledPolicy> {
    let source =
        fs::read(path).with_context(|| format!("read policy source {}", path.display()))?;
    CompiledPolicy::from_source(&source)
        .with_context(|| format!("compile policy source {}", path.display()))
}

fn load_signing_key(path: &Path) -> Result<SigningKey> {
    let bytes = load_hex_key_file(path)?;
    Ok(SigningKey::from_bytes(&bytes))
}

fn load_verifying_key(path: &Path) -> Result<VerifyingKey> {
    let bytes = load_hex_key_file(path)?;
    VerifyingKey::from_bytes(&bytes)
        .with_context(|| format!("decode verifying key {}", path.display()))
}

fn load_hex_key_file(path: &Path) -> Result<[u8; 32]> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read key file {}", path.display()))?;
    hex_array(text.trim(), &format!("key file {}", path.display()))
}

/// Decodes hex into a fixed 32-byte array, naming the field on failure.
fn hex_array(text: &str, field: &str) -> Result<[u8; 32]> {
    hex_decode(text)
        .with_context(|| format!("decode {field}"))?
        .try_into()
        .map_err(|_| anyhow::anyhow!("{field} must hold exactly 32 bytes"))
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("restrict permissions on {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hex_decode(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        bail!("hex string has odd length");
    }
    (0..text.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&text[index..index + 2], 16)
                .with_context(|| format!("invalid hex at offset {index}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{hex_decode, hex_encode, parse_receipt};

    #[test]
    fn hex_round_trip() {
        let bytes = [0_u8, 1, 0xab, 0xff];
        let text = hex_encode(&bytes);
        assert_eq!(text, "0001abff");
        assert_eq!(hex_decode(&text).expect("decode"), bytes.to_vec());
        assert!(hex_decode("abc").is_err());
        assert!(hex_decode("zz").is_err());
    }

    #[test]
    fn parse_receipt_accepts_bare_and_wrapped_forms() {
        let zero32 = "00".repeat(32);
        let receipt = serde_json::json!({
            "schema_version": 2,
            "sequence": 0,
            "request_id": "r",
            "request_commitment": zero32,
            "record_commitment": zero32,
            "durable": true,
            "verifying_key": zero32,
            "signature": "",
        });
        assert!(parse_receipt(&receipt.to_string()).is_ok());
        let wrapped = serde_json::json!({ "decision": "Allowed", "receipt": receipt });
        assert!(parse_receipt(&wrapped.to_string()).is_ok());
        assert!(parse_receipt("{}").is_err());

        // A short commitment is a decode error, not a silent truncation.
        let mut short = receipt.clone();
        short["record_commitment"] = serde_json::json!("00");
        assert!(parse_receipt(&short.to_string()).is_err());
    }
}
