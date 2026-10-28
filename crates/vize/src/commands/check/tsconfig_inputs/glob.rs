//! Glob and path normalization helpers for tsconfig input resolution.

use std::path::{Path, PathBuf};

use super::spec::GlobSpec;

/// One half of `tsc`'s default `exclude`: the directory `key` — `outDir` or
/// `declarationDir` — names in `value`'s `compilerOptions`, used as a raw
/// exclude spec, which covers everything below it.
///
/// An empty string is not a directory and `tsc` skips it (its default-exclude
/// check is a truthiness test on the option), so it yields no spec.
pub(super) fn compiler_option_dir_exclude(
    value: &serde_json::Value,
    dir: &Path,
    key: &str,
) -> Option<GlobSpec> {
    let raw = value.get("compilerOptions")?.get(key)?.as_str()?;
    if raw.is_empty() {
        return None;
    }
    GlobSpec::exclude(dir, raw)
}

pub(in crate::commands::check) fn normalize_input_path(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}

pub(super) fn normalize_walked_path(root: &Path, normalized_root: &Path, path: &Path) -> PathBuf {
    // Avoid a canonicalize syscall per walked file; normalize the root once.
    path.strip_prefix(root)
        .map(|relative| normalized_root.join(relative))
        .unwrap_or_else(|_| normalize_input_path(path))
}
