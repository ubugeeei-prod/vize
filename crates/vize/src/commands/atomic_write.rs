//! Failure-safe replacement for source-mutating commands.

use std::borrow::Cow;
use std::fs::{self, File, Permissions};
use std::io::{self, Write};
use std::path::Path;

use tempfile::NamedTempFile;

/// Replace an existing file only after its successor is fully written and synced.
///
/// `tempfile` creates the sibling with exclusive create semantics, so an
/// existing file or symlink can never redirect writes. Its persist operation
/// uses an overwriting rename on Unix and `MoveFileExW(REPLACE_EXISTING)` on
/// Windows, without deleting the destination first.
pub(super) fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
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
    let mut temporary = tempfile::Builder::new()
        .prefix(".vize-write-")
        .suffix(".tmp")
        .tempfile_in(directory)?;

    operations.write_all(temporary.as_file_mut(), contents)?;
    operations.flush(temporary.as_file_mut())?;
    temporary.as_file().set_permissions(permissions)?;
    operations.sync_all(temporary.as_file())?;
    operations.replace(temporary, path)
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
    fn sync_all(&mut self, file: &File) -> io::Result<()>;
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

    fn sync_all(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()
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
        Sync,
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

        fn sync_all(&mut self, file: &File) -> io::Result<()> {
            file.sync_all()?;
            if self.0 == FailureStage::Sync {
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
            FailureStage::Sync,
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
}
