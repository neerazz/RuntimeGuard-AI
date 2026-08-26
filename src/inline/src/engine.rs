use crate::durable_log::{DurableLog, SyncPolicy};
use crate::policy::CompiledPolicy;
use crate::types::{ComplianceRecord, InferenceRequest, PolicyResult};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

const MANIFEST_FILE: &str = "manifest.json";
const WRITER_LOCK_FILE: &str = ".writer.lock";
const RECEIPT_DOMAIN: &[u8] = b"runtimeguard/commit-receipt/v2";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct EngineManifest {
    schema_version: u16,
    record_format: String,
    num_shards: usize,
    sync_policy: SyncPolicy,
    policy_digest: [u8; 32],
    receipt_verifying_key: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub log_directory: PathBuf,
    pub num_shards: usize,
    pub sync_policy: SyncPolicy,
    pub policy: CompiledPolicy,
    pub receipt_signing_key: SigningKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitReceipt {
    pub schema_version: u16,
    pub sequence: u64,
    pub request_id: String,
    pub request_commitment: [u8; 32],
    pub record_commitment: [u8; 32],
    pub durable: bool,
    pub verifying_key: [u8; 32],
    pub signature: Vec<u8>,
}

impl CommitReceipt {
    pub fn verify_with(&self, trusted_key: &VerifyingKey) -> bool {
        if trusted_key.to_bytes() != self.verifying_key {
            return false;
        }
        let Ok(signature) = Signature::from_slice(&self.signature) else {
            return false;
        };
        trusted_key
            .verify_strict(&receipt_bytes(self), &signature)
            .is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationOutcome {
    pub policy_result: PolicyResult,
    pub evidence: CommitReceipt,
}

#[derive(Debug, Clone)]
struct RecoveredRequest {
    request_commitment: [u8; 32],
    record_commitment: [u8; 32],
    sequence: u64,
    policy_result: PolicyResult,
}

/// Deterministic policy evaluation coupled to a sharded append-only evidence log.
///
/// `SyncPolicy::Data` and `SyncPolicy::Full` acknowledge an evaluation only after
/// the selected durability boundary completes. `SyncPolicy::None` is an explicit
/// low-latency baseline and returns a receipt with `durable == false`.
pub struct InlinePolicyEngine {
    _writer_lock: fs::File,
    logs: Vec<Mutex<DurableLog>>,
    next_sequence: AtomicU64,
    commit_lock: Mutex<()>,
    requests: Mutex<HashMap<String, RecoveredRequest>>,
    healthy: AtomicBool,
    policy: CompiledPolicy,
    receipt_signing_key: SigningKey,
    sync_policy: SyncPolicy,
}

impl InlinePolicyEngine {
    pub fn open(config: EngineConfig) -> Result<Self> {
        if config.num_shards == 0 {
            bail!("num_shards must be greater than zero");
        }
        fs::create_dir_all(&config.log_directory)
            .with_context(|| format!("create log directory {}", config.log_directory.display()))?;
        sync_parent_of(&config.log_directory)?;
        let writer_lock = acquire_writer_lease(&config.log_directory)?;
        validate_or_create_manifest(&config)?;

        let mut logs = Vec::with_capacity(config.num_shards);
        let mut recovered_sequences = Vec::new();
        let mut recovered_requests = HashMap::new();
        for shard_id in 0..config.num_shards {
            let path = config.log_directory.join(format!("shard-{shard_id}.rgl"));
            let log = DurableLog::open(path, config.sync_policy)?;
            for record in log.records()? {
                if usize::from(record.shard_id) != shard_id {
                    bail!(
                        "record {} declares shard {} but was recovered from shard {shard_id}",
                        record.sequence,
                        record.shard_id
                    );
                }
                let expected_shard = (record.sequence % config.num_shards as u64) as u16;
                if record.shard_id != expected_shard {
                    bail!(
                        "record {} is in shard {} but sequence mapping requires shard {expected_shard}",
                        record.sequence,
                        record.shard_id
                    );
                }
                if recovered_requests
                    .insert(
                        record.request_id.clone(),
                        RecoveredRequest {
                            request_commitment: record.request_commitment,
                            record_commitment: record.commitment(),
                            sequence: record.sequence,
                            policy_result: record.policy_result.clone(),
                        },
                    )
                    .is_some()
                {
                    bail!("duplicate request ID {} in evidence log", record.request_id);
                }
                recovered_sequences.push(record.sequence);
            }
            logs.push(Mutex::new(log));
        }
        recovered_sequences.sort_unstable();
        for (expected, actual) in recovered_sequences.iter().copied().enumerate() {
            if actual != expected as u64 {
                bail!(
                    "evidence sequence is not a contiguous prefix: expected {expected}, found {actual}"
                );
            }
        }

        Ok(Self {
            _writer_lock: writer_lock,
            logs,
            next_sequence: AtomicU64::new(recovered_sequences.len() as u64),
            commit_lock: Mutex::new(()),
            requests: Mutex::new(recovered_requests),
            healthy: AtomicBool::new(true),
            policy: config.policy,
            receipt_signing_key: config.receipt_signing_key,
            sync_policy: config.sync_policy,
        })
    }

    fn compute_shard(&self, sequence: u64) -> usize {
        (sequence % self.logs.len() as u64) as usize
    }

    pub fn evaluate(&self, request: &InferenceRequest) -> Result<EvaluationOutcome> {
        let request_commitment = request.commitment();
        if !self.healthy.load(Ordering::Acquire) {
            bail!("evidence writer is fail-stopped after a previous append failure");
        }
        if let Some(outcome) = self.replay(request, request_commitment)? {
            return Ok(outcome);
        }
        let policy_result = self.policy.evaluate(&request.prompt);
        let _commit_guard = self
            .commit_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("evidence commit mutex poisoned"))?;
        if !self.healthy.load(Ordering::Acquire) {
            bail!("evidence writer is fail-stopped after a previous append failure");
        }
        if let Some(outcome) = self.replay(request, request_commitment)? {
            return Ok(outcome);
        }
        let sequence = self.next_sequence.load(Ordering::Relaxed);
        let shard_id = self.compute_shard(sequence);
        let record = ComplianceRecord::from_evaluation(
            request,
            policy_result.clone(),
            self.policy.descriptor().clone(),
            shard_id as u16,
            sequence,
            Utc::now(),
        );
        let append_result = self.logs[shard_id]
            .lock()
            .map_err(|_| anyhow::anyhow!("shard {shard_id} log mutex poisoned"))?
            .append(&record);
        let appended = match append_result {
            Ok(receipt) => receipt,
            Err(error) => {
                self.healthy.store(false, Ordering::Release);
                return Err(error);
            }
        };
        self.next_sequence.store(sequence + 1, Ordering::Release);
        let evidence = self.sign_receipt(
            sequence,
            &request.id,
            request_commitment,
            appended.record_commitment,
        );
        self.requests
            .lock()
            .map_err(|_| anyhow::anyhow!("request index mutex poisoned"))?
            .insert(
                request.id.clone(),
                RecoveredRequest {
                    request_commitment,
                    record_commitment: appended.record_commitment,
                    sequence,
                    policy_result: policy_result.clone(),
                },
            );

        Ok(EvaluationOutcome {
            policy_result,
            evidence,
        })
    }

    fn replay(
        &self,
        request: &InferenceRequest,
        request_commitment: [u8; 32],
    ) -> Result<Option<EvaluationOutcome>> {
        let recovered = self
            .requests
            .lock()
            .map_err(|_| anyhow::anyhow!("request index mutex poisoned"))?
            .get(&request.id)
            .cloned();
        let Some(recovered) = recovered else {
            return Ok(None);
        };
        if recovered.request_commitment != request_commitment {
            bail!(
                "request ID conflict: {} was already committed with different content",
                request.id
            );
        }
        Ok(Some(EvaluationOutcome {
            policy_result: recovered.policy_result,
            evidence: self.sign_receipt(
                recovered.sequence,
                &request.id,
                recovered.request_commitment,
                recovered.record_commitment,
            ),
        }))
    }

    fn sign_receipt(
        &self,
        sequence: u64,
        request_id: &str,
        request_commitment: [u8; 32],
        record_commitment: [u8; 32],
    ) -> CommitReceipt {
        let mut receipt = CommitReceipt {
            schema_version: 2,
            sequence,
            request_id: request_id.to_owned(),
            request_commitment,
            record_commitment,
            durable: self.sync_policy != SyncPolicy::None,
            verifying_key: self.receipt_signing_key.verifying_key().to_bytes(),
            signature: Vec::new(),
        };
        receipt.signature = self
            .receipt_signing_key
            .sign(&receipt_bytes(&receipt))
            .to_bytes()
            .to_vec();
        receipt
    }

    pub fn recover_records(&self) -> Result<Vec<ComplianceRecord>> {
        let mut records = Vec::new();
        for (shard_id, log) in self.logs.iter().enumerate() {
            records.extend(
                log.lock()
                    .map_err(|_| anyhow::anyhow!("shard {shard_id} log mutex poisoned"))?
                    .records()?,
            );
        }
        records.sort_by_key(|record| record.sequence);
        Ok(records)
    }
}

fn acquire_writer_lease(directory: &std::path::Path) -> Result<fs::File> {
    let path = directory.join(WRITER_LOCK_FILE);
    let existed = path.exists();
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("open writer lease {}", path.display()))?;
    if !existed {
        file.sync_all()?;
        sync_directory(directory)?;
    }
    file.try_lock_exclusive()
        .with_context(|| format!("acquire writer lease {}", path.display()))?;
    Ok(file)
}

fn validate_or_create_manifest(config: &EngineConfig) -> Result<()> {
    let path = config.log_directory.join(MANIFEST_FILE);
    let expected = EngineManifest {
        schema_version: 2,
        record_format: "RGL2".to_owned(),
        num_shards: config.num_shards,
        sync_policy: config.sync_policy,
        policy_digest: config.policy.descriptor().digest,
        receipt_verifying_key: config.receipt_signing_key.verifying_key().to_bytes(),
    };
    if path.exists() {
        let bytes = fs::read(&path).with_context(|| format!("read manifest {}", path.display()))?;
        let actual: EngineManifest = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse manifest {}", path.display()))?;
        if actual != expected {
            bail!(
                "engine configuration conflicts with manifest {}:\n{}",
                path.display(),
                describe_manifest_conflict(&expected, &actual)
            );
        }
        return Ok(());
    }

    let bytes = serde_json::to_vec_pretty(&expected)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("create manifest {}", path.display()))?;
    file.write_all(&bytes)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    sync_directory(&config.log_directory)?;
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &std::path::Path) -> Result<()> {
    fs::File::open(path)
        .with_context(|| format!("open evidence directory {}", path.display()))?
        .sync_all()
        .with_context(|| format!("synchronize evidence directory {}", path.display()))
}

