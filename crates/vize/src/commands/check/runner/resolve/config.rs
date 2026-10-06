//! Locating the nearest TypeScript or JavaScript project configuration.

use std::path::{Path, PathBuf};

pub(in super::super) fn project_config_path(dir: &Path) -> Option<PathBuf> {
    ["tsconfig.json", "jsconfig.json"]
        .into_iter()
        .map(|name| dir.join(name))
        .find(|candidate| candidate.exists())
}

pub(in super::super) fn find_nearest_tsconfig_path(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        if let Some(config) = project_config_path(dir) {
            return Some(config);
        }
        current = dir.parent();
    }

    None
}

pub(in super::super) fn find_nearest_tsconfig_dir(path: &Path) -> Option<PathBuf> {
    find_nearest_tsconfig_path(path).and_then(|config| config.parent().map(Path::to_path_buf))
}

#[cfg(test)]
mod tests;
