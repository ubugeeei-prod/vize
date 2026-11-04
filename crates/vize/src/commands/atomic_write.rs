//! Failure-safe replacement for source-mutating commands.

use std::borrow::Cow;
use std::fs::{self, File, Permissions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use tempfile::NamedTempFile;
use vize_l0::FxHashMap;

/// Replace an existing file only after its successor is fully written.
///
/// Killing the process at any point leaves either the old bytes or the new
/// ones. There is no fsync, so surviving a power loss is not promised.
///
/// `tempfile` creates the sibling with exclusive create semantics, so an
/// existing file or symlink can never redirect writes. Its persist operation
/// uses an overwriting rename on Unix and `MoveFileExW(REPLACE_EXISTING)` on
/// Windows, without deleting the destination first.
///
/// Sibling create, rename, and unlink take a userspace mutex keyed by the
/// destination directory. Those namespace operations share that directory's
/// kernel inode lock; waiting here parks instead of spinning on it. The
/// payload write stays outside the mutex.
pub fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    atomic_write_with(path, contents, &mut OsOperations)
}

fn atomic_write_with<O: AtomicWriteOperations>(
    path: &Path,
    contents: &[u8],
    operations: &mut O,
) -> io::Result<()> {
    let (path, permissions) = resolve_write_target(path)?;
    let path = path.as_ref();
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let directory_lock = directory_write_lock(directory);
    let mut temporary = {
        let _guard = lock_directory(directory_lock);
        tempfile::Builder::new()
            .prefix(".vize-write-")
            .suffix(".tmp")
            .tempfile_in(directory)?
    };

    let prepared = write_temporary(&mut temporary, contents, permissions, operations);
    let _guard = lock_directory(directory_lock);
    if let Err(error) = prepared {
        drop(temporary);
        return Err(error);
    }
    operations.replace(temporary, path)
}

fn write_temporary<O: AtomicWriteOperations>(
    temporary: &mut NamedTempFile,
    contents: &[u8],
    permissions: Permissions,
    operations: &mut O,
) -> io::Result<()> {
    operations.write_all(temporary.as_file_mut(), contents)?;
    operations.flush(temporary.as_file_mut())?;
    temporary.as_file().set_permissions(permissions)
}

type DirectoryLockMap = FxHashMap<PathBuf, &'static Mutex<()>>;

fn directory_write_lock(directory: &Path) -> &'static Mutex<()> {
    static LOCKS: OnceLock<Mutex<DirectoryLockMap>> = OnceLock::new();
    let registry = LOCKS.get_or_init(|| Mutex::new(FxHashMap::default()));
    let mut guard = registry.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(lock) = guard.get(directory) {
        return *lock;
    }
    let lock = Box::leak(Box::new(Mutex::new(())));
    guard.insert(directory.to_path_buf(), lock);
    lock
}

struct DirectoryGuard<'a> {
    _guard: MutexGuard<'a, ()>,
}

fn lock_directory(mutex: &Mutex<()>) -> DirectoryGuard<'_> {
    DirectoryGuard {
        _guard: mutex.lock().unwrap_or_else(|error| error.into_inner()),
    }
}

fn resolve_write_target(path: &Path) -> io::Result<(Cow<'_, Path>, Permissions)> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        let target = fs::canonicalize(path)?;
        let permissions = fs::metadata(&target)?.permissions();
        Ok((Cow::Owned(target), permissions))
    } else {
        Ok((Cow::Borrowed(path), metadata.permissions()))
    }
}

trait AtomicWriteOperations {
    fn write_all(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()>;
    fn flush(&mut self, file: &mut File) -> io::Result<()>;
    fn replace(&mut self, temporary: NamedTempFile, path: &Path) -> io::Result<()>;
}

struct OsOperations;

impl AtomicWriteOperations for OsOperations {
    fn write_all(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()> {
        file.write_all(contents)
    }

    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        file.flush()
    }

