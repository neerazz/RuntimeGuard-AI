use crate::types::ComplianceRecord;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"RGL2";
const HEADER_LEN: usize = 8;
const DIGEST_LEN: usize = 32;
const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncPolicy {
    None,
    Data,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendReceipt {
    pub sequence: u64,
    pub offset: u64,
    pub record_commitment: [u8; 32],
    pub durable: bool,
}

/// Single-writer, append-only framed log for compliance records.
///
/// Each frame contains a magic value, payload length, serialized record, and
/// SHA-256 payload checksum. Reopening truncates only an incomplete tail frame;
/// a complete frame with an invalid checksum is treated as corruption.
pub struct DurableLog {
    path: PathBuf,
    file: File,
    sync_policy: SyncPolicy,
}

impl DurableLog {
    pub fn open(path: impl AsRef<Path>, sync_policy: SyncPolicy) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let existed = path.exists();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("open durable log {}", path.display()))?;
        if !existed {
            sync_parent_directory(&path)?;
        }

        recover_frames(&mut file, true)
            .with_context(|| format!("recover durable log {}", path.display()))?;
        file.seek(SeekFrom::End(0))?;

        Ok(Self {
            path,
            file,
            sync_policy,
        })
    }

    pub fn append(&mut self, record: &ComplianceRecord) -> Result<AppendReceipt> {
        let payload = serde_json::to_vec(record).context("serialize compliance record")?;
        if payload.len() > MAX_RECORD_BYTES {
            bail!("record exceeds {MAX_RECORD_BYTES} byte limit");
        }

        let offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(MAGIC)?;
        self.file.write_all(&(payload.len() as u32).to_be_bytes())?;
        self.file.write_all(&payload)?;
        self.file.write_all(&Sha256::digest(&payload))?;

        match self.sync_policy {
            SyncPolicy::None => {}
            SyncPolicy::Data => self.file.sync_data()?,
            SyncPolicy::Full => self.file.sync_all()?,
        }

        Ok(AppendReceipt {
            sequence: record.sequence,
            offset,
            record_commitment: record.commitment(),
            durable: self.sync_policy != SyncPolicy::None,
        })
    }

    pub fn records(&self) -> Result<Vec<ComplianceRecord>> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .with_context(|| format!("read durable log {}", self.path.display()))?;
        recover_frames(&mut file, false)
    }
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("log path has no parent: {}", path.display()))?;
    File::open(parent)
        .with_context(|| format!("open log directory {}", parent.display()))?
        .sync_all()
        .with_context(|| format!("synchronize log directory {}", parent.display()))
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> Result<()> {
    Ok(())
}

fn recover_frames(file: &mut File, truncate_partial_tail: bool) -> Result<Vec<ComplianceRecord>> {
    file.seek(SeekFrom::Start(0))?;
    let mut records = Vec::new();
    let mut offset = 0_u64;

    loop {
        let mut header = [0_u8; HEADER_LEN];
        let header_bytes = read_up_to(file, &mut header)?;
        if header_bytes == 0 {
            break;
        }
        if header_bytes < HEADER_LEN {
            truncate_tail(file, offset, truncate_partial_tail)?;
            break;
        }
        if &header[..4] != MAGIC {
            bail!("invalid frame magic at byte offset {offset}");
        }

        let payload_len =
            u32::from_be_bytes(header[4..8].try_into().expect("fixed header")) as usize;
        if payload_len > MAX_RECORD_BYTES {
            bail!("frame at byte offset {offset} exceeds size limit");
        }

        let mut payload = vec![0_u8; payload_len];
        if read_up_to(file, &mut payload)? < payload_len {
            truncate_tail(file, offset, truncate_partial_tail)?;
            break;
        }
        let mut expected_digest = [0_u8; DIGEST_LEN];
        if read_up_to(file, &mut expected_digest)? < DIGEST_LEN {
            truncate_tail(file, offset, truncate_partial_tail)?;
            break;
        }

        let actual_digest: [u8; DIGEST_LEN] = Sha256::digest(&payload).into();
        if actual_digest != expected_digest {
            bail!("frame checksum mismatch at byte offset {offset}");
        }
        let record: ComplianceRecord = serde_json::from_slice(&payload)
            .with_context(|| format!("decode frame at byte offset {offset}"))?;
        records.push(record);
        offset += (HEADER_LEN + payload_len + DIGEST_LEN) as u64;
    }

    file.seek(SeekFrom::End(0))?;
    Ok(records)
}

fn read_up_to(file: &mut File, buffer: &mut [u8]) -> Result<usize> {
    let mut read = 0;
    while read < buffer.len() {
        match file.read(&mut buffer[read..]) {
            Ok(0) => break,
            Ok(count) => read += count,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(read)
}

fn truncate_tail(file: &mut File, offset: u64, truncate: bool) -> Result<()> {
    if !truncate {
        bail!("incomplete frame at byte offset {offset}");
    }
    file.set_len(offset)?;
    file.sync_data()?;
    Ok(())
}
