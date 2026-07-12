use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const REQUEST_DOMAIN: &[u8] = b"runtimeguard/request/v2";
const RECORD_DOMAIN: &[u8] = b"runtimeguard/compliance-record/v2";

fn update_field(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn finish(hasher: Sha256) -> [u8; 32] {
    hasher.finalize().into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub model_id: String,
    pub prompt: String, // The actual user input for policy checking
    pub input_data: String,
    pub context: String,
    pub user_id: Option<String>,
}

impl InferenceRequest {
    /// Returns a domain-separated commitment to every request field.
    /// Raw request content is not stored in the compliance record.
    pub fn commitment(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(REQUEST_DOMAIN);
        update_field(&mut hasher, self.id.as_bytes());
        update_field(
            &mut hasher,
            &self.timestamp.timestamp_micros().to_be_bytes(),
        );
        update_field(&mut hasher, self.model_id.as_bytes());
        update_field(&mut hasher, self.prompt.as_bytes());
        update_field(&mut hasher, self.input_data.as_bytes());
        update_field(&mut hasher, self.context.as_bytes());
        match &self.user_id {
            Some(user_id) => {
                hasher.update([1]);
                update_field(&mut hasher, user_id.as_bytes());
            }
            None => hasher.update([0]),
        }
        finish(hasher)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Allowed,
    Blocked,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyResult {
    pub decision: Decision,
    pub rules_triggered: Vec<String>,
    /// Fixed-point confidence in millionths. This field is deterministic and
    /// avoids committing platform/serialization-specific floating-point bytes.
    pub confidence_micros: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDescriptor {
    pub id: String,
    pub version: String,
    pub digest: [u8; 32],
}

impl PolicyDescriptor {
    pub fn from_bytes(id: impl Into<String>, version: impl Into<String>, bytes: &[u8]) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            digest: finish(Sha256::new_with_prefix(bytes)),
        }
    }
}

impl PolicyResult {
    pub fn allow() -> Self {
        Self {
            decision: Decision::Allowed,
            rules_triggered: vec![],
            confidence_micros: 1_000_000,
        }
    }

    pub fn block(reason: String) -> Self {
        Self {
            decision: Decision::Blocked,
            rules_triggered: vec![reason],
            confidence_micros: 0,
        }
    }

    pub fn escalate(reason: String) -> Self {
        Self {
            decision: Decision::Escalated,
            rules_triggered: vec![reason],
            confidence_micros: 500_000,
        }
    }

    pub fn merge(self, other: PolicyResult) -> PolicyResult {
        let decision = match (self.decision, other.decision) {
            (Decision::Blocked, _) | (_, Decision::Blocked) => Decision::Blocked,
            (Decision::Escalated, _) | (_, Decision::Escalated) => Decision::Escalated,
            _ => Decision::Allowed,
        };

        let mut rules = self.rules_triggered;
        rules.extend(other.rules_triggered);

        PolicyResult {
            decision,
            rules_triggered: rules,
            confidence_micros: u32::min(self.confidence_micros, other.confidence_micros),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplianceRecord {
    pub schema_version: u16,
    pub sequence: u64,
    pub request_id: String,
    pub evaluated_at: DateTime<Utc>,
    pub model_id: String,
    pub policy: PolicyDescriptor,
    pub policy_result: PolicyResult,
    pub request_commitment: [u8; 32],
    pub shard_id: u16,
}

impl ComplianceRecord {
    pub fn from_evaluation(
        request: &InferenceRequest,
        policy_result: PolicyResult,
        policy: PolicyDescriptor,
        shard_id: u16,
        sequence: u64,
        evaluated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            schema_version: 2,
            sequence,
            request_id: request.id.clone(),
            evaluated_at,
            model_id: request.model_id.clone(),
            policy,
            policy_result,
            request_commitment: request.commitment(),
            shard_id,
        }
    }

    /// Returns the protocol-stable binary representation committed by V2.
    /// JSON is only the local framed-log storage encoding and is not hashed.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(256);
        push_bytes(&mut bytes, &self.schema_version.to_be_bytes());
        push_bytes(&mut bytes, &self.sequence.to_be_bytes());
        push_bytes(&mut bytes, self.request_id.as_bytes());
        push_bytes(
            &mut bytes,
            &self.evaluated_at.timestamp_micros().to_be_bytes(),
        );
        push_bytes(&mut bytes, self.model_id.as_bytes());
        push_bytes(&mut bytes, self.policy.id.as_bytes());
        push_bytes(&mut bytes, self.policy.version.as_bytes());
        push_bytes(&mut bytes, &self.policy.digest);
        bytes.push(match self.policy_result.decision {
            Decision::Allowed => 0,
            Decision::Blocked => 1,
            Decision::Escalated => 2,
        });
        push_bytes(
            &mut bytes,
            &self.policy_result.confidence_micros.to_be_bytes(),
        );
        push_bytes(
            &mut bytes,
            &(self.policy_result.rules_triggered.len() as u64).to_be_bytes(),
        );
        for rule in &self.policy_result.rules_triggered {
            push_bytes(&mut bytes, rule.as_bytes());
        }
        push_bytes(&mut bytes, &self.request_commitment);
        push_bytes(&mut bytes, &self.shard_id.to_be_bytes());
        bytes
    }

    pub fn commitment(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(RECORD_DOMAIN);
        hasher.update(self.canonical_bytes());
        finish(hasher)
    }
}

fn push_bytes(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}
