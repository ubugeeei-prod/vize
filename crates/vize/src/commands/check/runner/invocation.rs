//! Resolves the program selected by the original `check` invocation.

use std::path::{Path, PathBuf};

use super::resolve::{resolve_project_root, resolve_tsconfig_path};

/// Resolve before collecting transitive imports: imports outside the package
/// may widen the virtual mirror, but must not select a different `tsconfig`.
pub(super) fn resolve_invocation_program(
    effective_tsconfig: Option<&Path>,
    cwd: &Path,
) -> (PathBuf, Option<PathBuf>) {
    let project_root = resolve_project_root(effective_tsconfig, cwd, &[]);
    let tsconfig_path = resolve_tsconfig_path(effective_tsconfig, cwd, &project_root, &[]);
    (project_root, tsconfig_path)
}

pub(super) fn resolve_nuxt_project_root(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    fallback: &Path,
) -> PathBuf {
    let Some(tsconfig) = explicit_tsconfig else {
        return fallback.to_path_buf();
    };
    let tsconfig_path = if tsconfig.is_absolute() {
        tsconfig.to_path_buf()
    } else {
        cwd.join(tsconfig)
    };
    let tsconfig_dir = vize_s0::path::canonicalize_non_verbatim(&tsconfig_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| fallback.to_path_buf());
    if super::super::nuxt::is_nuxt_project_root(&tsconfig_dir) {
        return tsconfig_dir;
    }
    if tsconfig_dir.join("package.json").exists() {
        return tsconfig_dir;
    }
    if let Some(parent) = tsconfig_dir.parent()
        && super::super::nuxt::is_nuxt_project_root(parent)
    {
        return parent.to_path_buf();
    }
    tsconfig_dir
}
