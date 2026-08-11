//! Fail-closed ownership checks for content-addressed cache directories.

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::publish_config_atomically;

const DIGEST_LENGTH: usize = 64;
const SHARD_LENGTH: usize = 2;

pub(in crate::commands::check::runner::nuxt_tsconfig) fn ensure_bucket(
    cache_root: &Path,
    shard: &str,
) -> Result<PathBuf, std::io::Error> {
    ensure_owned_directory(cache_root, shard, "bucket", is_shard)
}

pub(in crate::commands::check::runner::nuxt_tsconfig) fn ensure_project(
    bucket: &Path,
    digest: &str,
) -> Result<PathBuf, std::io::Error> {
    ensure_owned_directory(bucket, digest, "project", is_digest)
}

pub(in crate::commands::check::runner::nuxt_tsconfig) fn ensure_entry(
    project_cache: &Path,
    digest: &str,
) -> Result<PathBuf, std::io::Error> {
    ensure_owned_directory(project_cache, digest, "entry", is_digest)
}

pub(in crate::commands::check::runner::nuxt_tsconfig) fn validate_project(
    bucket: &Path,
    project: &Path,
) -> Result<bool, std::io::Error> {
    let Some(name) = project.file_name().and_then(|name| name.to_str()) else {
        return Ok(false);
    };
    if !is_digest(name) {
        return Ok(false);
    }
    validate_collectable_directory(bucket, project, name, "project")
}

pub(in crate::commands::check::runner::nuxt_tsconfig) fn validate_entry(
    project_cache: &Path,
    entry: &Path,
) -> Result<bool, std::io::Error> {
    let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
        return Ok(false);
    };
    if !is_digest(name) {
        return Ok(false);
    }
    validate_collectable_directory(project_cache, entry, name, "entry")
}

/// A directory whose ownership marker is missing is an interrupted creation,
/// not foreign state: collection scans skip it instead of failing the check.
fn validate_collectable_directory(
    parent: &Path,
    path: &Path,
    identity: &str,
    kind: &str,
) -> Result<bool, std::io::Error> {
    match validate_owned_directory(parent, path, identity, kind) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn ensure_owned_directory(
    parent: &Path,
    identity: &str,
    kind: &str,
    validate_identity: fn(&str) -> bool,
) -> Result<PathBuf, std::io::Error> {
    if !validate_identity(identity) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Nuxt config cache identity is not a lowercase hexadecimal identity",
        ));
    }
    let path = parent.join(identity);
    match fs::create_dir(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    validate_directory_path(parent, &path)?;
    let marker = path.join(format!(".{kind}-owner"));
    let expected = format!("vize-nuxt-{kind}:v2:{identity}\n");
    if !marker.exists() {
        let mut saw_entry = false;
        let mut has_bootstrap_lock = false;
        let mut unknown = false;
        for entry in fs::read_dir(&path)?.filter_map(Result::ok) {
            saw_entry = true;
            let name = entry.file_name();
            match name.to_str() {
                Some(".publish.lock") => {
                    let metadata = fs::symlink_metadata(entry.path())?;
                    has_bootstrap_lock = !metadata.file_type().is_symlink() && metadata.is_file();
                    unknown |= !has_bootstrap_lock;
                }
                Some(name) if super::is_pending_name(name) => {}
                _ => unknown = true,
            }
        }
        if unknown || (saw_entry && !has_bootstrap_lock) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Nuxt config cache directory has no ownership marker",
            ));
        }
        publish_config_atomically(&marker, expected.as_bytes())?;
    }
    validate_owned_directory(parent, &path, identity, kind)?;
    Ok(path)
}

fn validate_owned_directory(
    parent: &Path,
    path: &Path,
    identity: &str,
    kind: &str,
) -> Result<(), std::io::Error> {
    validate_directory_path(parent, path)?;
    let marker = path.join(format!(".{kind}-owner"));
    let metadata = fs::symlink_metadata(&marker)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Nuxt config cache ownership marker is not a regular file",
        ));
    }
    let expected = format!("vize-nuxt-{kind}:v2:{identity}\n");
    if fs::read(&marker)? != expected.as_bytes() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Nuxt config cache ownership marker has unexpected bytes",
        ));
    }
    Ok(())
}

fn validate_directory_path(parent: &Path, path: &Path) -> Result<(), std::io::Error> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Nuxt config cache path is not an owned directory",
        ));
    }
    let parent = fs::canonicalize(parent)?;
    let path = fs::canonicalize(path)?;
    if path.parent() != Some(parent.as_path()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Nuxt config cache directory escapes its owned parent",
        ));
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    value.len() == DIGEST_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_shard(value: &str) -> bool {
    value.len() == SHARD_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(all(test, unix))]