    fn replace(&mut self, temporary: NamedTempFile, path: &Path) -> io::Result<()> {
        temporary.persist(path).map(drop).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{AtomicWriteOperations, OsOperations, atomic_write, atomic_write_with};
    use std::fs::{self, File};
    use std::io::{self, Write};
    use std::path::Path;
    use tempfile::NamedTempFile;

    const ORIGINAL: &[u8] = b"original source\n";
    const REPLACEMENT: &[u8] = b"complete replacement\n";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FailureStage {
        Write,
        Flush,
        Replace,
    }

    struct FailingOperations(FailureStage);

    impl FailingOperations {
        fn injected_failure() -> io::Error {
            io::Error::other("injected atomic write failure")
        }
    }

    impl AtomicWriteOperations for FailingOperations {
        fn write_all(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()> {
            if self.0 == FailureStage::Write {
                file.write_all(&contents[..contents.len() / 2])?;
                return Err(Self::injected_failure());
            }
            file.write_all(contents)
        }

        fn flush(&mut self, file: &mut File) -> io::Result<()> {
            file.flush()?;
            if self.0 == FailureStage::Flush {
                return Err(Self::injected_failure());
            }
            Ok(())
        }

        fn replace(&mut self, temporary: NamedTempFile, path: &Path) -> io::Result<()> {
            if self.0 == FailureStage::Replace {
                return Err(Self::injected_failure());
            }
            OsOperations.replace(temporary, path)
        }
    }

    #[test]
    fn replacement_preserves_permissions_and_cleans_up() {
        let project = tempfile::tempdir().unwrap();
        let path = project.path().join("App.vue");
        fs::write(&path, ORIGINAL).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        }
        let original_permissions = fs::metadata(&path).unwrap().permissions();

        atomic_write(&path, REPLACEMENT).unwrap();

        assert_eq!(fs::read(&path).unwrap(), REPLACEMENT);
        let replaced_permissions = fs::metadata(&path).unwrap().permissions();
        assert_eq!(
            replaced_permissions.readonly(),
            original_permissions.readonly()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(replaced_permissions.mode(), original_permissions.mode());
        }
        assert_no_temporary_files(project.path());
    }

    #[cfg(unix)]
    #[test]
    fn replacement_follows_a_symlink_without_replacing_it() {
        use std::os::unix::fs::symlink;

        let project = tempfile::tempdir().unwrap();
        let target_directory = project.path().join("target");
        let link_directory = project.path().join("link");
        fs::create_dir_all(&target_directory).unwrap();
        fs::create_dir_all(&link_directory).unwrap();
        let target = target_directory.join("App.vue");
        let link = link_directory.join("App.vue");
        fs::write(&target, ORIGINAL).unwrap();
        symlink(&target, &link).unwrap();

        atomic_write(&link, REPLACEMENT).unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(&target).unwrap(), REPLACEMENT);
        assert_eq!(fs::read(&link).unwrap(), REPLACEMENT);
        assert_no_temporary_files(&target_directory);
        assert_no_temporary_files(&link_directory);
    }

    #[test]
    fn every_pre_replace_failure_preserves_original_bytes() {
        let project = tempfile::tempdir().unwrap();
        let path = project.path().join("App.vue");

        for stage in [
            FailureStage::Write,
            FailureStage::Flush,
            FailureStage::Replace,
        ] {
            fs::write(&path, ORIGINAL).unwrap();
            let result = atomic_write_with(&path, REPLACEMENT, &mut FailingOperations(stage));
            assert!(result.is_err(), "{stage:?} should fail");
            assert_eq!(fs::read(&path).unwrap(), ORIGINAL, "failed at {stage:?}");
            assert_no_temporary_files(project.path());
        }
    }

