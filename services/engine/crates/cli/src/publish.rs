#[cfg(test)]
use std::io::Write;
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
struct Temporary(PathBuf);
// Declaration order closes the handle before Temporary removes its name on Windows.
pub struct PendingFile {
    file: File,
    temporary: Temporary,
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Publishes complete bytes exclusively. Both names stay on the same filesystem.
/// A failed job may retain previously published complete artifacts.
pub fn publish_with(
    target: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    let directory = target
        .parent()
        .ok_or_else(|| io::Error::other("missing output directory"))?;
    let mut pending = PendingFile::create(directory)?;
    write(pending.writer())?;
    pending.publish(target)
}
impl PendingFile {
    pub fn create(directory: &Path) -> io::Result<Self> {
        (0..128)
            .find_map(|_| {
                let path = directory.join(format!(
                    ".mosaicforge-{}-{}.tmp",
                    std::process::id(),
                    NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
                ));
                match OpenOptions::new().write(true).create_new(true).open(&path) {
                    Ok(file) => Some(Ok(Self {
                        file,
                        temporary: Temporary(path),
                    })),
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => None,
                    Err(e) => Some(Err(e)),
                }
            })
            .unwrap_or_else(|| Err(io::Error::other("cannot create exclusive temporary")))
    }
    pub fn writer(&mut self) -> &mut File {
        &mut self.file
    }
    pub fn publish(self, target: &Path) -> io::Result<()> {
        let Self { file, temporary } = self;
        let synced = file.sync_all();
        drop(file);
        synced?;
        fs::hard_link(&temporary.0, target)?;
        fs::remove_file(&temporary.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    static TEST_ID: AtomicU64 = AtomicU64::new(0);
    fn directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "mosaicforge-publish-{}-{}",
            std::process::id(),
            TEST_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn partial_write_failure_never_publishes_or_leaves_a_temporary() {
        let dir = directory();
        let result = publish_with(&dir.join("final.svg"), |file| {
            file.write_all(b"incomplete")?;
            Err(io::Error::other("injected write failure"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn racing_publishers_have_exactly_one_complete_winner() {
        let dir = directory();
        let target = dir.join("final.svg");
        let barrier = Arc::new(Barrier::new(2));
        let outcomes = std::thread::scope(|scope| {
            let tasks: Vec<_> = [
                b"first complete file".as_slice(),
                b"second complete file".as_slice(),
            ]
            .into_iter()
            .map(|bytes| {
                let target = &target;
                let barrier = &barrier;
                scope.spawn(move || {
                    publish_with(target, |file| {
                        file.write_all(bytes)?;
                        barrier.wait();
                        Ok(())
                    })
                })
            })
            .collect();
            tasks
                .into_iter()
                .map(|task| task.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(outcomes.iter().filter(|r| r.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|r| r
                    .as_ref()
                    .is_err_and(|e| e.kind() == io::ErrorKind::AlreadyExists))
                .count(),
            1
        );
        assert!(
            [
                b"first complete file".as_slice(),
                b"second complete file".as_slice()
            ]
            .contains(&fs::read(&target).unwrap().as_slice())
        );
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_file(target).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