mod tests {
    use super::{ensure_bucket, ensure_entry, ensure_project};
    use std::{fs, os::unix::fs::symlink};

    #[test]
    fn project_and_entry_symlinks_never_mutate_their_targets() {
        let case = tempfile::tempdir().unwrap();
        let cache = case.path().join("cache");
        let dependency = case.path().join("node_modules/package");
        fs::create_dir_all(&cache).unwrap();
        fs::create_dir_all(&dependency).unwrap();
        let sentinel = dependency.join("sentinel.txt");
        fs::write(&sentinel, "owned by dependency\n").unwrap();

        let bucket = ensure_bucket(&cache, "00").unwrap();
        let project_digest = format!("{:064x}", 1);
        symlink(&dependency, bucket.join(&project_digest)).unwrap();
        assert!(ensure_project(&bucket, &project_digest).is_err());
        assert_eq!(
            fs::read_to_string(&sentinel).unwrap(),
            "owned by dependency\n"
        );

        fs::remove_file(bucket.join(&project_digest)).unwrap();
        let project = ensure_project(&bucket, &project_digest).unwrap();
        let entry_digest = format!("{:064x}", 2);
        symlink(&dependency, project.join(&entry_digest)).unwrap();
        assert!(ensure_entry(&project, &entry_digest).is_err());
        assert_eq!(
            fs::read_to_string(&sentinel).unwrap(),
            "owned by dependency\n"
        );
    }

    #[test]
    fn a_foreign_digest_directory_is_never_adopted() {
        let case = tempfile::tempdir().unwrap();
        let cache = case.path().join("cache");
        fs::create_dir(&cache).unwrap();
        let bucket = ensure_bucket(&cache, "00").unwrap();
        let digest = format!("{:064x}", 3);
        let foreign = bucket.join(&digest);
        fs::create_dir(&foreign).unwrap();
        let sentinel = foreign.join("foreign.txt");
        fs::write(&sentinel, "foreign\n").unwrap();

        assert!(ensure_project(&bucket, &digest).is_err());
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "foreign\n");
    }

    #[test]
    fn an_in_progress_bootstrap_lock_is_the_only_adoptable_regular_file() {
        let case = tempfile::tempdir().unwrap();
        let cache = case.path().join("cache");
        fs::create_dir(&cache).unwrap();
        let bucket = cache.join("ab");
        fs::create_dir(&bucket).unwrap();
        fs::write(bucket.join(".publish.lock"), []).unwrap();

        assert_eq!(ensure_bucket(&cache, "ab").unwrap(), bucket);
        assert_eq!(
            fs::read_to_string(bucket.join(".bucket-owner")).unwrap(),
            "vize-nuxt-bucket:v2:ab\n"
        );
    }

    #[test]
    fn a_pending_named_file_without_the_bootstrap_lock_is_foreign() {
        let case = tempfile::tempdir().unwrap();
        let cache = case.path().join("cache");
        fs::create_dir(&cache).unwrap();
        let digest = format!("{:064x}", 4);
        let foreign = cache.join(&digest);
        fs::create_dir(&foreign).unwrap();
        let pending = foreign.join(".vize-nuxt-config-1-foreign.pending");
        fs::write(&pending, "foreign\n").unwrap();

        assert!(ensure_project(&cache, &digest).is_err());
        assert_eq!(fs::read_to_string(pending).unwrap(), "foreign\n");
        assert!(!foreign.join(".project-owner").exists());
    }

    #[test]
    fn concurrent_first_users_publish_one_exact_bucket_identity() {
        let case = tempfile::tempdir().unwrap();
        let cache = case.path().join("cache");
        fs::create_dir(&cache).unwrap();
        let start = std::sync::Arc::new(std::sync::Barrier::new(3));
        let tasks = (0..2)
            .map(|_| {
                let cache = cache.clone();
                let start = std::sync::Arc::clone(&start);
                std::thread::spawn(move || {
                    start.wait();
                    ensure_bucket(&cache, "cd").unwrap()
                })
            })
            .collect::<Vec<_>>();
        start.wait();
        let paths = tasks
            .into_iter()
            .map(|task| task.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(paths[0], paths[1]);
        assert_eq!(
            fs::read_to_string(paths[0].join(".bucket-owner")).unwrap(),
            "vize-nuxt-bucket:v2:cd\n"
        );
    }
}
