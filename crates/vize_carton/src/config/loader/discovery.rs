//! Config path discovery helpers.
//!
//! Loading accepts either a direct file path or a root-like directory path. The
//! checks here stay intentionally shallow; config precedence and parse fallback
//! are handled by the parent loader.

use std::path::{Path, PathBuf};

/// Return a direct config file path when the user supplied a file.
pub(super) fn resolve_file_path(base: &Path) -> Option<PathBuf> {
    if base.is_file() {
        Some(base.to_path_buf())
    } else {
        None
    }
}

/// Return a directory root for config auto-discovery.
///
/// Nonexistent extensionless paths are treated as directory roots so callers can
/// ask for a project location before it has been created. Paths with an
/// extension are assumed to be explicit files and are not searched as dirs.
pub(super) fn resolve_dir_path(base: &Path) -> Option<PathBuf> {
    if base.is_dir() {
        return Some(base.to_path_buf());
    }

    if base.extension().is_none() {
        return Some(base.to_path_buf());
    }

    None
}
