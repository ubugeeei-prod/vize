//! Config path discovery helpers.
//!
//! Loading accepts either a direct file path or a root-like directory path. The
//! checks here stay intentionally shallow; config precedence and parse fallback
//! are handled by the parent loader.

use std::path::{Path, PathBuf};

/// Config file names in precedence order for directory auto-discovery.
pub(super) const CONFIG_FILE_NAMES: [&str; 11] = [
    "vize.config.pkl",
    "vize.config.ts",
    "vize.config.js",
    "vize.config.mjs",
    "vize.config.json",
    "vite.config.ts",
    "vite.config.js",
    "vite.config.mjs",
    "vite.config.cjs",
    "vite.config.mts",
    "vite.config.cts",
];

/// Automatic CLI discovery stops at the nearest package/TypeScript boundary.
/// Explicit workspace roots remain shallow so multi-root LSP folders never
/// inherit a sibling or parent project's policy.
pub(super) fn search_directories(base: &Path, automatic: bool) -> Vec<PathBuf> {
    let mut directories = Vec::new();
    let mut current = base.to_path_buf();
    loop {
        directories.push(current.clone());
        if !automatic
            || ["package.json", "tsconfig.json", "jsconfig.json"]
                .iter()
                .any(|name| current.join(name).is_file())
        {
            break;
        }
        let Some(parent) = current.parent() else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent.to_path_buf();
    }
    directories
}

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
