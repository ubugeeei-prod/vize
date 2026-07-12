use std::ffi::OsString;
use std::fs::{self, TryLockError};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use super::error::CorsaResult;
use vize_carton::cstr;

const LOCK_RETRY_DELAY: Duration = Duration::from_millis(25);
/// Materialization normally takes far less than this; bound contention so a
/// live but wedged owner produces an actionable error instead of a hung check.
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(any(not(windows), test))]
const LEGACY_LOCK_SUFFIX: &str = ".lock";
#[cfg(any(windows, test))]
const WINDOWS_LOCK_SUFFIX: &str = ".materialize.lock";

#[cfg(not(windows))]
const LOCK_SUFFIX: &str = LEGACY_LOCK_SUFFIX;
// Older Windows releases created `<virtual-root>.lock` as a directory. A crash
// left that directory behind forever. A distinct file name makes those stale
// directories inert without guessing whether an older process still owns one.
#[cfg(windows)]
const LOCK_SUFFIX: &str = WINDOWS_LOCK_SUFFIX;

pub(super) struct MaterializeLock {
    file: fs::File,
}

impl MaterializeLock {
    pub(super) fn acquire(virtual_root: &Path) -> CorsaResult<Self> {
        Self::acquire_with_timeout(virtual_root, LOCK_WAIT_TIMEOUT).map_err(Into::into)
    }

