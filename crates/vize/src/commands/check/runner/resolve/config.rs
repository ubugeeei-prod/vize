//! Locating the nearest project configuration for check inputs.

use std::path::{Path, PathBuf};

pub(in super::super) fn find_nearest_tsconfig_dir(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        if dir.join("tsconfig.json").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    None
}

