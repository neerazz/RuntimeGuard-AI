use crate::types::{PolicyDescriptor, PolicyResult};
use anyhow::Result;
use regex::Regex;

pub const EXAMPLE_POLICY_V1_SOURCE: &[u8] = br#"runtimeguard-example-policy/v1
precedence=ssn,card,suspicious,allow
ssn=\b\d{3}-\d{2}-\d{4}\b
card=\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b
suspicious=(?i)\b(hack|exploit|bypass|jailbreak)\b
"#;

/// A compiled deterministic policy whose evidence descriptor is derived from
/// the exact source defining the evaluated rules.
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    descriptor: PolicyDescriptor,
    ssn_pattern: Regex,
    card_pattern: Regex,
    escalation_pattern: Regex,
}

impl CompiledPolicy {
    /// The small deterministic policy used by this research artifact.
    ///
    /// It is an evaluation fixture, not a complete enterprise policy language.
    pub fn example_v1() -> Result<Self> {
        Ok(Self {
            descriptor: PolicyDescriptor::from_bytes(
                "runtimeguard-example-policy",
                "v1",
                EXAMPLE_POLICY_V1_SOURCE,
            ),
            ssn_pattern: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")?,
            card_pattern: Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b")?,
            escalation_pattern: Regex::new(r"(?i)\b(hack|exploit|bypass|jailbreak)\b")?,
        })
    }

    pub fn descriptor(&self) -> &PolicyDescriptor {
        &self.descriptor
    }

    pub fn evaluate(&self, prompt: &str) -> PolicyResult {
        if self.ssn_pattern.is_match(prompt) {
            return PolicyResult::block("PII_DETECTED: SSN pattern found".to_owned());
        }
        if self.card_pattern.is_match(prompt) {
            return PolicyResult::block("PII_DETECTED: payment-card pattern found".to_owned());
        }
        if let Some(found) = self.escalation_pattern.find(prompt) {
            return PolicyResult::escalate(format!(
                "SUSPICIOUS_CONTENT: '{}' detected",
                found.as_str().to_lowercase()
            ));
        }
        PolicyResult::allow()
    }
}
