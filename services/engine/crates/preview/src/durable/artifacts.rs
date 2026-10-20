use super::*;
use mosaicforge_core::EngineError;
use std::io::{self, Seek};
const QUOTA: u64 = 1024 * 1024 * 1024;
pub struct ArtifactFile {
    pub file: File,
    pub length: u64,
    pub hash: String,
    pub kind: String,
    pub job_id: String,
}
fn directory(path: &Path) -> Result<(), BlockError> {
    if let Ok(m) = fs::symlink_metadata(path) {
        if !m.is_dir() || m.file_type().is_symlink() {
            return Err(err(500, "artifact_corrupt"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if m.permissions().mode() & 0o077 != 0 {
                return Err(err(500, "artifact_corrupt"));
            }
        }
        return Ok(());
    }
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    match b.create(path) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => return directory(path),
        Err(e) => return Err(disk(e)),
    };
    sync_directory(path.parent().unwrap()).map_err(disk)
}
fn create(path: &Path) -> Result<File, BlockError> {
    let mut o = OpenOptions::new();
    o.write(true).read(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    o.open(path).map_err(disk)
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>, BlockError> {
    let m = fs::symlink_metadata(path).map_err(disk)?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > limit {
        return Err(err(500, "artifact_corrupt"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if m.permissions().mode() & 0o077 != 0 {
            return Err(err(500, "artifact_corrupt"));
        }
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(disk)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(disk)?;
    if bytes.len() as u64 != m.len() {
        return Err(err(500, "artifact_corrupt"));
    }
    Ok(bytes)
}
fn file_hash(path: &Path) -> Result<(File, u64, String), BlockError> {
    let m = fs::symlink_metadata(path).map_err(disk)?;
    if !m.is_file() || m.file_type().is_symlink() || m.len() > QUOTA {
        return Err(err(500, "artifact_corrupt"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if m.permissions().mode() & 0o077 != 0 {
            return Err(err(500, "artifact_corrupt"));
        }
    }
    let mut f = File::open(path).map_err(disk)?;
    let mut h = Sha256::new();
    let mut n = 0;
    let mut b = [0; 65536];
    loop {
        let c = f.read(&mut b).map_err(disk)?;
        if c == 0 {
            break;
        }
        n += c as u64;
        if n > QUOTA {
            return Err(err(507, "artifact_quota"));
        }
        h.update(&b[..c]);
    }
    if n != m.len() {
        return Err(err(500, "artifact_corrupt"));
    }
    f.rewind().map_err(disk)?;
    Ok((f, n, format!("{:x}", h.finalize())))
}
struct Limited {
    file: File,
    remaining: u64,
    exceeded: bool,
}
impl Write for Limited {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if b.len() as u64 > self.remaining {
            self.exceeded = true;
            return Err(io::Error::other("artifact quota"));
        }
        let n = self.file.write(b)?;
        self.remaining -= n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
struct Selection<'a> {
    journal: &'a Journal,
    refs: std::vec::IntoIter<Value>,
    records: std::vec::IntoIter<Value>,
    job: String,
    remaining: u64,
    next: Option<u64>,
}
impl Iterator for Selection<'_> {
    type Item = Result<u64, EngineError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let result = (|| -> Result<Option<u64>, BlockError> {
            loop {
                if let Some(r) = self.records.next() {
                    let i = r["index"]
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                        .ok_or_else(|| err(500, "journal_corrupt"))?;
                    if self.next != Some(i) {
                        return Err(err(500, "journal_corrupt"));
                    }
                    self.next = i.checked_add(1);
                    if r["accepted"] == true {
                        self.remaining -= 1;
                        return Ok(Some(i));
                    }
                    continue;
                }
                let Some(reference) = self.refs.next() else {
                    return Err(err(409, "short_selection"));
                };
                let id = reference["blockId"]
                    .as_str()
                    .filter(|s| uuid(s))
                    .ok_or_else(|| err(400, "invalid_request"))?;
                let rh = reference["requestHash"]
                    .as_str()
                    .ok_or_else(|| err(400, "invalid_request"))?;
                let bytes = self.journal.replay(
                    &self.journal.root.join(format!("{id}.json")),
                    &self.job,
                    id,
                    rh,
                )?;
                let v: Value = serde_json::from_slice(&bytes).map_err(disk)?;
                if v["recordsSha256"] != reference["recordsSha256"] {
                    return Err(err(500, "journal_corrupt"));
                }
                let records = v["records"]
                    .as_array()
                    .filter(|r| !r.is_empty() && r.len() <= 10000)
                    .ok_or_else(|| err(500, "journal_corrupt"))?;
                if self.next.is_none() {
                    self.next = v["startIndex"].as_str().and_then(|s| s.parse().ok());
                }
                if v["evaluatedCount"].as_u64() != Some(records.len() as u64) {
                    return Err(err(500, "journal_corrupt"));
                }
                self.records = records.clone().into_iter();
            }
        })();
        match result {
            Ok(Some(i)) => Some(Ok(i)),
            Ok(None) => None,
            Err(_) => {
                self.remaining = 0;
                Some(Err(EngineError::InvalidSpec("invalid committed selection")))
            }
        }
    }
}
impl Journal {
    /// Startup only: the OS owner lock is held and no worker can be active.
    pub(super) fn recover_artifact_temporaries(&self) -> Result<(), BlockError> {
        let root = self.root.join("artifacts");
        match fs::symlink_metadata(&root) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(disk(e)),
            Ok(_) => (),
        }
        directory(&root)?;
        for entry in fs::read_dir(&root).map_err(disk)? {
            let entry = entry.map_err(disk)?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.strip_suffix(".tmp").is_some_and(uuid) {
                self.remove_temporary(&entry.path())?;
            }
        }
        sync_directory(&root).map_err(disk)
    }
    fn artifact_root(&self) -> Result<PathBuf, BlockError> {
        let p = self.root.join("artifacts");
        directory(&p)?;
        Ok(p)
    }
    pub fn is_cancelled(&self, job: &str) -> bool {
        fs::symlink_metadata(self.root.join("artifacts").join(format!("{job}.cancel"))).is_ok()
    }
    pub fn cancel(&self, bytes: &[u8]) -> Result<Vec<u8>, BlockError> {
        let request: crate::contract::cancel_request::CancelRequest =
            serde_json::from_slice(bytes).map_err(|_| err(400, "invalid_request"))?;
        let v = serde_json::to_value(request).map_err(disk)?;
        let job = v["jobId"]
            .as_str()
            .filter(|s| uuid(s))
            .ok_or_else(|| err(400, "invalid_request"))?;
        let _g = self.publication.lock().map_err(disk)?;
        let root = self.artifact_root()?;
        let p = root.join(format!("{job}.cancel"));
        if !self.is_cancelled(job) {
            self.output_quota(&root)?;
            let f = create(&p)?;
            f.sync_all().map_err(disk)?;
        }
        sync_directory(&root).map_err(disk)?;
        canonical(&json!({"schemaVersion":"1","jobId":job,"cancelled":true}))
    }
    fn output_quota(&self, root: &Path) -> Result<u64, BlockError> {
        fn size(p: &Path) -> Result<u64, BlockError> {
            let m = fs::symlink_metadata(p).map_err(disk)?;
            if m.file_type().is_symlink() {
                return Err(err(500, "artifact_corrupt"));
            }
            if m.is_file() {
                return Ok(m.len());
            }
            let mut n = 0u64;
            for e in fs::read_dir(p).map_err(disk)? {
                n = n
                    .checked_add(size(&e.map_err(disk)?.path())?)
                    .ok_or_else(|| err(507, "artifact_quota"))?;
                if n > QUOTA {
                    return Err(err(507, "artifact_quota"));
                }
            }
            Ok(n)
        }
        let used = size(root)?;
        QUOTA
            .checked_sub(used)
            .ok_or_else(|| err(507, "artifact_quota"))
    }
    pub fn finalize(&self, bytes: &[u8]) -> Result<Vec<u8>, BlockError> {
        let request: crate::contract::finalize_request::FinalizeRequest =
            serde_json::from_slice(bytes).map_err(|_| err(400, "invalid_request"))?;
        let v = serde_json::to_value(&request).map_err(disk)?;
        let job = v["jobId"]
            .as_str()
            .filter(|s| uuid(s))
            .ok_or_else(|| err(400, "invalid_request"))?;
        let id = v["artifactId"]
            .as_str()
            .filter(|s| uuid(s))
            .ok_or_else(|| err(400, "invalid_request"))?;
        let count = v["requestedAcceptedCount"]
            .as_u64()
            .filter(|n| (1..=1000000).contains(n))
            .ok_or_else(|| err(400, "invalid_request"))?;
        let refs = v["blocks"]
            .as_array()
            .filter(|r| !r.is_empty() && r.len() <= 4095)
            .ok_or_else(|| err(400, "invalid_request"))?;
        if request.job.generation.count.get() != 1
            || request.template_svg.len() > crate::validation::MAX_TEMPLATE_BYTES
            || canonical(&request.job)?.len() > crate::validation::MAX_SPEC_BYTES
        {
            return Err(err(400, "invalid_request"));
        }
        let _work = self.lock.lock().map_err(disk)?;
        let root = self.artifact_root()?;
        let sealed = root.join(id);
        let rh = hash(bytes);
        if self.is_cancelled(job) {
            return Err(err(409, "job_cancelled"));
        }
        if fs::symlink_metadata(&sealed).is_ok() {
            return self.finalize_replay(&sealed, job, &rh);
        }
        let start = request
            .job
            .generation
            .start_index
            .parse::<u64>()
            .map_err(|_| err(400, "invalid_request"))?;
        let sh = self.validate_references(job, refs, count, start)?;
        let config = hash(&canonical(
            &json!({"schemaVersion":"1","job":request.job,"requestedAcceptedCount":count,"selectionSha256":sh}),
        )?);
        let temp = root.join(format!("{id}.tmp"));
        if fs::symlink_metadata(&temp).is_ok() {
            self.remove_temporary(&temp)?;
        }
        directory(&temp)?;
        let result = (|| {
            let available = self.output_quota(&root)?;
            // Keep a bounded reserve for the manifest and metadata alongside the PDF.
            let reserve = count
                .checked_mul(100)
                .and_then(|n| n.checked_add(16384))
                .ok_or_else(|| err(507, "artifact_quota"))?;
            let limit = available
                .checked_sub(reserve)
                .ok_or_else(|| err(507, "artifact_quota"))?;
            let template = import_svg(request.template_svg.as_bytes())
                .map_err(|_| err(400, "invalid_template"))?;
            if template.hash.as_str() != request.job.generation.template_hash.as_str() {
                return Err(err(400, "template_hash_mismatch"));
            }
            let selection = Selection {
                journal: self,
                refs: refs.clone().into_iter(),
                records: Vec::new().into_iter(),
                job: job.into(),
                remaining: count,
                next: None,
            };
            // The verified first pass hashes indices; the second pass renders and writes records.
            let mut manifest = create(&temp.join("manifest.ndjson"))?;
            writeln!(manifest,"{}",json!({"schemaVersion":"1","kind":"job","jobId":job,"artifactId":id,"documentConfigurationHash":config,"requestedAcceptedCount":count,"seed":request.job.generation.seed,"engineVersion":"0.1.0","rulesVersion":request.job.generation.rules_version,"backend":"cpu"})).map_err(disk)?;
            let mut output = Limited {
                file: create(&temp.join("document.pdf"))?,
                remaining: limit,
                exceeded: false,
            };
            let digest = mosaicforge_cpu::pdf::write_selected_pdf_stream(
                &template,
                &request.job,
                count,
                selection,
                &mut output,
                || self.is_cancelled(job),
                |index, page| {
                    writeln!(
                        manifest,
                        "{}",
                        json!({"kind":"variant","index":index.to_string(),"pageNumber":page})
                    )
                    .map_err(|_| EngineError::RenderFailed("manifest write"))
                },
            )
            .map_err(|e| {
                if self.is_cancelled(job) {
                    err(409, "job_cancelled")
                } else if output.exceeded {
                    err(507, "artifact_quota")
                } else if matches!(e, EngineError::InvalidSpec(_)) {
                    err(409, "finalization_failed")
                } else {
                    err(500, "finalization_failed")
                }
            })?;
            output.file.sync_all().map_err(disk)?;
            writeln!(
                manifest,
                "{}",
                json!({"kind":"completed","pageCount":count,"sha256":digest,"selectionSha256":sh})
            )
            .map_err(disk)?;
            manifest.sync_all().map_err(disk)?;
            let (_, pdf_bytes, pdf_hash) = file_hash(&temp.join("document.pdf"))?;
            let (_, manifest_bytes, manifest_hash) = file_hash(&temp.join("manifest.ndjson"))?;
            let response = json!({"schemaVersion":"1","jobId":job,"artifactId":id,"requestHash":rh,"documentConfigurationHash":config,"pageCount":count,"sha256":pdf_hash,"bytes":pdf_bytes.to_string(),"manifestSha256":manifest_hash,"manifestBytes":manifest_bytes.to_string(),"selectionSha256":sh});
            let _: crate::contract::finalize_response::FinalizeResponse =
                serde_json::from_value(response.clone()).map_err(disk)?;
            let encoded = canonical(&response)?;
            let mut metadata = create(&temp.join("metadata.json"))?;
            metadata.write_all(&encoded).map_err(disk)?;
            metadata.sync_all().map_err(disk)?;
            sync_directory(&temp).map_err(disk)?;
            self.output_quota(&root)?;
            let _publication = self.publication.lock().map_err(disk)?;
            if self.is_cancelled(job) {
                return Err(err(409, "job_cancelled"));
            }
            fs::rename(&temp, &sealed).map_err(disk)?;
            sync_directory(&root).map_err(disk)?;
            Ok(encoded)
        })();
        if result.is_err() && temp.exists() {
            let _ = self.remove_temporary(&temp);
        }
        result
    }
    fn remove_temporary(&self, p: &Path) -> Result<(), BlockError> {
        directory(p)?;
        for e in fs::read_dir(p).map_err(disk)? {
            let e = e.map_err(disk)?;
            let n = e.file_name();
            if ![
                "document.pdf",
                "manifest.ndjson",
                "variants.tmp",
                "metadata.json",
            ]
            .iter()
            .any(|s| n == *s)
            {
                return Err(err(500, "artifact_corrupt"));
            }
            let m = fs::symlink_metadata(e.path()).map_err(disk)?;
            if !m.is_file() || m.file_type().is_symlink() {
                return Err(err(500, "artifact_corrupt"));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if m.permissions().mode() & 0o077 != 0 {
                    return Err(err(500, "artifact_corrupt"));
                }
            }
            fs::remove_file(e.path()).map_err(disk)?;
        }
        fs::remove_dir(p).map_err(disk)
    }
    fn validate_references(
        &self,
        job: &str,
        refs: &[Value],
        count: u64,
        start: u64,
    ) -> Result<String, BlockError> {
        let mut selection_hash = Sha256::new();
        selection_hash.update(b"mosaicforge-selection-v1\n");
        let mut next = Some(start);
        let mut accepted = 0u64;
        for reference in refs {
            if self.is_cancelled(job) {
                return Err(err(409, "job_cancelled"));
            }
            let id = reference["blockId"]
                .as_str()
                .filter(|s| uuid(s))
                .ok_or_else(|| err(400, "invalid_request"))?;
            let rh = reference["requestHash"]
                .as_str()
                .ok_or_else(|| err(400, "invalid_request"))?;
            let v: Value = serde_json::from_slice(&self.replay(
                &self.root.join(format!("{id}.json")),
                job,
                id,
                rh,
            )?)
            .map_err(disk)?;
            if v["recordsSha256"] != reference["recordsSha256"] {
                return Err(err(500, "journal_corrupt"));
            }
            let records = v["records"]
                .as_array()
                .filter(|r| !r.is_empty() && r.len() <= 10000)
                .ok_or_else(|| err(500, "journal_corrupt"))?;
            let start = v["startIndex"]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| err(500, "journal_corrupt"))?;
            if next != Some(start) {
                return Err(err(500, "journal_corrupt"));
            }
            let mut local = 0u64;
            for r in records {
                let index = r["index"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or_else(|| err(500, "journal_corrupt"))?;
                if next != Some(index) {
                    return Err(err(500, "journal_corrupt"));
                }
                next = index.checked_add(1);
                if r["accepted"] == true {
                    if accepted + local < count {
                        selection_hash.update(format!("{index}\n"));
                    }
                    local += 1;
                }
            }
            if v["evaluatedCount"].as_u64() != Some(records.len() as u64)
                || v["acceptedCount"].as_u64() != Some(local)
                || v["rejectedCount"].as_u64() != Some(records.len() as u64 - local)
                || v["nextCandidateIndex"]
                    != next.map(|n| json!(n.to_string())).unwrap_or(Value::Null)
            {
                return Err(err(500, "journal_corrupt"));
            }
            accepted += local;
        }
        if accepted < count {
            return Err(err(409, "short_selection"));
        }
        Ok(format!("{:x}", selection_hash.finalize()))
    }
    fn finalize_replay(&self, p: &Path, job: &str, rh: &str) -> Result<Vec<u8>, BlockError> {
        directory(p)?;
        let bytes = read(&p.join("metadata.json"), 16384)?;
        let v: Value = serde_json::from_slice(&bytes).map_err(disk)?;
        let _: crate::contract::finalize_response::FinalizeResponse =
            serde_json::from_value(v.clone()).map_err(disk)?;
        if v["jobId"] != job || v["requestHash"] != rh {
            return Err(err(409, "artifact_conflict"));
        }
        self.verify_member(p, "pdf", &v)?;
        self.verify_member(p, "manifest", &v)?;
        sync_directory(p.parent().unwrap()).map_err(disk)?;
        Ok(bytes)
    }
    fn verify_member(&self, p: &Path, kind: &str, v: &Value) -> Result<ArtifactFile, BlockError> {
        let (name, size, h) = if kind == "pdf" {
            ("document.pdf", "bytes", "sha256")
        } else {
            ("manifest.ndjson", "manifestBytes", "manifestSha256")
        };
        let (file, length, hash) = file_hash(&p.join(name))?;
        if v[size].as_str().and_then(|s| s.parse::<u64>().ok()) != Some(length) || v[h] != hash {
            return Err(err(500, "artifact_corrupt"));
        }
        Ok(ArtifactFile {
            file,
            length,
            hash,
            kind: kind.into(),
            job_id: v["jobId"]
                .as_str()
                .ok_or_else(|| err(500, "artifact_corrupt"))?
                .into(),
        })
    }
    pub fn artifact(&self, id: &str, kind: &str) -> Result<ArtifactFile, BlockError> {
        if !uuid(id) || !["pdf", "manifest"].contains(&kind) {
            return Err(err(404, "artifact_not_found"));
        }
        let _g = self.publication.lock().map_err(disk)?;
        let p = self.artifact_root()?.join(id);
        if !p.exists() {
            return Err(err(404, "artifact_not_found"));
        }
        directory(&p)?;
        let bytes = read(&p.join("metadata.json"), 16384)?;
        let v: Value = serde_json::from_slice(&bytes).map_err(disk)?;
        let _: crate::contract::finalize_response::FinalizeResponse =
            serde_json::from_value(v.clone()).map_err(disk)?;
        if v["artifactId"] != id {
            return Err(err(500, "artifact_corrupt"));
        }
        let job = v["jobId"]
            .as_str()
            .filter(|s| uuid(s))
            .ok_or_else(|| err(500, "artifact_corrupt"))?;
        if self.is_cancelled(job) {
            return Err(err(404, "artifact_not_found"));
        }
        self.verify_member(&p, kind, &v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quota_writer_never_writes_the_excess_chunk() {
        let p = std::env::temp_dir().join(format!("mosaic-limited-{}", uuid::Uuid::new_v4()));
        let mut output = Limited {
            file: create(&p).unwrap(),
            remaining: 7,
            exceeded: false,
        };
        output.write_all(b"partial").unwrap();
        assert!(output.write_all(b"x").is_err());
        assert!(output.exceeded);
        assert_eq!(output.file.metadata().unwrap().len(), 7);
        drop(output);
        fs::remove_file(p).unwrap();
    }
}
