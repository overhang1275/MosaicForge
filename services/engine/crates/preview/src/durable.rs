//! Immutable bounded worker journals. No caller-controlled filesystem names.
mod artifacts;
use crate::contract::{BlockRequest, BlockResponse};
pub use artifacts::ArtifactFile;
use mosaicforge_core::template::import_svg;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

const MAX_RECORD_BYTES: u64 = 4 * 1024 * 1024;
#[derive(Clone, Copy, Debug)]
pub struct JournalLimits {
    pub bytes: u64,
    pub files: usize,
    pub time: Duration,
}
impl Default for JournalLimits {
    fn default() -> Self {
        Self {
            bytes: 256 * 1024 * 1024,
            files: 4096,
            time: Duration::from_secs(30),
        }
    }
}
pub struct Journal {
    root: PathBuf,
    limits: JournalLimits,
    lock: Mutex<()>,
    publication: Mutex<()>,
    _owner: File,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    response: Value,
    checksum: String,
}
#[derive(Debug)]
pub struct BlockError {
    pub status: u16,
    pub code: &'static str,
}
fn err(status: u16, code: &'static str) -> BlockError {
    BlockError { status, code }
}
fn disk(_: impl std::fmt::Debug) -> BlockError {
    err(500, "journal_error")
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn canonical(value: &impl Serialize) -> Result<Vec<u8>, BlockError> {
    serde_json::to_vec(&serde_json::to_value(value).map_err(disk)?).map_err(disk)
}
fn uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.hyphenated().to_string() == value)
}
impl Journal {
    pub fn open(root: &Path, limits: JournalLimits) -> std::io::Result<Self> {
        if root.as_os_str().is_empty()
            || limits.bytes == 0
            || limits.files == 0
            || limits.time.is_zero()
        {
            return Err(std::io::Error::other("invalid journal configuration"));
        }
        let root = std::path::absolute(root)?;
        // Every existing ancestor is checked, so the configured private root cannot traverse a symlink.
        for p in root.ancestors() {
            if let Ok(m) = fs::symlink_metadata(p)
                && m.file_type().is_symlink()
            {
                return Err(std::io::Error::other("journal symlink"));
            }
        }
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        let existed = root.exists();
        builder.create(&root)?;
        if !existed {
            for parent in root.ancestors().skip(1) {
                sync_directory(parent)?;
            }
        }
        let metadata = fs::symlink_metadata(&root)?;
        if !metadata.is_dir() {
            return Err(std::io::Error::other("journal is not directory"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(std::io::Error::other("journal must be private (0700)"));
            }
        }
        // Fail at startup on platforms/filesystems unable to provide directory durability.
        let owner = File::open(&root)?;
        owner.try_lock().map_err(std::io::Error::other)?;
        owner.sync_all()?;
        let journal = Self {
            root,
            limits,
            lock: Mutex::new(()),
            publication: Mutex::new(()),
            _owner: owner,
        };
        journal
            .recover_artifact_temporaries()
            .map_err(|_| std::io::Error::other("artifact recovery failed"))?;
        Ok(journal)
    }
    pub fn execute(&self, bytes: &[u8]) -> Result<Vec<u8>, BlockError> {
        let _guard = self.lock.lock().map_err(disk)?;
        let started = Instant::now();
        let request: BlockRequest =
            serde_json::from_slice(bytes).map_err(|_| err(400, "invalid_request"))?;
        let value = serde_json::to_value(&request).map_err(disk)?;
        let job_id = value["jobId"]
            .as_str()
            .ok_or_else(|| err(400, "invalid_request"))?;
        let block_id = value["blockId"]
            .as_str()
            .ok_or_else(|| err(400, "invalid_request"))?;
        let count = value["candidateCount"]
            .as_u64()
            .filter(|v| (1..=10000).contains(v))
            .ok_or_else(|| err(400, "invalid_request"))?;
        let limit = value["acceptedLimit"]
            .as_u64()
            .filter(|v| (1..=10000).contains(v))
            .ok_or_else(|| err(400, "invalid_request"))?;
        let start = value["startIndex"]
            .as_str()
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| err(400, "invalid_request"))?;
        if !uuid(job_id)
            || !uuid(block_id)
            || start.checked_add(count - 1).is_none()
            || request.job.generation.count.get() != 1
            || request.template_svg.is_empty()
            || request.template_svg.len() > crate::validation::MAX_TEMPLATE_BYTES
            || canonical(&request.job)?.len() > crate::validation::MAX_SPEC_BYTES
        {
            return Err(err(400, "invalid_request"));
        }
        mosaicforge_core::pdf::validate_pdf_job(&request.job)
            .map_err(|_| err(400, "invalid_request"))?;
        let request_hash = hash(bytes);
        if self.is_cancelled(job_id) {
            return Err(err(409, "job_cancelled"));
        }
        let path = self.root.join(format!("{block_id}.json"));
        if fs::symlink_metadata(&path).is_ok() {
            return self.replay(&path, job_id, block_id, &request_hash);
        }
        let template = import_svg(request.template_svg.as_bytes())
            .map_err(|_| err(400, "invalid_template"))?;
        if template.hash.as_str() != request.job.generation.template_hash.as_str() {
            return Err(err(400, "template_hash_mismatch"));
        }
        self.quota(0)?;
        let mut records = Vec::with_capacity(count as usize);
        let mut accepted = 0u64;
        for offset in 0..count {
            if self.is_cancelled(job_id) {
                return Err(err(409, "job_cancelled"));
            }
            if started.elapsed() > self.limits.time {
                return Err(err(503, "block_timeout"));
            }
            let index = start + offset;
            let mut job = request.job.clone();
            job.generation.start_index = index.to_string().parse().map_err(disk)?;
            let exported =
                mosaicforge_cpu::export::generate_and_export(&template, &job.generation, index, 0)
                    .map_err(|_| err(500, "compute_error"))?;
            let reason = if let Some(svg) = &exported.artifact {
                let physical = mosaicforge_cpu::pdf::physical_candidate_rejection(
                    &template,
                    &job,
                    &exported.variant,
                )
                .map_err(|_| err(500, "compute_error"))?;
                if physical.is_none() {
                    mosaicforge_cpu::pdf::geometry::optimize_page(svg, &job)
                        .map_err(|_| err(500, "compute_error"))?;
                }
                physical.map(str::to_owned)
            } else {
                Some(
                    exported
                        .variant
                        .result
                        .reason
                        .as_ref()
                        .and_then(|r| serde_json::to_value(r).ok())
                        .and_then(|r| r.as_str().map(str::to_owned))
                        .unwrap_or_else(|| "generation_rejected".into()),
                )
            };
            let ok = reason.is_none();
            if ok {
                accepted += 1;
            }
            records.push(json!({"index":index.to_string(),"accepted":ok,"reason":reason}));
            if accepted == limit {
                break;
            }
        }
        if started.elapsed() > self.limits.time {
            return Err(err(503, "block_timeout"));
        }
        let evaluated = records.len() as u64;
        let response = json!({"schemaVersion":"1","jobId":job_id,"blockId":block_id,"requestHash":request_hash,"startIndex":start.to_string(),"evaluatedCount":evaluated,"acceptedCount":accepted,"rejectedCount":evaluated-accepted,"nextCandidateIndex":(start+(evaluated-1)).checked_add(1).map(|v|v.to_string()),"recordsId":uuid::Uuid::new_v4().to_string(),"recordsSha256":hash(&canonical(&records)?),"records":records});
        let _: BlockResponse = serde_json::from_value(response.clone()).map_err(disk)?;
        let stored = canonical(&Stored {
            checksum: hash(&canonical(&response)?),
            response,
        })?;
        if stored.len() as u64 > MAX_RECORD_BYTES {
            return Err(err(507, "journal_quota"));
        }
        self.quota(stored.len() as u64)?;
        let temporary = self.root.join(format!("{}.tmp", uuid::Uuid::new_v4()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let result = (|| {
            let mut file = options.open(&temporary).map_err(disk)?;
            file.write_all(&stored).map_err(disk)?;
            file.sync_all().map_err(disk)?;
            drop(file);
            // link is atomic and exclusive; unlike rename it cannot replace committed data.
            let _publication = self.publication.lock().map_err(disk)?;
            if self.is_cancelled(job_id) {
                return Err(err(409, "job_cancelled"));
            }
            match fs::hard_link(&temporary, &path) {
                Ok(()) => (),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    return self.replay(&path, job_id, block_id, &request_hash);
                }
                Err(e) => return Err(disk(e)),
            }
            sync_directory(&self.root).map_err(disk)?;
            self.replay(&path, job_id, block_id, &request_hash)
        })();
        let _ = fs::remove_file(temporary);
        sync_directory(&self.root).map_err(disk)?;
        result
    }
    fn quota(&self, additional: u64) -> Result<(), BlockError> {
        let mut bytes = additional;
        let mut count = 0;
        for entry in fs::read_dir(&self.root).map_err(disk)? {
            let entry = entry.map_err(disk)?;
            if entry.file_name() == "artifacts" {
                let m = fs::symlink_metadata(entry.path()).map_err(disk)?;
                if !m.is_dir() || m.file_type().is_symlink() {
                    return Err(err(500, "journal_error"));
                }
                continue;
            }
            count += 1;
            if count >= self.limits.files {
                return Err(err(507, "journal_quota"));
            }
            let metadata = fs::symlink_metadata(entry.path()).map_err(disk)?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(err(500, "journal_error"));
            }
            bytes = bytes
                .checked_add(metadata.len())
                .ok_or_else(|| err(507, "journal_quota"))?;
            if bytes > self.limits.bytes {
                return Err(err(507, "journal_quota"));
            }
        }
        // Reserve both temporary and final directory entries during exclusive publication.
        if bytes > self.limits.bytes
            || (additional > 0 && count.saturating_add(2) > self.limits.files)
        {
            return Err(err(507, "journal_quota"));
        }
        Ok(())
    }
    fn replay(
        &self,
        path: &Path,
        job_id: &str,
        block_id: &str,
        request_hash: &str,
    ) -> Result<Vec<u8>, BlockError> {
        let metadata = fs::symlink_metadata(path).map_err(disk)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_RECORD_BYTES
        {
            return Err(err(500, "journal_corrupt"));
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(disk)?
            .take(MAX_RECORD_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(disk)?;
        let stored: Stored =
            serde_json::from_slice(&bytes).map_err(|_| err(500, "journal_corrupt"))?;
        if hash(&canonical(&stored.response)?) != stored.checksum {
            return Err(err(500, "journal_corrupt"));
        }
        let r = &stored.response;
        let _: BlockResponse =
            serde_json::from_value(r.clone()).map_err(|_| err(500, "journal_corrupt"))?;
        if r["requestHash"] != request_hash {
            return Err(err(409, "block_conflict"));
        }
        if r["jobId"] != job_id || r["blockId"] != block_id {
            return Err(err(500, "journal_corrupt"));
        }
        if hash(&canonical(&r["records"])?) != r["recordsSha256"].as_str().unwrap_or("") {
            return Err(err(500, "journal_corrupt"));
        }
        // A replay crosses a fresh directory barrier as publication may have survived an interrupted fsync.
        sync_directory(&self.root).map_err(disk)?;
        canonical(r)
    }
}
fn sync_directory(path: &Path) -> std::io::Result<()> {
    File::open(path)?.sync_all()
}
