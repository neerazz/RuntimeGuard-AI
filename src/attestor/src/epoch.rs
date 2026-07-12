use crate::merkle::{Hash, MerkleTree};
use anyhow::{bail, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use runtimeguard_inline::types::ComplianceRecord;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const EPOCH_DOMAIN: &[u8] = b"runtimeguard/signed-epoch/v2";
const STATEMENT_HASH_DOMAIN: &[u8] = b"runtimeguard/statement-hash/v2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpochStatement {
    pub schema_version: u16,
    pub epoch_id: String,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub record_count: u64,
    pub first_evaluated_at_micros: i64,
    pub last_evaluated_at_micros: i64,
    pub policy_id: String,
    pub policy_version: String,
    pub policy_digest: [u8; 32],
    pub merkle_root: Hash,
    pub previous_statement_hash: Hash,
    pub signer_key_id: [u8; 16],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedEpoch {
    pub statement: EpochStatement,
    pub verifying_key: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpochInclusionProof {
    pub index: u64,
    pub leaf_count: u64,
    pub siblings: Vec<Hash>,
}

impl SignedEpoch {
    /// Verifies against a public key obtained from an independently trusted source.
    /// The serialized key is informational and is never a trust root by itself.
    pub fn verify_with(&self, trusted_key: &VerifyingKey) -> bool {
        if trusted_key.to_bytes() != self.verifying_key {
            return false;
        }
        let Ok(signature) = Signature::from_slice(&self.signature) else {
            return false;
        };
        trusted_key
            .verify_strict(&statement_bytes(&self.statement), &signature)
            .is_ok()
    }
}

pub fn seal_epoch(
    epoch_id: impl Into<String>,
    records: &[ComplianceRecord],
    previous_statement_hash: Hash,
    signing_key: &SigningKey,
) -> Result<SignedEpoch> {
    if records.is_empty() {
        bail!("cannot seal an empty epoch");
    }

    let mut ordered: Vec<&ComplianceRecord> = records.iter().collect();
    ordered.sort_by_key(|record| record.sequence);
    for pair in ordered.windows(2) {
        if pair[1].sequence != pair[0].sequence + 1 {
            bail!(
                "epoch sequences must be contiguous: {} followed by {}",
                pair[0].sequence,
                pair[1].sequence
            );
        }
    }

    let policy = &ordered[0].policy;
    if ordered.iter().any(|record| record.policy != *policy) {
        bail!("all records in an epoch must use the same policy descriptor");
    }

    let commitments: Vec<Hash> = ordered.iter().map(|record| record.commitment()).collect();
    let commitment_slices: Vec<&[u8]> = commitments.iter().map(Hash::as_slice).collect();
    let tree = MerkleTree::from_data(&commitment_slices);
    let verifying_key = signing_key.verifying_key().to_bytes();
    let key_digest: Hash = Sha256::digest(verifying_key).into();
    let statement = EpochStatement {
        schema_version: 2,
        epoch_id: epoch_id.into(),
        first_sequence: ordered[0].sequence,
        last_sequence: ordered[ordered.len() - 1].sequence,
        record_count: ordered.len() as u64,
        first_evaluated_at_micros: ordered[0].evaluated_at.timestamp_micros(),
        last_evaluated_at_micros: ordered[ordered.len() - 1].evaluated_at.timestamp_micros(),
        policy_id: policy.id.clone(),
        policy_version: policy.version.clone(),
        policy_digest: policy.digest,
        merkle_root: tree.root().expect("non-empty epoch has a root"),
        previous_statement_hash,
        signer_key_id: key_digest[..16].try_into().expect("fixed key ID"),
    };
    let signature = signing_key.sign(&statement_bytes(&statement));

    Ok(SignedEpoch {
        statement,
        verifying_key,
        signature: signature.to_bytes().to_vec(),
    })
}

pub fn epoch_proof(records: &[ComplianceRecord], index: usize) -> Result<EpochInclusionProof> {
    if index >= records.len() {
        bail!(
            "record index {index} is outside epoch of size {}",
            records.len()
        );
    }
    let mut ordered: Vec<&ComplianceRecord> = records.iter().collect();
    ordered.sort_by_key(|record| record.sequence);
    let commitments: Vec<Hash> = ordered.iter().map(|record| record.commitment()).collect();
    let commitment_slices: Vec<&[u8]> = commitments.iter().map(Hash::as_slice).collect();
    Ok(EpochInclusionProof {
        index: index as u64,
        leaf_count: records.len() as u64,
        siblings: MerkleTree::from_data(&commitment_slices).generate_proof(index),
    })
}

pub fn statement_hash(statement: &EpochStatement) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(STATEMENT_HASH_DOMAIN);
    hasher.update(statement_bytes(statement));
    hasher.finalize().into()
}

pub fn verify_epoch_successor(
    previous: &SignedEpoch,
    next: &SignedEpoch,
    previous_trusted_key: &VerifyingKey,
    next_trusted_key: &VerifyingKey,
) -> bool {
    previous.verify_with(previous_trusted_key)
        && next.verify_with(next_trusted_key)
        && next.statement.previous_statement_hash == statement_hash(&previous.statement)
        && previous
            .statement
            .last_sequence
            .checked_add(1)
            .is_some_and(|expected| next.statement.first_sequence == expected)
}

/// Verifies signature, policy binding, sequence-to-index mapping, logical tree
/// size, and Merkle membership under an externally supplied trust anchor.
pub fn verify_record_inclusion(
    record: &ComplianceRecord,
    proof: &EpochInclusionProof,
    epoch: &SignedEpoch,
    trusted_key: &VerifyingKey,
) -> bool {
    let statement = &epoch.statement;
    if !epoch.verify_with(trusted_key)
        || proof.leaf_count != statement.record_count
        || record.sequence < statement.first_sequence
        || record.sequence > statement.last_sequence
        || proof.index != record.sequence - statement.first_sequence
        || record.policy.id != statement.policy_id
        || record.policy.version != statement.policy_version
        || record.policy.digest != statement.policy_digest
    {
        return false;
    }
    MerkleTree::verify_proof_with_size(
        &record.commitment(),
        proof.index as usize,
        proof.leaf_count as usize,
        &proof.siblings,
        &statement.merkle_root,
    )
}

fn statement_bytes(statement: &EpochStatement) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(320);
    bytes.extend_from_slice(EPOCH_DOMAIN);
    push_field(&mut bytes, &statement.schema_version.to_be_bytes());
    push_field(&mut bytes, statement.epoch_id.as_bytes());
    push_field(&mut bytes, &statement.first_sequence.to_be_bytes());
    push_field(&mut bytes, &statement.last_sequence.to_be_bytes());
    push_field(&mut bytes, &statement.record_count.to_be_bytes());
    push_field(
        &mut bytes,
        &statement.first_evaluated_at_micros.to_be_bytes(),
    );
    push_field(
        &mut bytes,
        &statement.last_evaluated_at_micros.to_be_bytes(),
    );
    push_field(&mut bytes, statement.policy_id.as_bytes());
    push_field(&mut bytes, statement.policy_version.as_bytes());
    push_field(&mut bytes, &statement.policy_digest);
    push_field(&mut bytes, &statement.merkle_root);
    push_field(&mut bytes, &statement.previous_statement_hash);
    push_field(&mut bytes, &statement.signer_key_id);
    bytes
}

fn push_field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}