    fn acquire_with_timeout(virtual_root: &Path, timeout: Duration) -> io::Result<Self> {
        let path = lock_path_for(virtual_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .truncate(false)
            .write(true)
            .open(&path)?;
        let started = Instant::now();
        wait_for_lock(
            &path,
            timeout,
            || file.try_lock(),
            || started.elapsed(),
            thread::sleep,
        )?;
        Ok(Self { file })
    }
}

impl Drop for MaterializeLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn wait_for_lock(
    path: &Path,
    timeout: Duration,
    mut try_lock: impl FnMut() -> Result<(), TryLockError>,
    mut elapsed: impl FnMut() -> Duration,
    mut sleep: impl FnMut(Duration),
) -> io::Result<()> {
    loop {
        match try_lock() {
            Ok(()) => return Ok(()),
            Err(TryLockError::Error(error)) if error.kind() == ErrorKind::Interrupted => {}
            Err(TryLockError::Error(error)) => return Err(error),
            Err(TryLockError::WouldBlock) => {}
        }

        let waited = elapsed();
        if waited >= timeout {
            return Err(lock_timeout_error(path, timeout));
        }
        sleep(LOCK_RETRY_DELAY.min(timeout - waited));
    }
}

fn lock_timeout_error(path: &Path, timeout: Duration) -> io::Error {
    io::Error::new(
        ErrorKind::TimedOut,
        cstr!(
            "timed out after {timeout:?} waiting for materialization lock `{}`; another Vize process may still be writing this cache. Wait for it to finish or terminate the stuck process, then retry (the OS releases this lock when its owner exits)",
            path.display()
        ),
    )
}

fn lock_path_for(virtual_root: &Path) -> PathBuf {
    lock_path_with_suffix(virtual_root, LOCK_SUFFIX)
}

fn lock_path_with_suffix(virtual_root: &Path, suffix: &str) -> PathBuf {
    let Some(file_name) = virtual_root.file_name() else {
        return virtual_root.with_extension(suffix.trim_start_matches('.'));
    };

    let mut lock_name = OsString::from(file_name);
    lock_name.push(suffix);
    virtual_root.with_file_name(lock_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::sync::mpsc;

    const CHILD_ROOT_ENV: &str = "VIZE_MATERIALIZE_LOCK_CHILD_ROOT";
    const CHILD_READY_ENV: &str = "VIZE_MATERIALIZE_LOCK_CHILD_READY";

    #[test]
    fn lock_waits_until_existing_holder_drops() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("node_modules/.vize/canon");
        let first = MaterializeLock::acquire(&root).unwrap();
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let root_for_thread = root.clone();
        let handle = thread::spawn(move || {
            let _second = MaterializeLock::acquire(&root_for_thread).unwrap();
            acquired_tx.send(()).unwrap();
        });

        assert!(acquired_rx.recv_timeout(Duration::from_millis(75)).is_err());
        drop(first);
        acquired_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn bounded_wait_returns_actionable_timeout() {
        let now = Cell::new(Duration::ZERO);
        let sleeps = Cell::new(0usize);
        let path = Path::new("project.materialize.lock");

        let error = wait_for_lock(
            path,
            Duration::from_millis(60),
            || Err(TryLockError::WouldBlock),
            || now.get(),
            |delay| {
                sleeps.set(sleeps.get() + 1);
                now.set(now.get() + delay);
            },
        )
        .unwrap_err();

        assert_eq!(error.kind(), ErrorKind::TimedOut);
        assert_eq!(now.get(), Duration::from_millis(60));
        assert_eq!(sleeps.get(), 3);
        let message = error.to_string();
        assert!(message.contains("project.materialize.lock"));
        assert!(message.contains("terminate the stuck process, then retry"));
        assert!(message.contains("OS releases this lock when its owner exits"));
    }

    #[test]
    fn wait_retries_contention_but_propagates_io_failures() {
        let attempts = Cell::new(0usize);
        let now = Cell::new(Duration::ZERO);
        wait_for_lock(
            Path::new("lock"),
            Duration::from_secs(1),
            || {
                attempts.set(attempts.get() + 1);
                match attempts.get() {
                    1 => Err(TryLockError::Error(io::Error::from(ErrorKind::Interrupted))),
                    2 => Err(TryLockError::WouldBlock),
                    _ => Ok(()),
                }
            },
            || now.get(),
            |delay| now.set(now.get() + delay),
        )
        .unwrap();
        assert_eq!(attempts.get(), 3);

        let error = wait_for_lock(
            Path::new("lock"),
            Duration::from_secs(1),
            || Err(TryLockError::Error(io::Error::other("boom"))),
            || Duration::ZERO,
            |_| panic!("I/O errors must not be retried"),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "boom");
    }

    #[test]
    fn windows_lock_path_avoids_the_legacy_directory_name() {
        let root = Path::new("node_modules/.vize/canon");
        let legacy = lock_path_with_suffix(root, LEGACY_LOCK_SUFFIX);
        let current = lock_path_with_suffix(root, WINDOWS_LOCK_SUFFIX);

        assert_eq!(legacy, Path::new("node_modules/.vize/canon.lock"));
        assert_eq!(
            current,
            Path::new("node_modules/.vize/canon.materialize.lock")
        );
        assert_ne!(current, legacy);
    }

    #[cfg(windows)]
    #[test]
    fn stale_legacy_directory_does_not_block_windows_locking() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("node_modules/.vize/canon");
        fs::create_dir_all(lock_path_with_suffix(&root, LEGACY_LOCK_SUFFIX)).unwrap();

        let lock = MaterializeLock::acquire_with_timeout(&root, Duration::from_secs(1)).unwrap();
        drop(lock);
    }

    #[test]
    fn os_lock_is_released_after_abrupt_owner_termination() {
        if let (Some(root), Some(ready)) = (
            std::env::var_os(CHILD_ROOT_ENV),
            std::env::var_os(CHILD_READY_ENV),
        ) {
            let _lock = MaterializeLock::acquire(Path::new(&root)).unwrap();
            fs::write(ready, b"ready").unwrap();
            thread::sleep(Duration::from_secs(60));
            return;
        }

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("node_modules/.vize/canon");
        let ready = temp.path().join("owner-ready");
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("os_lock_is_released_after_abrupt_owner_termination")
            .arg("--nocapture")
            .env(CHILD_ROOT_ENV, &root)
            .env(CHILD_READY_ENV, &ready)
            .spawn()
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready.exists() {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("lock owner exited before acquiring the lock: {status}");
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("lock owner did not become ready within 5s");
            }
            thread::sleep(Duration::from_millis(10));
        }

        child.kill().unwrap();
        child.wait().unwrap();
        let recovered =
            MaterializeLock::acquire_with_timeout(&root, Duration::from_secs(1)).unwrap();
        drop(recovered);
    }
}
