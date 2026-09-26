//! Native typed-environment transport and implicit dependency identity.

use crate::host::HostError;
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use vize_l0::{String, cstr};

pub(super) fn build(
    moonc: &Path,
    std_path: &Path,
    scratch: &Path,
    text: &str,
) -> Result<PathBuf, HostError> {
    let source = scratch.join("environment.mbti");
    let interface = scratch.join("environment.mi");
    std::fs::write(&source, text)
        .map_err(|error| HostError::Failed(cstr!("interface write: {error}")))?;
    let mut prelude = OsString::from(std_path.join("prelude/prelude.mi"));
    prelude.push(":prelude");
    let output = Command::new(moonc)
        .current_dir(scratch)
        .arg("build-interface")
        .arg(&source)
        .arg("-o")
        .arg(&interface)
        .args(["-pkg", "vize/environment", "-error-format", "json"])
        .arg("-std-path")
        .arg(std_path)
        .arg("-i")
        .arg(prelude)
        .output()
        .map_err(|error| HostError::Unavailable(cstr!("{error}")))?;
    if !output.status.success() {
        return Err(HostError::Failed(cstr!(
            "typed environment: {}",
            core::str::from_utf8(&output.stderr)
                .unwrap_or_default()
                .trim()
        )));
    }
    Ok(interface)
}

/// Hash the ordered relative names and bytes of every `.mi` input. Reading
/// fresh at lookup invalidates a cache if a core interface changes in place.
pub(super) fn dependency_key(root: &Path) -> Result<String, HostError> {
    let mut files = Vec::new();
    collect(root, &mut files)
        .map_err(|error| HostError::Unavailable(cstr!("core interfaces: {error}")))?;
    files.sort();
    let mut digest = Sha256::new();
    for file in files {
        let name = file
            .strip_prefix(root)
            .map_err(|error| HostError::Failed(cstr!("{error}")))?;
        let name = name.to_string_lossy();
        let bytes =
            std::fs::read(&file).map_err(|error| HostError::Unavailable(cstr!("{error}")))?;
        digest.update(name.len().to_le_bytes());
        digest.update(name.as_bytes());
        digest.update(bytes.len().to_le_bytes());
        digest.update(bytes);
    }
    let mut key = String::default();
    for byte in digest.finalize() {
        vize_l0::append!(key, "{byte:02x}");
    }
    Ok(key)
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_symlink() {
            let target = std::fs::metadata(&path)?;
            if target.is_dir() {
                return Err(std::io::Error::other(
                    "core interface directory symlinks are unsupported",
                ));
            }
            if target.is_file() && path.extension().is_some_and(|extension| extension == "mi") {
                files.push(path);
            }
        } else if kind.is_dir() {
            collect(&path, files)?;
        } else if kind.is_file() && path.extension().is_some_and(|extension| extension == "mi") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::dependency_key;
    use vize_l0::cstr;

    #[test]
    fn dependency_identity_tracks_contents_and_relative_names_not_scratch_paths() {
        let run = crate::native::RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir()
            .join(cstr!("vize-moonbit-core-key-{}-{run}", std::process::id()).as_str());
        let first = directory.join("first");
        let second = directory.join("second");
        for root in [&first, &second] {
            std::fs::create_dir_all(root.join("prelude")).unwrap();
            std::fs::write(root.join("prelude/prelude.mi"), b"first-interface").unwrap();
        }
        let original = dependency_key(&first).unwrap();
        assert_eq!(original, dependency_key(&second).unwrap());
        std::fs::write(first.join("prelude/prelude.mi"), b"second-interface").unwrap();
        let changed = dependency_key(&first).unwrap();
        assert_ne!(original, changed);
        std::fs::rename(
            first.join("prelude/prelude.mi"),
            first.join("prelude/renamed.mi"),
        )
        .unwrap();
        assert_ne!(changed, dependency_key(&first).unwrap());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
