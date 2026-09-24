//! Filesystem primitives for `vize lib`: hashing, path safety, atomic writes.

use std::fmt::Write as _;
use std::fs;
use std::io::{ErrorKind, Write as _};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};

/// Lowercase hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest.iter() {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// SHA-256 of a local file, or `None` when it does not exist.
pub fn file_sha256(path: &Path) -> LibResult<Option<String>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(sha256_hex(&bytes))),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(LibError::io("read", path, &error)),
    }
}

/// Item hash over `path\0sha256\n` lines in path order (matches the registry).
pub fn content_hash<'a>(files: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut entries: Vec<(&str, &str)> = files.into_iter().collect();
    entries.sort_unstable();
    let mut buffer = Vec::new();
    for (path, sha) in entries {
        buffer.extend_from_slice(path.as_bytes());
        buffer.push(0);
        buffer.extend_from_slice(sha.as_bytes());
        buffer.push(b'\n');
    }
    sha256_hex(&buffer)
}

/// Validate a registry-relative POSIX path: relative, no `..`, no empty or
/// `.` segments, no backslashes. Returns the path unchanged when valid.
pub fn validate_relative_path(path: &str) -> LibResult<&str> {
    let invalid = path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..");
    if invalid {
        return Err(LibError::new(cstr!("unsafe registry path {path:?}")));
    }
    Ok(path)
}

/// Join a validated registry path below `base`.
pub fn join_relative(base: &Path, path: &str) -> LibResult<PathBuf> {
    let path = validate_relative_path(path)?;
    Ok(path
        .split('/')
        .fold(base.to_path_buf(), |joined, segment| joined.join(segment)))
}

/// Keep a destination inside the canonical project root without following
/// existing symlinks in any of its path components. A lexical `..` check is
/// insufficient: a directory below the project may itself point elsewhere.
pub fn ensure_project_path(root: &Path, path: &Path) -> LibResult<()> {
    let relative = path.strip_prefix(root).map_err(|_| {
        LibError::new(cstr!(
            "destination {} is outside the project root {}",
            path.display(),
            root.display()
        ))
    })?;
    let mut current = root.to_path_buf();
    let components: Vec<_> = relative.components().collect();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(segment) = component else {
            return Err(LibError::new(cstr!(
                "destination {} must stay inside the project root",
                path.display()
            )));
        };
        current.push(segment);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(LibError::new(cstr!(
                    "destination {} contains a symbolic link at {}",
                    path.display(),
                    current.display()
                )));
            }
            Ok(metadata) if index + 1 < components.len() && !metadata.is_dir() => {
                return Err(LibError::new(cstr!(
                    "destination {} has a non-directory parent {}",
                    path.display(),
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(LibError::io("inspect", &current, &error)),
        }
    }
    Ok(())
}

/// Normalize a user-supplied directory to a project-relative POSIX path.
///
/// Absolute paths must live inside `root`; relative paths may not climb out of it.
pub fn project_relative_dir(root: &Path, dir: &Path) -> LibResult<String> {
    let relative = if dir.is_absolute() {
        dir.strip_prefix(root).map_err(|_| {
            LibError::new(cstr!(
                "{} is outside the project root {}",
                dir.display(),
                root.display()
            ))
        })?
    } else {
        dir
    };
    let mut segments: Vec<String> = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(segment) => segments.push(segment.to_string_lossy().into()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(LibError::new(cstr!(
                    "target directory {} must stay inside the project root",
                    dir.display()
                )));
            }
        }
    }
    if segments.is_empty() {
        return Ok(String::from("."));
    }
    let normalized = segments.join("/");
    ensure_project_path(root, &root.join(&normalized))?;
    Ok(normalized.into())
}

/// Write `contents` to `path`, creating parent directories; existing files are
/// replaced through [`crate::commands::atomic_write::atomic_write`] and new files
/// are persisted from an exclusive temporary sibling.
pub fn write_file(path: &Path, contents: &[u8]) -> LibResult<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| LibError::io("create", parent, &error))?;
    if fs::symlink_metadata(path).is_ok() {
        return crate::commands::atomic_write::atomic_write(path, contents)
            .map_err(|error| LibError::io("write", path, &error));
    }
    let mut temporary = tempfile::Builder::new()
        .prefix(".vize-lib-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|error| LibError::io("create a temporary file in", parent, &error))?;
    temporary
        .write_all(contents)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| LibError::io("write", path, &error))?;
    temporary
        .persist_noclobber(path)
        .map(drop)
        .map_err(|error| LibError::io("create", path, &error.error))
}

/// Delete a file (missing is fine) and prune empty parents up to `stop`.
pub fn remove_file_and_prune(path: &Path, stop: &Path) -> LibResult<()> {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(LibError::io("remove", path, &error)),
    }
    let mut current = path.parent();
    while let Some(directory) = current {
        if directory == stop || !directory.starts_with(stop) {
            break;
        }
        if fs::remove_dir(directory).is_err() {
            break;
        }
        current = directory.parent();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        content_hash, ensure_project_path, project_relative_dir, sha256_hex, validate_relative_path,
    };
    use std::path::Path;

    #[test]
    fn hashes_match_the_node_registry_builder() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let sha = sha256_hex(b"x");
        assert_eq!(
            content_hash([("b.ts", sha.as_str()), ("a.ts", sha.as_str())]),
            content_hash([("a.ts", sha.as_str()), ("b.ts", sha.as_str())])
        );
    }

    #[test]
    fn rejects_paths_that_escape_the_target() {
        for path in [
            "",
            "/abs.ts",
            "../x.ts",
            "a/../../x.ts",
            "a//b.ts",
            "a\\b.ts",
            "c:x",
            "./a",
        ] {
            assert!(validate_relative_path(path).is_err(), "{path}");
        }
        assert!(validate_relative_path("families/form/rating/rating.vue").is_ok());
    }

    #[test]
    fn normalizes_target_directories_inside_the_root() {
        let root = Path::new("/project");
        assert_eq!(
            project_relative_dir(root, Path::new("./src/ui")).unwrap(),
            "src/ui"
        );
        assert_eq!(
            project_relative_dir(root, Path::new("/project/src")).unwrap(),
            "src"
        );
        assert!(project_relative_dir(root, Path::new("../elsewhere")).is_err());
        assert!(project_relative_dir(root, Path::new("/elsewhere")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_directories_and_files() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("app");
        let outside = temporary.path().join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("linked")).unwrap();
        assert!(ensure_project_path(&root, &root.join("linked/file.ts")).is_err());
        assert!(project_relative_dir(&root, Path::new("linked")).is_err());

        std::fs::write(outside.join("file.ts"), "unchanged").unwrap();
        symlink(outside.join("file.ts"), root.join("file.ts")).unwrap();
        assert!(ensure_project_path(&root, &root.join("file.ts")).is_err());
        assert!(ensure_project_path(&root, &root.join("../outside/file.ts")).is_err());
        assert!(ensure_project_path(&root, &root.join("new/child.ts")).is_ok());
    }
}
