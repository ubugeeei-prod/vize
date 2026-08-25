//! Deterministic, workspace-bounded source discovery.

use ignore::WalkBuilder;
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_s0::{FxHashSet, String};

use super::DoctorError;

pub(super) struct DoctorSource {
    pub(super) path: PathBuf,
    pub(super) source: String,
}

pub(super) fn discover_sources(
    root: &Path,
    inputs: &[String],
) -> Result<Vec<DoctorSource>, DoctorError> {
    let root = root.canonicalize().map_err(|_| DoctorError::InvalidInput {
        path: root.to_path_buf(),
        reason: "the workspace root does not exist or cannot be resolved",
    })?;
    let mut paths = Vec::new();
    let mut seen = FxHashSet::default();

    for input in inputs {
        let requested = Path::new(input.as_str());
        let candidate = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            root.join(requested)
        };
        let resolved = candidate
            .canonicalize()
            .map_err(|_| DoctorError::InvalidInput {
                path: candidate.clone(),
                reason: "the path does not exist or cannot be resolved",
            })?;
        if !resolved.starts_with(&root) {
            return Err(DoctorError::InvalidInput {
                path: resolved,
                reason: "the path is outside the workspace root",
            });
        }
        if resolved.is_file() {
            if !is_supported_source(&resolved) {
                return Err(DoctorError::InvalidInput {
                    path: resolved,
                    reason: "the file type is not supported by whole-application analysis",
                });
            }
            add_source_path(&root, &resolved, &mut paths, &mut seen)?;
        } else if resolved.is_dir() {
            discover_directory(&root, &resolved, &mut paths, &mut seen)?;
        } else {
            return Err(DoctorError::InvalidInput {
                path: resolved,
                reason: "the path is not a regular file or directory",
            });
        }
    }

    paths.sort();
    let reads = paths
        .into_par_iter()
        .map(|path| {
            let source = fs::read_to_string(root.join(&path))
                .map(String::from)
                .map_err(|source| DoctorError::ReadSource {
                    path: path.clone(),
                    source,
                })?;
            Ok(DoctorSource { path, source })
        })
        .collect::<Vec<_>>();
    reads.into_iter().collect()
}

fn discover_directory(
    root: &Path,
    directory: &Path,
    paths: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) -> Result<(), DoctorError> {
    for entry in WalkBuilder::new(directory)
        .standard_filters(true)
        .hidden(true)
        .follow_links(false)
        .build()
    {
        let entry = entry.map_err(|source| DoctorError::WalkDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
        if entry.file_type().is_some_and(|kind| kind.is_file()) {
            add_source_path(root, entry.path(), paths, seen)?;
        }
    }
    Ok(())
}

fn add_source_path(
    root: &Path,
    absolute: &Path,
    paths: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) -> Result<(), DoctorError> {
    if !is_supported_source(absolute) {
        return Ok(());
    }
    let relative = absolute
        .strip_prefix(root)
        .map_err(|_| DoctorError::InvalidInput {
            path: absolute.to_path_buf(),
            reason: "the source is outside the workspace root",
        })?
        .to_path_buf();
    if seen.insert(relative.clone()) {
        paths.push(relative);
    }
    Ok(())
}

fn is_supported_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("vue" | "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts")
    )
}
