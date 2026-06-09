//! Glob and path normalization helpers for tsconfig input resolution.

use std::path::{Path, PathBuf};

use super::spec::GlobSpec;

pub(super) fn normalize_tsconfig_glob(value: &str) -> std::string::String {
    let mut normalized = value.replace('\\', "/");
    if normalized.is_empty() {
        normalized.push_str("**/*");
        return normalized;
    }

    if normalized == "." {
        normalized.clear();
        normalized.push_str("**/*");
        return normalized;
    }

    if normalized.contains(['*', '?', '[']) {
        return normalized;
    }

    let has_extension = Path::new(&normalized).extension().is_some();
    if has_extension {
        return normalized;
    }

    if !normalized.ends_with('/') {
        normalized.push('/');
    }
    normalized.push_str("**/*");
    normalized
}

pub(super) fn normalize_tsconfig_glob_base(
    base_dir: &Path,
    value: &str,
) -> (PathBuf, std::string::String) {
    let mut base_dir = base_dir.to_path_buf();
    let mut normalized = normalize_tsconfig_glob(value);

    loop {
        if let Some(rest) = normalized.strip_prefix("./") {
            normalized = rest.to_owned();
        } else if let Some(rest) = normalized.strip_prefix("../") {
            if let Some(parent) = base_dir.parent() {
                base_dir = parent.to_path_buf();
            }
            normalized = rest.to_owned();
        } else {
            break;
        }
    }

    if normalized.is_empty() {
        normalized.push_str("**/*");
    }

    (base_dir, normalized)
}

pub(super) fn default_exclude_specs(base_dir: &Path) -> Vec<GlobSpec> {
    ["node_modules", "bower_components", "jspm_packages"]
        .into_iter()
        .filter_map(|value| GlobSpec::new(base_dir, value))
        .collect()
}

pub(super) fn normalize_path_separators(path: &Path) -> std::string::String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn normalize_input_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub(super) fn normalize_walked_path(root: &Path, normalized_root: &Path, path: &Path) -> PathBuf {
    // Avoid a canonicalize syscall per walked file; normalize the root once.
    path.strip_prefix(root)
        .map(|relative| normalized_root.join(relative))
        .unwrap_or_else(|_| normalize_input_path(path))
}
