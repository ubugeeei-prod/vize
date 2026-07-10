use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(not(unix))]
use std::thread;
#[cfg(not(unix))]
use std::time::Duration;

use super::error::CorsaResult;

#[cfg(not(unix))]
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(25);

#[cfg(unix)]
pub(super) struct MaterializeLock {
    file: fs::File,
}

#[cfg(not(unix))]
pub(super) struct MaterializeLock {
    path: PathBuf,
}

#[cfg(unix)]
impl MaterializeLock {
    pub(super) fn acquire(virtual_root: &Path) -> CorsaResult<Self> {
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
        lock_file(&file)?;
        Ok(Self { file })
    }
}

#[cfg(not(unix))]
impl MaterializeLock {
    pub(super) fn acquire(virtual_root: &Path) -> CorsaResult<Self> {
        let path = lock_path_for(virtual_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        loop {
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    thread::sleep(LOCK_RETRY_DELAY);
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
}

#[cfg(unix)]
impl Drop for MaterializeLock {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

#[cfg(not(unix))]
impl Drop for MaterializeLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn lock_path_for(virtual_root: &Path) -> PathBuf {
    let Some(file_name) = virtual_root.file_name() else {
        return virtual_root.with_extension("lock");
    };

    let mut lock_name = OsString::from(file_name);
    lock_name.push(".lock");
    virtual_root.with_file_name(lock_name)
}

#[cfg(unix)]
fn lock_file(file: &fs::File) -> io::Result<()> {
    loop {
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn lock_waits_until_existing_holder_drops() {
        let root = std::env::temp_dir()
            .join("vize-canon-lock-tests")
            .join(std::process::id().to_string())
            .join("node_modules/.vize/canon");
        let _ = fs::remove_dir_all(root.parent().unwrap());

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

        let _ = fs::remove_dir_all(root.parent().unwrap());
    }
}
