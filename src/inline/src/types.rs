use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub model_id: String,
    pub input_data: String,
    pub context: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Decision {
    Allowed,
    Blocked,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub decision: Decision,
    pub rules_triggered: Vec<String>,
    pub confidence_score: f64,
}

impl PolicyResult {
    pub fn allow() -> Self {
        Self {
            decision: Decision::Allowed,
            rules_triggered: vec![],
            confidence_score: 1.0,
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
            confidence_score: f64::min(self.confidence_score, other.confidence_score),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRecord {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub policy_result: PolicyResult,
    pub request_hash: String,
    pub shard_id: usize,
    pub latency_ms: f64,
}