#[cfg(unix)]
fn sync_parent_of(path: &std::path::Path) -> Result<()> {
    sync_directory(parent_directory_for_sync(path))
}

/// Returns the actual directory containing `path` for a metadata sync.
///
/// Rust represents the parent of a single relative component (`evidence`) as
/// an empty path. POSIX treats `open("")` as ENOENT, while the intended parent
/// is the current directory. Nested and absolute paths retain their real
/// parent unchanged.
fn parent_directory_for_sync(path: &std::path::Path) -> &std::path::Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
}

#[cfg(not(unix))]
fn sync_directory(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent_of(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

/// Renders only the fields that actually differ, in operator terms.
///
/// The common cause is an edited policy source against an existing evidence
/// directory, so the message names that case instead of dumping byte arrays.
fn describe_manifest_conflict(expected: &EngineManifest, actual: &EngineManifest) -> String {
    let mut lines = Vec::new();
    let mut diff = |field: &str, expected: String, found: String| {
        if expected != found {
            lines.push(format!(
                "  {field}: configured {expected}, manifest {found}"
            ));
        }
    };
    diff(
        "schema_version",
        expected.schema_version.to_string(),
        actual.schema_version.to_string(),
    );
    diff(
        "record_format",
        expected.record_format.clone(),
        actual.record_format.clone(),
    );
    diff(
        "num_shards",
        expected.num_shards.to_string(),
        actual.num_shards.to_string(),
    );
    diff(
        "sync_policy",
        format!("{:?}", expected.sync_policy),
        format!("{:?}", actual.sync_policy),
    );
    diff(
        "policy_digest",
        hex(&expected.policy_digest),
        hex(&actual.policy_digest),
    );
    diff(
        "receipt_verifying_key",
        hex(&expected.receipt_verifying_key),
        hex(&actual.receipt_verifying_key),
    );
    if expected.policy_digest != actual.policy_digest {
        lines.push(
            "  hint: the policy source changed. Policy identity is the exact source bytes, \
             so use a new evidence directory for the new policy version."
                .to_owned(),
        );
    }
    lines.join("\n")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn receipt_bytes(receipt: &CommitReceipt) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(192);
    bytes.extend_from_slice(RECEIPT_DOMAIN);
    push_field(&mut bytes, &receipt.schema_version.to_be_bytes());
    push_field(&mut bytes, &receipt.sequence.to_be_bytes());
    push_field(&mut bytes, receipt.request_id.as_bytes());
    push_field(&mut bytes, &receipt.request_commitment);
    push_field(&mut bytes, &receipt.record_commitment);
    push_field(&mut bytes, &[u8::from(receipt.durable)]);
    push_field(&mut bytes, &receipt.verifying_key);
    bytes
}

fn push_field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use super::parent_directory_for_sync;
    use std::path::Path;

    #[test]
    fn bare_evidence_directory_syncs_its_real_parent_directory() {
        // `runtimeguard evaluate --evidence-dir evidence` is the documented
        // quickstart shape. Its Path parent is an empty component, which must
        // resolve to the current directory instead of attempting to open "".
        assert_eq!(
            parent_directory_for_sync(Path::new("evidence")),
            Path::new(".")
        );
        assert_eq!(
            parent_directory_for_sync(Path::new("state/evidence")),
            Path::new("state")
        );
    }
}