    fn assert_no_temporary_files(directory: &Path) {
        let temporary_count = fs::read_dir(directory)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".vize-write-")
            })
            .count();
        assert_eq!(temporary_count, 0);
    }

    /// A real child process is stopped between its complete temporary write
    /// and the rename.
    #[cfg(unix)]
    mod interruption {
        use super::{ORIGINAL, REPLACEMENT};
        use crate::commands::atomic_write::{
            AtomicWriteOperations, OsOperations, atomic_write_with,
        };
        use std::fs::{self, File};
        use std::io::{self, BufRead, BufReader, Read};
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::process::ExitStatusExt;
        use std::path::{Path, PathBuf};
        use std::process::{Child, ChildStderr, ChildStdin, Command, Stdio};
        use tempfile::NamedTempFile;

        const TARGET_ENV: &str = "VIZE_ATOMIC_WRITE_CHILD_TARGET";
        const AT_REPLACE: &str = "vize-atomic-write-child: at replace";
        const MODE: u32 = 0o640;

        /// Holds the replace stage until stdin closes.
        struct GatedOperations;

        impl AtomicWriteOperations for GatedOperations {
            fn write_all(&mut self, file: &mut File, contents: &[u8]) -> io::Result<()> {
                OsOperations.write_all(file, contents)
            }

            fn flush(&mut self, file: &mut File) -> io::Result<()> {
                OsOperations.flush(file)
            }

            fn replace(&mut self, temporary: NamedTempFile, path: &Path) -> io::Result<()> {
                eprintln!("\n{AT_REPLACE}");
                io::copy(&mut io::stdin().lock(), &mut io::sink())?;
                OsOperations.replace(temporary, path)
            }
        }

        #[test]
        fn child_replaces_target() {
            let Some(target) = std::env::var_os(TARGET_ENV) else {
                return;
            };
            atomic_write_with(Path::new(&target), REPLACEMENT, &mut GatedOperations).unwrap();
        }

        struct HeldChild {
            process: Child,
            gate: ChildStdin,
            stderr: BufReader<ChildStderr>,
        }

        fn original_target(directory: &Path) -> PathBuf {
            let target = directory.join("App.vue");
            fs::write(&target, ORIGINAL).unwrap();
            fs::set_permissions(&target, fs::Permissions::from_mode(MODE)).unwrap();
            target
        }

        /// Returns once the child sits in `replace` with the sibling fully written.
        fn spawn_held_child(target: &Path) -> HeldChild {
            let module = module_path!()
                .split_once("::")
                .map_or(module_path!(), |(_, path)| path);
            let mut process = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    &[module, "::child_replaces_target"].concat(),
                    "--nocapture",
                ])
                .env(TARGET_ENV, target)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let gate = process.stdin.take().unwrap();
            let mut stderr = BufReader::new(process.stderr.take().unwrap());
            let mut line = Vec::new();
            loop {
                line.clear();
                let read = stderr.read_until(b'\n', &mut line).unwrap();
                assert_ne!(read, 0, "child exited before reaching replace");
                if line.trim_ascii_end() == AT_REPLACE.as_bytes() {
                    break;
                }
            }
            let siblings = temporary_siblings(target.parent().unwrap());
            assert_eq!(siblings.len(), 1, "{siblings:?}");
            assert_eq!(fs::read(&siblings[0]).unwrap(), REPLACEMENT);
            assert_eq!(fs::read(target).unwrap(), ORIGINAL);
            HeldChild {
                process,
                gate,
                stderr,
            }
        }

        fn temporary_siblings(directory: &Path) -> Vec<PathBuf> {
            fs::read_dir(directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| {
                            name.starts_with(".vize-write-") && name.ends_with(".tmp")
                        })
                })
                .collect()
        }

        fn mode(path: &Path) -> u32 {
            fs::metadata(path).unwrap().permissions().mode() & 0o7777
        }

        #[test]
        fn sigkill_before_rename_leaves_the_target_untouched() {
            let project = tempfile::tempdir().unwrap();
            let target = original_target(project.path());
            let HeldChild {
                mut process,
                gate,
                stderr: _stderr,
            } = spawn_held_child(&target);

            process.kill().unwrap();
            let status = process.wait().unwrap();
            drop(gate);

            assert_eq!(status.signal(), Some(libc::SIGKILL));
            assert_eq!(fs::read(&target).unwrap(), ORIGINAL);
            assert_eq!(mode(&target), MODE);
        }

        #[test]
        fn released_child_replaces_the_target() {
            let project = tempfile::tempdir().unwrap();
            let target = original_target(project.path());
            let HeldChild {
                mut process,
                gate,
                mut stderr,
            } = spawn_held_child(&target);

            drop(gate);
            let status = process.wait().unwrap();
            if !status.success() {
                let mut rest = Vec::new();
                stderr.read_to_end(&mut rest).unwrap();
                panic!(
                    "child failed with {status}: {}",
                    std::str::from_utf8(&rest).unwrap_or("<non-UTF-8 stderr>")
                );
            }

            assert_eq!(fs::read(&target).unwrap(), REPLACEMENT);
            assert_eq!(mode(&target), MODE);
            assert!(temporary_siblings(project.path()).is_empty());
        }
    }
}

#[cfg(test)]
mod directory_contention;
