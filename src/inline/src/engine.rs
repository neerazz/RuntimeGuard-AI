use crate::types::{ComplianceRecord, InferenceRequest, PolicyResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{error, info};
use chrono::Utc;

// Mock DiskBuffer for fallback
pub struct DiskBuffer {
    path: String,
}

impl DiskBuffer {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub async fn append(&mut self, record: ComplianceRecord) -> Result<()> {
        // In a real impl, this would write to disk
        info!("fallback_append record_id={}", record.request_id);
        Ok(())
    }
}

pub struct InlinePolicyEngine {
    shard_queues: Vec<mpsc::Sender<ComplianceRecord>>,
    fallback_buffer: Arc<Mutex<DiskBuffer>>,
    num_shards: usize,
}

impl InlinePolicyEngine {
    pub fn new(num_shards: usize, queue_capacity: usize) -> Self {
        let mut queues = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            let (tx, mut rx) = mpsc::channel(queue_capacity);
            queues.push(tx);
            
            // Mock consumer to drain queue
            tokio::spawn(async move {
                while let Some(record) = rx.recv().await {
                    // In real impl, this would batch and send to ZK prover
                    // For now, we just drain
                }
            });
        }

        Self {
            shard_queues: queues,
            fallback_buffer: Arc::new(Mutex::new(DiskBuffer::new("fallback.log".to_string()))),
            num_shards,
        }
    }

    fn compute_shard(&self, request_id: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request_id.hash(&mut hasher);
        (hasher.finish() as usize) % self.num_shards
    }

    pub async fn evaluate(&self, req: &InferenceRequest) -> PolicyResult {
        let start = std::time::Instant::now();

        // 1. Evaluate Rules (Mock logic for now, as strict rule impl wasn't requested in detail, 
        //    but the architecture was the focus)
        //    In a real system, we'd iterate over self.rules here.
        let result = PolicyResult::allow(); 

        // 2. Log Append with Backpressure Handling (Theorem 1)
        let shard_id = self.compute_shard(&req.id);
        
        let record = ComplianceRecord {
            request_id: req.id.clone(),
            timestamp: Utc::now(),
            policy_result: result.clone(),
            request_hash: "mock_hash".to_string(),
            shard_id,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        };

        // CRITICAL: Non-blocking send
        let queue = &self.shard_queues[shard_id];
        match queue.try_send(record.clone()) {
            Ok(_) => {
                // Successfully enqueued to shard
            },
            Err(mpsc::error::TrySendError::Full(_)) => {
                // Theorem 1 Violation Prevention:
                // Instead of blocking or dropping, we spawn a background task to write to disk.
                // This ensures the inline path returns immediately.
                let fallback = self.fallback_buffer.clone();
                let rec = record.clone();
                tokio::spawn(async move {
                    let mut guard = fallback.lock().await;
                    if let Err(e) = guard.append(rec).await {
                        error!("Failed to write to fallback buffer: {}", e);
                    }
                });
                info!("Backpressure detected, offloaded to fallback buffer");
            },
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("Shard queue closed!");
                 let fallback = self.fallback_buffer.clone();
                let rec = record.clone();
                tokio::spawn(async move {
                    let _ = fallback.lock().await.append(rec).await;
                });
            }
        }

        result
    }
}
