//! Resolve regular Git metadata before the walker reads any ignore authority.
use std::fs;
use std::path::{Component, Path, PathBuf};
use vize_l0::ToCompactString;

use super::{Refusal, RefusalKind, SourceCustody, SourceRole};

pub(super) struct Resolved {
    pub common: PathBuf,
    pub sources: Vec<SourceCustody>,
}

fn legacy_refusal(marker: &Path) -> Refusal {
    Refusal::new(
        RefusalKind::LinkedGitMetadata,
        marker,
        "regular .git directory required",
    )
}

fn metadata_file(path: &Path) -> Result<Option<Vec<u8>>, Refusal> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => Err(Refusal::new(
            RefusalKind::Symlink,
            path,
            "Git metadata symlink is unqualified",
        )),
        Ok(metadata) if !metadata.is_file() => Err(Refusal::new(
            RefusalKind::UnsupportedFileType,
            path,
            "Git metadata source must be regular",
        )),
        Ok(_) => fs::read(path)
            .map(Some)
            .map_err(|error| Refusal::io(path, error)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Refusal::io(path, error)),
    }
}

fn metadata_path<'a>(path: &Path, bytes: &'a [u8]) -> Result<&'a str, Refusal> {
    let source = std::str::from_utf8(bytes).map_err(|error| Refusal {
        kind: RefusalKind::SourceEncoding,
        path: path.to_path_buf(),
        details: error.to_compact_string(),
        original_bytes: Some(bytes.to_vec()),
    })?;
    let value = source.strip_suffix('\n').unwrap_or(source);
    let value = value.strip_suffix('\r').unwrap_or(value);
    if value.is_empty() || value.contains(['\n', '\r', '\0']) {
        return Err(legacy_refusal(path));
    }
    Ok(value)
}

fn real_directory(path: &Path, legacy_marker: Option<&Path>) -> Result<PathBuf, Refusal> {
    let absolute = std::path::absolute(path).map_err(|error| Refusal::io(path, error))?;
    let mut current = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => continue,
            Component::ParentDir => {
                current.pop();
                continue;
            }
            other => current.push(other.as_os_str()),
        }
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound
                && let Some(marker) = legacy_marker
            {
                legacy_refusal(marker)
            } else {
                Refusal::io(&current, error)
            }
        })?;
        if metadata.is_symlink() {
            return Err(Refusal::new(
                RefusalKind::Symlink,
                &current,
                "Git metadata directory symlink is unqualified",
            ));
        }
        if !metadata.is_dir() {
            return Err(Refusal::new(
                RefusalKind::UnsupportedFileType,
                &current,
                "Git metadata directory required",
            ));
        }
    }
    if current.to_str().is_none() {
        return Err(Refusal::new(
            RefusalKind::UnsupportedFileType,
            &current,
            "UTF-8 Git metadata path required",
        ));
    }
    fs::canonicalize(&current).map_err(|error| Refusal::io(&current, error))
}

/// Main repositories retain their existing source packet. A gitfile additionally
/// owns its exact bytes and the present/absent commondir source it consults.
pub(super) fn resolve(marker: &Path) -> Result<Resolved, Refusal> {
    let metadata = fs::symlink_metadata(marker).map_err(|error| Refusal::io(marker, error))?;
    if metadata.is_dir() {
        return Ok(Resolved {
            common: marker.to_path_buf(),
            sources: Vec::new(),
        });
    }
    if metadata.is_symlink() {
        return Err(Refusal::new(
            RefusalKind::Symlink,
            marker,
            "Git metadata symlink is unqualified",
        ));
    }
    if !metadata.is_file() {
        return Err(legacy_refusal(marker));
    }
    let bytes = metadata_file(marker)?.ok_or_else(|| legacy_refusal(marker))?;
    let source = metadata_path(marker, &bytes)?;
    let target = source
        .strip_prefix("gitdir: ")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| legacy_refusal(marker))?;
    let git_dir_path = marker.parent().unwrap_or(marker).join(target);
    // Keep the independently frozen deliberately-unqualified marker refusal.
    let git_dir = real_directory(&git_dir_path, Some(marker))?;
    let commondir = git_dir.join("commondir");
    let common_bytes = metadata_file(&commondir)?;
    let common = if let Some(bytes) = &common_bytes {
        real_directory(&git_dir.join(metadata_path(&commondir, bytes)?), None)?
    } else {
        git_dir
    };
    Ok(Resolved {
        common,
        sources: vec![
            SourceCustody {
                path: marker.to_path_buf(),
                role: SourceRole::GitMetadata,
                bytes: Some(bytes),
            },
            SourceCustody {
                path: commondir,
                role: SourceRole::GitMetadata,
                bytes: common_bytes,
            },
        ],
    })
}
